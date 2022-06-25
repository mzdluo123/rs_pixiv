use crate::{Cache, ForcepError, Metadata};
use async_trait::async_trait;
use std::cmp;

/// A trait that represents a structure or enum that can evict items out of a [`Cache`] instance.
///
/// This is very generic, and it is up to the implementation for the order they would like to evict
/// items, and the total number of items they would like to evict. The inner `evict` function is
/// the function called to start the eviction process.
///
/// # Implementations
///
/// This can be implemented by the user of this library, however there are also ready-made
/// implementations:
///
/// * [`LruEvictor`] - Least Recently Used eviction impl
/// * [`FifoEvictor`] - First-in First-out eviction impl
///
/// # Examples
///
/// Basic implementation that will remove all items in the order they're found in the
/// meta-database:
/// ```rust,no_run
/// use forceps::{Cache, evictors::Evictor};
/// struct MyEvictor;
///
/// #[async_trait::async_trait]
/// impl Evictor for MyEvictor {
///     type Err = Box<dyn std::error::Error>;
///
///     async fn evict(&self, cache: &Cache) -> Result<(), Self::Err> {
///         for result in cache.metadata_iter() {
///             let (key, meta) = result?;
///             cache.remove(&key).await?;
///         }
///
///         Ok(())
///     }
/// }
/// ```
#[async_trait]
pub trait Evictor {
    /// The error type that the `evict` method will produce.
    type Err;

    /// The method that is called to evict old or outdated items from the [`Cache`] database.
    ///
    /// See [`Evictor`] trait for more information/explanation.
    async fn evict(&self, cache: &Cache) -> Result<(), Self::Err>;
}

/// A trait for evictors that will evict items until a minimum size is met
///
/// This trait default implements `evict_to_min_size`, and requires `batch_size` and `min_size`.
#[async_trait]
trait MinSzEvictor {
    type Candidate: EvictCandidate + Send + Sync;

    /// Getter for the batch size of the eviction. Recommended to use `#[inline]`
    ///
    /// The batch size indicates how many eviction candidates should be found for each eviction
    /// loop. Bigger values help bigger evictions.
    fn batch_size(&self) -> usize;
    /// The minimum size (in bytes) to evict to.
    fn min_size(&self) -> u64;

    /// Main eviction algorithm to evict items in a [`Cache`] until a minimum size is met
    ///
    /// Configuration is done via the `batch_size` and `min_size` function implementions
    async fn evict_to_min_size(&self, cache: &Cache) -> Result<(), ForcepError> {
        // run the evictor in a loop so if it runs out of candidates, it can just jump back and
        // look for a new set of candidates
        'evictor: loop {
            let (mut total_size, scan) =
                find_evict_candidates::<Self::Candidate>(cache, self.batch_size())?;
            // break if there are no candidates (cache is completely empty)
            if scan.len() <= 0 {
                break;
            }

            // loop through all candidates and remove them one-by-one until it meets size
            // requirement
            // TODO: maybe in the future this can batched into FuturesUnordered?
            for e in &scan {
                if total_size <= self.min_size() {
                    break 'evictor;
                }

                // this almost certainly won't fail, and if it does we should treat it as fatal
                // (pushed up the stack)
                let meta = cache.remove(e.key()).await?;
                total_size -= meta.get_size();
            }
        }
        Ok(())
    }
}

/// A trait that represents a candidate for eviction. Used as a generalization for
/// [`find_evict_candidates`]
///
/// A structure that implements this trait can be created using
/// [`from_meta`](EvictCandidate::from_meta). To check whether it should be evicted over another
/// [`EvictCandidate`], [`should_evict_over`](EvictCandidate::should_evict_over) can be used.
trait EvictCandidate {
    /// Creates `Self` based on the metadata entry.
    fn from_meta(key: Vec<u8>, meta: Metadata) -> Self;
    /// The key that represents the cache entry.
    fn key(&self) -> &[u8];
    /// An ordering that represents whether this candidate should be evicted *before* the `other`
    /// candidate.
    ///
    /// [`cmp::Ordering::Greater`] represents that this candidate should be evicted over `other`.
    fn should_evict_over(&self, other: &Self) -> cmp::Ordering;
}

