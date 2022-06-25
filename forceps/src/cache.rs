use crate::{mem_cache::MemCache, ForcepError, MetaDb, Metadata, Result};
use bytes::Bytes;
use std::io;
use std::path;
use std::result;
use tokio::fs as afs;

/// Creates a writeable and persistent temporary file in the path provided, returning the path and
/// file handle.
async fn tempfile(dir: &path::Path) -> Result<(afs::File, path::PathBuf)> {
    let tmppath = crate::tmp::tmppath_in(dir);
    let tmp = afs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&tmppath)
        .await
        .map_err(ForcepError::Io)?;
    Ok((tmp, tmppath))
}

#[derive(Debug, Clone)]
pub(crate) struct Options {
    pub(crate) path: path::PathBuf,
    pub(crate) dir_depth: u8,
    pub(crate) track_access: bool,

    // maximum size of the in-memory lru in bytes
    pub(crate) lru_size: usize,

    // read and write buffer sizes
    pub(crate) rbuff_sz: usize,
    pub(crate) wbuff_sz: usize,
}

/// The main component of `forceps`, and  acts as the API for interacting with the on-disk cache.
///
/// This structure includes the async `read`, `write`, and `remove` operations which are the basic
/// operations of the cache. It also includes some misc functions to interact with metadata and
/// evict items from the cache.
///
/// # Eviction
///
/// This cache can evict items with a number of different eviction algorithms. To see more, see
/// [`evict_with`] and the [`evictors`] module.
///
/// # Memory Cache
///
/// An in-memory cache can be optionally enabled as a layer over the regular on-disk cache. The
/// memcache provides fast `HIT`s for recently used entries, circumventing filesystem operations
/// altogether. To enable, use the [`CacheBuilder`]`::memory_lru_max_size` method.
///
/// # Examples
///
/// ```rust
/// # #[tokio::main(flavor = "current_thread")]
/// # async fn main() {
/// use forceps::Cache;
///
/// let cache = Cache::new("./cache")
///     .build()
///     .await
///     .unwrap();
/// # }
/// ```
///
/// [`evict_with`]: #method.evict_with
/// [`evictors`]: crate::evictors
/// [`CacheBuilder`]: crate::CacheBuilder
#[derive(Debug)]
pub struct Cache {
    meta: MetaDb,
    mem: MemCache,
    opts: Options,
}

impl Cache {
    /// Creates a new [`CacheBuilder`], which can be used to customize and create a [`Cache`]
    /// instance. This function is an alias for [`CacheBuilder::new`].
    ///
    /// The `path` supplied is the base directory of the cache instance.
    ///
    /// [`CacheBuilder`]: crate::CacheBuilder
    /// [`CacheBuilder::new`]: crate::CacheBuilder::new
    ///
    /// # Examples
    ///
    /// ```rust
    /// use forceps::Cache;
    ///
    /// let builder = Cache::new("./cache");
    /// // Use other methods for configuration
    /// ```
    #[inline]
    #[allow(clippy::new_ret_no_self)]
    pub fn new<P: AsRef<path::Path>>(path: P) -> crate::CacheBuilder {
        crate::CacheBuilder::new(path)
    }

    /// Creates a new Cache instance based on the CacheBuilder
    pub(crate) async fn create(opts: Options) -> Result<Self> {
        // create the base directory for the cache
        afs::create_dir_all(&opts.path)
            .await
            .map_err(ForcepError::Io)?;

        let mut meta_path = opts.path.clone();
        meta_path.push("index");
        Ok(Self {
            meta: MetaDb::new(&meta_path)?,
            mem: MemCache::new(opts.lru_size),
            opts,
        })
    }

    /// Creates a PathBuf based on the key provided
    fn path_from_key(&self, key: &[u8]) -> path::PathBuf {
        let hex = hex::encode(key);
        let mut buf = self.opts.path.clone();

        // push segments of key as paths to the PathBuf. If the hex isn't long enough, then push
        // "__" instead.
        for n in (0..self.opts.dir_depth).map(|x| x as usize * 2) {
            let n_end = n + 2;
            buf.push(if n_end >= hex.len() {
                "__"
            } else {
                &hex[n..n_end]
            })
        }
        buf.push(&hex);
        buf
    }

    /// Tracks the access for a cache entry if the option is enabled
    #[inline]
    fn track_access_for(&self, k: &[u8]) -> Result<()> {
        if self.opts.track_access {
            self.meta.track_access_for(k)?;
        }
        Ok(())
    }

