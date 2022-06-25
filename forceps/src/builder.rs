use crate::{
    cache::{Cache, Options},
    Result,
};
use std::path;

/// A builder for the [`Cache`] object. Exposes APIs for configuring the initial setup of the
/// database.
///
/// # Examples
///
/// ```rust
/// # #[tokio::main(flavor = "current_thread")]
/// # async fn main() {
/// use forceps::CacheBuilder;
///
/// let cache = CacheBuilder::new("./cache")
///     .dir_depth(3)
///     .read_write_buffer(1024 * 16)
///     .build()
///     .await
///     .unwrap();
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct CacheBuilder {
    opts: Options,
}

impl CacheBuilder {
    /// Creates a new [`CacheBuilder`], which can be used to customize and create a [`Cache`]
    /// instance.
    ///
    /// The `path` supplied is the base directory of the cache instance.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use forceps::CacheBuilder;
    ///
    /// let builder = CacheBuilder::new("./cache");
    /// // Use other methods for configuration
    /// ```
    pub fn new<P: AsRef<path::Path>>(path: P) -> Self {
        let opts = Options {
            path: path.as_ref().to_owned(),
            dir_depth: 2,
            track_access: false,

            // default to no in-mem lru
            lru_size: 0,

            // default buffer sizes to 8kb
            rbuff_sz: 8192,
            wbuff_sz: 8192,
        };
        CacheBuilder { opts }
    }

    /// Sets the depth of directories created in the cache folder.
    ///
    /// **Default is `2`**
    ///
    /// This will set the depth of folders created and expected when reading and writing to the
    /// database. Increasing this value could increase the time to write to the database.
    ///
    /// # Breaking Warning
    ///
    /// Changing this value on a live database without migrations will cause the database `read`
    /// operations to essentially skip over the data. This means that all data will be
    /// inaccessible, despite the metadata being accessible.
    pub fn dir_depth(mut self, depth: u8) -> Self {
        self.opts.dir_depth = depth;
        self
    }

    /// Sets the maximum size (in bytes) for the in-memory Least-Recently-Used cache.
    ///
    /// **Default is `0`**
    ///
    /// Setting this above `0` will create an in-memory store for recently-used entries. This will
    /// use up more RAM, but will also significantly increase speed on memcache `HIT`s.
    pub fn memory_lru_max_size(mut self, size: usize) -> Self {
        self.opts.lru_size = size;
        self
    }

    /// Changes the in-memory buffer sizes for reading and writing `fs` operations.
    ///
    /// **Default is `8kb` (`8196`)**
    ///
    /// Increasing this value may benefit performance as more bulk reading is involved. However,
    /// this option completely depends on the size of the data you are reading/writing.
    pub fn read_write_buffer(mut self, size: usize) -> Self {
        self.opts.rbuff_sz = size;
        self.opts.wbuff_sz = size;
        self
    }

    /// If set to `true`, this will track track the total hits and the last time an entry was
    /// accessed in the metadata.
    ///
    /// **Default is `false`**
    ///
    /// Be warned, turning this on will cause blocking metadata database calls to occur on `read`
    /// operations. This does not normally occur and can cause problems for `async` applications.
    pub fn track_access(mut self, toggle: bool) -> Self {
        self.opts.track_access = toggle;
        self
    }

    /// Builds the new [`Cache`] instance using the configured options of the builder.
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
    pub async fn build(self) -> Result<Cache> {
        Cache::create(self.opts).await
    }
}

impl Default for CacheBuilder {
    /// Creates a [`CacheBuilder`] with the directory set to `./cache`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() {
    /// use forceps::CacheBuilder;
    ///
    /// let cache = CacheBuilder::default()
    ///     .build()
    ///     .await
    ///     .unwrap();
    /// # }
    /// ```
    fn default() -> Self {
        const DIR: &str = "./cache";
        Self::new(DIR)
    }
}