/// Finds the total size of the cache and a vector of eviction candidates.
///
/// The eviction candidates is an in-order list of candidates that should be removed from the
/// cache. These are selected based on the priority to evict, which is determined by
/// [`EvictCandidate::should_evict_over`].
///
/// This function runs at approx. `O(n * batch)` where `n` is the number of metadata entries, and
/// `batch` is the variable provided.
fn find_evict_candidates<E: EvictCandidate>(
    cache: &Cache,
    batch: usize,
) -> Result<(u64, Vec<E>), ForcepError> {
    let mut total_sz = 0;
    let mut entries = Vec::with_capacity(batch);

    for result in cache.metadata_iter() {
        // if any weird errors happen with the iter, then we should treat them as fatal and just
        // push them up the stack
        let (key, meta) = result?;

        // increment the total size counter
        total_sz += meta.get_size();

        // if the `entries` vector isn't filled, just insert the entry
        let entry = E::from_meta(key, meta);
        if entries.len() < batch {
            entries.push(entry);
            continue;
        }

        // `entries` vector is completely filled, so swap the values if this current metadata entry
        // should be evicted sooner than the one stored
        //
        // this slowly weeds out all entries of metadata until there are only the ones left that
        // should be evicted first
        if let Some((index, _)) = entries
            .iter()
            .enumerate()
            .find(|(_, e)| entry.should_evict_over(e) == cmp::Ordering::Greater)
        {
            entries[index] = entry;
        }
    }

    // sort the entries so they're in order by how they should be evicted
    entries.sort_unstable_by(|a, b| b.should_evict_over(a));
    Ok((total_sz, entries))
}

/// [`EvictCandidate`] implementation for the Least Recently Used eviction algorithm.
#[derive(Debug)]
struct LruEc {
    key: Vec<u8>,
    last_access: u64,
}
impl EvictCandidate for LruEc {
    #[inline]
    fn from_meta(key: Vec<u8>, meta: Metadata) -> Self {
        Self {
            key,
            last_access: meta.get_last_accessed_raw(),
        }
    }
    #[inline]
    fn key(&self) -> &[u8] {
        &self.key
    }
    #[inline]
    fn should_evict_over(&self, other: &Self) -> cmp::Ordering {
        // `other > self` would result in cmp::Ordering::Greater
        other.last_access.cmp(&self.last_access)
    }
}

/// Least Recently Used eviction algorithm for a [`Cache`]
///
/// This algorithm will evict items based on when they were lasted `read` from the [`Cache`]. It
/// will start with the least recent `read` item, going up until a certain size requirement is met.
///
/// ## Important Note
///
/// This algorithm will not work as expected **unless** the `CacheBuilder::track_access` option has
/// been set to `true`. If this is not the case, then this will work exactly like the
/// [`FifoEvictor`] algorithm.
///
/// ## O(?) & Async
///
/// This eviction algorithm has no guarantee on being fast or efficient at all. It is expected that
/// this algorithm is called infrequently, only when absolutely needed.
///
/// This algorithm also contains blocking calls in an `async` context, mainly metadata iterations
/// and lookups. The reason for the `async` context is for the actual removals from cache.
///
/// # Configuration
///
/// **Minimum Size**
///
/// This is the minimum size that the cache must shrink to until the eviction algorithm will stop
/// evicting items.
///
/// **Batch Size**
///
/// To create one batch, an entire iteration over the metadata must be performed. However, smaller
/// values means that there is less checking/sorting for each batch found. Higher values should be
/// used if you're expecting to evict more items at a time.
///
/// # Examples
///
/// ```rust
/// # #[tokio::main(flavor = "current_thread")]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use forceps::{Cache, evictors::LruEvictor};
///
/// let cache = Cache::new("./cache")
///     .build()
///     .await?;
/// const MIN_SIZE: u64 = 512 * 1024 * 1024; // 512MiB
///
/// // Option 1:
/// cache.evict_with(LruEvictor::new(MIN_SIZE).set_batch_size(500)).await?;
///
/// // Option 2:
/// use forceps::evictors::Evictor;
/// LruEvictor::new(MIN_SIZE).set_batch_size(500).evict(&cache).await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct LruEvictor {
    min_sz: u64,
    batch_size: usize,
}