    /// Reads an entry from the database, returning a vector of bytes that represent the entry.
    ///
    /// # Not Found
    ///
    /// If the entry is not found, then it will return
    /// `Err(`[`Error::NotFound`](ForcepError::NotFound)`)`.
    ///
    /// # Metadata
    ///
    /// This function will *not* perform a metadata read or write **unless** the `track_access`
    /// build option is set. If the option is set, then it will perform a blocking read/write to
    /// write new values to track the last access time and the total hits.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() {
    /// use forceps::Cache;
    ///
    /// let cache = Cache::new("./cache")
    ///     .build()
    ///     .await
    ///     .unwrap();
    /// # cache.write(b"MY_KEY", b"Hello World").await.unwrap();
    ///
    /// let value = cache.read(b"MY_KEY").await.unwrap();
    /// assert_eq!(value.as_ref(), b"Hello World");
    /// # }
    /// ```
    pub async fn read<K: AsRef<[u8]>>(&self, key: K) -> Result<Bytes> {
        use tokio::io::AsyncReadExt;
        let k = key.as_ref();

        // look in the memory cache to see if it's there and return if it is
        if let Some(val) = self.mem.get(k) {
            return self.track_access_for(k).map(|_| val);
        }

        let file = {
            let path = self.path_from_key(k);
            afs::OpenOptions::new()
                .read(true)
                .open(&path)
                .await
                .map_err(|e| match e.kind() {
                    io::ErrorKind::NotFound => ForcepError::NotFound,
                    _ => ForcepError::Io(e),
                })?
        };

        // create a new buffer based on the estimated size of the file
        let size_guess = file.metadata().await.map(|x| x.len()).unwrap_or(0);
        let mut buf = Vec::with_capacity(size_guess as usize);

        // read the entire file to the buffer
        tokio::io::BufReader::with_capacity(self.opts.rbuff_sz, file)
            .read_to_end(&mut buf)
            .await
            .map_err(ForcepError::Io)?;

        self.track_access_for(k)?;
        let bytes = Bytes::from(buf);
        self.mem.put(k, Bytes::clone(&bytes));
        Ok(bytes)
    }

    /// Writes an entry with the specified key to the cache database. This will replace the
    /// previous entry if it exists, otherwise it will store a completely new one.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() {
    /// use forceps::Cache;
    ///
    /// let cache = Cache::new("./cache")
    ///     .build()
    ///     .await
    ///     .unwrap();
    ///
    /// cache.write(b"MY_KEY", b"Hello World").await.unwrap();
    /// # }
    /// ```
    pub async fn write<K: AsRef<[u8]>, V: AsRef<[u8]>>(
        &self,
        key: K,
        value: V,
    ) -> Result<Metadata> {
        use tokio::io::AsyncWriteExt;
        let key = key.as_ref();
        let value = value.as_ref();

        let (tmp, tmp_path) = tempfile(&self.opts.path).await?;
        // write all data to a temporary file
        {
            let mut writer = tokio::io::BufWriter::with_capacity(self.opts.wbuff_sz, tmp);
            writer.write_all(value).await.map_err(ForcepError::Io)?;
            writer.flush().await.map_err(ForcepError::Io)?;
        }

        // move the temporary file to the final destination
        let final_path = self.path_from_key(key);
        if let Some(parent) = final_path.parent() {
            afs::create_dir_all(parent).await.map_err(ForcepError::Io)?;
        }
        afs::rename(&tmp_path, &final_path)
            .await
            .map_err(ForcepError::Io)?;

        if !self.mem.is_nil() {
            self.mem.put(key, Bytes::from(Vec::from(value)));
        }
        self.meta.insert_metadata_for(key, value)
    }

    /// Removes an entry from the cache, returning its [`Metadata`].
    ///
    /// This will remove the entry from both the main cache database and the metadata database.
    /// Please note that this will return `Error::NotFound` if either the main database *or* the
    /// meta database didn't find the entry.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() {
    /// use forceps::Cache;
    ///
    /// let cache = Cache::new("./cache")
    ///     .build()
    ///     .await
    ///     .unwrap();
    ///
    /// # cache.write(b"MY_KEY", b"Hello World").await.unwrap();
    /// let metadata = cache.remove(b"MY_KEY").await.unwrap();
    /// assert_eq!(metadata.get_size(), b"Hello World".len() as u64);
    /// # }
    /// ```
    pub async fn remove<K: AsRef<[u8]>>(&self, key: K) -> Result<Metadata> {
        let key = key.as_ref();

        let cur_path = self.path_from_key(key);
        let tmp_path = crate::tmp::tmppath_in(&self.opts.path);

        // move then delete the file
        //
        // the purpose of moving then deleting is that file moves are much faster than file
        // deletes. if we were to delete in place, and another thread starts reading, it could
        // spell bad news.
        afs::rename(&cur_path, &tmp_path)
            .await
            .map_err(|e| match e.kind() {
                io::ErrorKind::NotFound => ForcepError::NotFound,
                _ => ForcepError::Io(e),
            })?;
        afs::remove_file(&tmp_path).await.map_err(ForcepError::Io)?;

        // remove the metadata for the entry
        self.meta.remove_metadata_for(key)
    }

