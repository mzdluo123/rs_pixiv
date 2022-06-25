use crate::Md5Bytes;
use bytes::Bytes;
use lru::LruCache;
use parking_lot::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

type Lru = LruCache<Md5Bytes, Bytes>;

#[inline]
fn hash_key(k: &[u8]) -> Md5Bytes {
    md5::compute(k).into()
}

#[derive(Debug)]
struct MemCacheInner {
    cache: Mutex<Lru>,

    cap: usize,
    current: AtomicUsize,
}

#[derive(Debug)]
pub(crate) struct MemCache(Option<MemCacheInner>);

impl MemCacheInner {
    fn new(cap: usize) -> Self {
        Self {
            cache: Mutex::new(Lru::unbounded()),
            cap,
            current: AtomicUsize::new(0),
        }
    }

    fn get(&self, k: &[u8]) -> Option<Bytes> {
        let mut guard = self.cache.lock();
        guard.get(&hash_key(k)).map(Bytes::clone)
    }

    fn put(&self, k: &[u8], v: Bytes) -> Option<Bytes> {
        let mut guard = self.cache.lock();
        let v_sz = v.len();
        let other = guard.put(hash_key(k), v);

        // sub the last value and add the new one to the total bytes counter
        if let Some(ref other) = other {
            self.current.fetch_sub(other.len(), Ordering::Relaxed);
        }
        let updated_sz = self.current.fetch_add(v_sz, Ordering::SeqCst) + v_sz;

        // if the new size is greater than the cap, start evicting items
        if updated_sz > self.cap {
            self.evict(&mut guard, updated_sz);
        }

        other
    }

    fn evict(&self, lru: &mut Lru, mut current: usize) -> usize {
        // pop items until it meets size requirement
        loop {
            match lru.pop_lru() {
                Some((_, b)) => current -= b.len(),
                None => break,
            }
            if current <= self.cap {
                break;
            }
        }
        self.current.swap(current, Ordering::SeqCst);
        current
    }

    #[cfg(test)]
    fn peek(&self, k: &[u8]) -> Option<Bytes> {
        let guard = self.cache.lock();
        guard.peek(&hash_key(k)).map(Bytes::clone)
    }
}

impl MemCache {
    pub(crate) fn new(bytes_cap: usize) -> Self {
        if bytes_cap == 0 {
            Self(None)
        } else {
            Self(Some(MemCacheInner::new(bytes_cap)))
        }
    }

    #[inline]
    pub(crate) fn is_nil(&self) -> bool {
        self.0.is_none()
    }

    #[inline]
    pub(crate) fn get(&self, k: &[u8]) -> Option<Bytes> {
        self.0.as_ref().and_then(|c| c.get(k))
    }
    #[inline]
    pub(crate) fn put(&self, k: &[u8], v: Bytes) -> Option<Bytes> {
        self.0.as_ref().and_then(|c| c.put(k, v))
    }

    // functions for tests
    #[cfg(test)]
    fn peek(&self, k: &[u8]) -> Option<Bytes> {
        self.0.as_ref().and_then(|c| c.peek(k))
    }
    #[cfg(test)]
    fn current_size(&self) -> Option<usize> {
        self.0.as_ref().map(|x| x.current.load(Ordering::SeqCst))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    const D: &'static [u8] = &[0; 4096];

    #[test]
    fn verify_eviction() {
        let cache = MemCache::new(D.len() * 2);
        cache.put(b"ENT1", Bytes::from(D));
        cache.put(b"ENT2", Bytes::from(D));
        // verify that ENT1 still exists in cache (no eviction yet)
        assert!(cache.peek(b"ENT1").is_some());

        // with the new put, cache should evict ENT1
        cache.put(b"ENT3", Bytes::from(D));
        assert!(cache.peek(b"ENT1").is_none());
    }

    #[test]
    fn size_updates() {
        // make sure current size is being updated after insertion of items
        let cache = MemCache::new(D.len() * 2);
        cache.put(b"ENT1", Bytes::from(D));
        cache.put(b"ENT2", Bytes::from(D));
        assert_eq!(cache.current_size().unwrap(), D.len() * 2);

        // this will verify that the last replaced entry has its byte count removed
        cache.put(b"ENT2", Bytes::from(D));
        assert_eq!(cache.current_size().unwrap(), D.len() * 2);
    }
}