impl LruEvictor {
    /// Creates a new instance of [`LruEvictor`] that will evict items to `min_size` bytes.
    pub fn new(min_size: u64) -> Self {
        Self {
            min_sz: min_size,
            batch_size: 250,
        }
    }
    /// Sets the batch size for the evictor
    ///
    /// Larger numbers generally increase performance on larger evictions, but decrease performance
    /// on smaller evictions. Read [`LruEvictor`] documentation for more information.
    ///
    /// **Default is 250**
    pub fn set_batch_size(mut self, size: usize) -> Self {
        self.batch_size = size;
        self
    }
}
#[async_trait]
impl MinSzEvictor for LruEvictor {
    type Candidate = LruEc;

    #[inline]
    fn min_size(&self) -> u64 {
        self.min_sz
    }
    #[inline]
    fn batch_size(&self) -> usize {
        self.batch_size
    }
}
#[async_trait]
impl Evictor for LruEvictor {
    type Err = ForcepError;
    async fn evict(&self, cache: &Cache) -> Result<(), Self::Err> {
        self.evict_to_min_size(cache).await
    }
}

/// [`EvictCandidate`] implementation for the First-in-first-out eviction algorithm.
#[derive(Debug)]
struct FifoEc {
    key: Vec<u8>,
    last_modified: u64,
}
impl EvictCandidate for FifoEc {
    fn from_meta(key: Vec<u8>, meta: Metadata) -> Self {
        Self {
            key,
            last_modified: meta.get_last_modified_raw(),
        }
    }
    fn key(&self) -> &[u8] {
        &self.key
    }
    fn should_evict_over(&self, other: &Self) -> cmp::Ordering {
        // `other > self` would result in cmp::Ordering::Greater
        other.last_modified.cmp(&self.last_modified)
    }
}

/// First-in-first-out eviction algorithm for a [`Cache`]
///
/// This algorithm will evict items in a [`Cache`] in the order that they were originally written
/// to the cache, so the first entry will be the first removed. It will stop evicting items when a
/// certain minimum total size is met.
///
/// ## O(?) & Async
///
/// This eviction algorithm has no guarantee on being fast or efficient at all. It is expected that
/// this algorithm is called infrequently, only when absolutely needed.
///
/// This algorithm also contains blocking calls in an `async` context, mainly metadata iterations
/// and lookups. The reason for the `async` context is for the actual removals from cache.
///
/// # Configuration
///
/// **Minimum Size**
///
/// This is the minimum size that the cache must shrink to until the eviction algorithm will stop
/// evicting items.
///
/// **Batch Size**
///
/// To create one batch, an entire iteration over the metadata must be performed. However, smaller
/// values means that there is less checking/sorting for each batch found. Higher values should be
/// used if you're expecting to evict more items at a time.
///
/// # Examples
///
/// ```rust
/// # #[tokio::main(flavor = "current_thread")]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use forceps::{Cache, evictors::FifoEvictor};
///
/// let cache = Cache::new("./cache")
///     .build()
///     .await?;
/// const MIN_SIZE: u64 = 512 * 1024 * 1024; // 512MiB
///
/// // Option 1:
/// cache.evict_with(FifoEvictor::new(MIN_SIZE).set_batch_size(500)).await?;
///
/// // Option 2:
/// use forceps::evictors::Evictor;
/// FifoEvictor::new(MIN_SIZE).set_batch_size(500).evict(&cache).await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct FifoEvictor {
    min_sz: u64,
    batch_size: usize,
}

impl FifoEvictor {
    /// Creates a new instance of [`FifoEvictor`] that will evict items to `min_size` bytes.
    pub fn new(min_size: u64) -> Self {
        Self {
            min_sz: min_size,
            batch_size: 250,
        }
    }
    /// Sets the batch size for the evictor
    ///
    /// Larger numbers generally increase performance on larger evictions, but decrease performance
    /// on smaller evictions. Read [`FifoEvictor`] documentation for more information.
    ///
    /// **Default is 250**
    pub fn set_batch_size(mut self, size: usize) -> Self {
        self.batch_size = size;
        self
    }
}
#[async_trait]
impl MinSzEvictor for FifoEvictor {
    type Candidate = FifoEc;

    #[inline]
    fn min_size(&self) -> u64 {
        self.min_sz
    }
    #[inline]
    fn batch_size(&self) -> usize {
        self.batch_size
    }
}
#[async_trait]
impl Evictor for FifoEvictor {
    type Err = ForcepError;
    async fn evict(&self, cache: &Cache) -> Result<(), Self::Err> {
        self.evict_to_min_size(cache).await
    }
}