    /// Queries the index database for metadata on the entry with the corresponding key.
    ///
    /// This will return the metadata for the associated key. For information about what metadata
    /// is stored, look at [`Metadata`].
    ///
    /// # Non-Async
    ///
    /// Note that this function is not an async call. This is because the backend database used,
    /// `sled`, is not async-compatible. However, these calls are instead very fast.
    ///
    /// # Not Found
    ///
    /// If the entry is not found, then it will return
    /// `Err(`[`Error::NotFound`](ForcepError::NotFound)`)`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() {
    /// use forceps::Cache;
    ///
    /// let cache = Cache::new("./cache")
    ///     .build()
    ///     .await
    ///     .unwrap();
    ///
    /// # cache.write(b"MY_KEY", b"Hello World").await.unwrap();
    /// let meta = cache.read_metadata(b"MY_KEY").unwrap();
    /// assert_eq!(meta.get_size(), b"Hello World".len() as u64);
    /// # }
    /// ```
    #[inline]
    pub fn read_metadata<K: AsRef<[u8]>>(&self, key: K) -> Result<Metadata> {
        self.meta.get_metadata(key.as_ref())
    }

    /// An iterator over the entire metadata database, which provides metadata for every entry.
    ///
    /// This iterator provides every key in the database and the associated metadata for that key.
    /// This is *not* an iterator over the actual values of the database.
    ///
    /// # Non-Async
    ///
    /// Note that this function is not an async call. This is because the backend database used,
    /// `sled`, is not async-compatible. However, these calls are instead very fast.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() {
    /// use forceps::Cache;
    ///
    /// let cache = Cache::new("./cache")
    ///     .build()
    ///     .await
    ///     .unwrap();
    ///
    /// # cache.write(b"MY_KEY", b"Hello World").await.unwrap();
    /// for result in cache.metadata_iter() {
    ///     let (key, meta) = result.unwrap();
    ///     println!("{}", String::from_utf8_lossy(&key))
    /// }
    /// # }
    /// ```
    #[inline]
    pub fn metadata_iter(&self) -> impl Iterator<Item = Result<(Vec<u8>, Metadata)>> {
        self.meta.metadata_iter()
    }

    /// Runs the specified eviction algorithm over this instance cache instance.
    ///
    /// Eviction algorithms will remove items out of the cache until certain a condition has been
    /// met, usually a size requirement. See the [`evictors`] module for more information and
    /// examples.
    ///
    /// [`evictors`]: crate::evictors
    #[inline]
    pub async fn evict_with<E>(&self, evictor: E) -> result::Result<(), E::Err>
    where
        E: crate::evictors::Evictor,
    {
        evictor.evict(self).await
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::CacheBuilder;

    async fn default_cache() -> Cache {
        CacheBuilder::default().build().await.unwrap()
    }

    #[tokio::test]
    async fn short_path() {
        let cache = default_cache().await;
        cache.path_from_key(&[0xAA]);
        cache.path_from_key(&[0xAA, 0xBB]);
        cache.path_from_key(&[0xAA, 0xBB, 0xCC]);
    }

    #[tokio::test]
    async fn write_read_remove() {
        let cache = default_cache().await;

        cache.write(&b"CACHE_KEY", &b"Hello World").await.unwrap();
        let data = cache.read(&b"CACHE_KEY").await.unwrap();
        assert_eq!(data.as_ref(), b"Hello World");
        cache.remove(&b"CACHE_KEY").await.unwrap();
    }

    #[tokio::test]
    async fn tracking_test() {
        let cache = CacheBuilder::default()
            .track_access(true)
            .build()
            .await
            .unwrap();

        cache.write(b"CACHE_KEY", b"Hello World").await.unwrap();
        for _ in 0..100 {
            cache.read(b"CACHE_KEY").await.unwrap();
        }
        assert_eq!(cache.read_metadata(b"CACHE_KEY").unwrap().get_hits(), 100);
    }

    #[tokio::test]
    async fn read_metadata() {
        let cache = default_cache().await;

        cache.write(&b"CACHE_KEY", &b"Hello World").await.unwrap();
        let metadata = cache.read_metadata(&b"CACHE_KEY").unwrap();
        assert_eq!(metadata.get_size(), b"Hello World".len() as u64);
    }
}
