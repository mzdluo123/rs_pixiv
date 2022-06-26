use actix_web::web::Bytes;
use cached::async_sync::Mutex;
use core::time;
use env_logger::fmt::Timestamp;
use futures::{Stream, StreamExt};
use log::{error, info, warn};
use once_cell::sync::OnceCell;
use std::collections::BinaryHeap;
use std::{
    cmp::Ordering,
    collections::HashMap,
    f32::DIGITS,
    fmt::Display,
    fs,
    hash::Hash,
    os::windows::prelude::MetadataExt,
    rc::Rc,
    sync::{Arc, RwLock},
    time::{Duration, Instant, SystemTime},
};
use tokio::io::AsyncWriteExt;

// #[derive(Debug,PartialEq, Eq)]
// pub struct FsCacheMetaData{
//     path:String,
//     last_access:u64
// }

// impl Ord for FsCacheMetaData {
//     fn cmp(&self, other: &Self) -> std::cmp::Ordering {
//         if self.last_access < other.last_access {
//             return Ordering::Less;
//         }
//         if self.last_access > other.last_access {
//             return Ordering::Greater;
//         }
//         return Ordering::Equal;
//     }
// }

// impl PartialOrd for FsCacheMetaData{
//     fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
//         match self.path.partial_cmp(&other.path) {
//             Some(core::cmp::Ordering::Equal) => {}
//             ord => return ord,
//         }
//         self.last_access.partial_cmp(&other.last_access)
//     }
// }

pub struct FsCache {
    // metadata: RwLock<BinaryHeap<Arc<FsCacheMetaData>>>,
    // index: RwLock<HashMap<String,Arc<FsCacheMetaData>>>,
    cache_folder: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum FsCacheError {
    InsertMataError,
    WriteFileError,
    CleanError,
}

impl FsCache {
     pub fn new(cache_folder:&str) -> FsCache {
        return FsCache {
            // metadata: RwLock::new(BinaryHeap::new()),
            // index:RwLock::new(HashMap::new()),
            cache_folder: cache_folder.to_string()
        };
    }

    // async fn read_meta(self:&Self, key:&str)->Option<Arc<FsCacheMetaData>>{
    //     let read_able_idnex = self.index.read().ok()?;
    //     if !read_able_idnex.contains_key(key) {
    //         return None;
    //     }
    //     return Some(read_able_idnex[key].clone());
    // }

    // async fn write_meta(self:&mut Self,key:&str, path:&str)->Result<Arc<FsCacheMetaData>,FsCacheError>{

    //      self.index.read().and_then(|i|{
    //         i.contains_key(key){
    //             return Ok(i[key].clone());
    //         }
    //     }).map_err(|e|{
    //         return FsCacheError::InsertMataError;
    //     });

    //     let meta = tokio::fs::metadata(path).await;
    //     match meta {
    //         Ok(i)=>{
    //             let fs_meta =Arc::new(FsCacheMetaData{
    //                 last_access : i.last_access_time(),
    //                 path: path.to_string()
    //             });
    //             self.metadata.get_mut().and_then(|x|Ok( {
    //                 x.push(fs_meta.clone());
    //             }));
    //             self.index.get_mut().and_then(|x| Ok({
    //                 x.insert(key.to_string(), fs_meta);
    //             }));
    //             Ok(())
    //         }
    //         Err(e)=>{
    //             error!("insert meta error {}",e);
    //             Err(FsCacheError::InsertMataError)
    //         }
    //     }
    // }

    // async fn remove_meta(self:&mut Self,key:&str){
    //     self.index.get_mut().and_then(|i|{
    //         if i.contains_key(key){
    //             i.remove(key);
    //             self.metadata.get_mut().and_then(|x|{
    //                 x.
    //             })
    //         }
    //     })
    // }

    pub async fn read(self: &Self, key: &str) -> Option<Bytes> {
        let path = format!("{},{}", self.cache_folder, key);
        let content = tokio::fs::read(&path).await;
        match content {
            Ok(v) => {
                return Some(Bytes::from(v));
            }
            Err(e) => {
                warn!("read cached file error {:?}", path);
                return None;
            }
        }

        // match self.read_meta(key).await {
        //     Some(i)=>{

        //     }
        //     None=>None
        // }
    }

    pub async fn write_cache<T, E>(
        self: &Self,
        key: &str,
        stream: &mut T,
    ) -> Result<(), FsCacheError>
    where
        T: Stream<Item = Result<Bytes, E>> + Unpin,
        E: Display,
    {
        // let read_able_index = self.index
        // .read().;
        let path = format!("{},{}", self.cache_folder, key);

        let mut file = tokio::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(path)
            .await
            .map_err(|e| {
                error!("create cache file error {:?}", e);
                FsCacheError::WriteFileError
            })?;

        while let Some(v) = stream.next().await {
            match v {
                Ok(c) => {
                    file.write_all(&c).await.unwrap();
                }
                Err(e) => {
                    error!("write cache file error {}", e);
                }
            }
        }
        Ok(())
        // Some(self.write_meta(key,&path));
    }

   
}



pub async fn clean(folder: &str) -> Result<(), FsCacheError> {
    let mut dir = tokio::fs::read_dir(folder)
        .await
        .map_err(|e| FsCacheError::CleanError)?;
    let now = SystemTime::now().elapsed().unwrap();
    while let Ok(Some(d)) = dir.next_entry().await {
        let meta = d.metadata().await;
        match meta {
            Ok(m) => {
                if m.last_access_time() - now.as_secs() >= 1000 * 60 * 60 {
                    //一小时
                    tokio::fs::remove_file(d.path()).await;
                }
            }
            Err(e) => {
                return Ok(());
            }
        }
    }
    info!("clean finished");
    Ok(())
}

pub async fn clean_task(cache: String) {
    
    loop {
        info!("run clean");
        clean(&cache).await;
        tokio::time::sleep(Duration::from_secs(60 * 60)).await;
    }
}
