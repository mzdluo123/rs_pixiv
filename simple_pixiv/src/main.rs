mod tmplate;
mod services;
mod download;
mod ill_struct;
mod fs_cache;
mod bookmark_struct;
mod random_img;

use fs_cache::FsCache;
use log::{ info, warn};
use actix_web::{ web::{self, Bytes}, App, HttpServer};
use random_img::refresh_random;

use std::{env, sync::{Mutex, Arc, RwLock},path::Path};

use crate::random_img::ImgIdStorage;


pub struct AppState {
    client:awc::Client,
    cache: Mutex<cached::TimedSizedCache<i32,Bytes>>,
    fs_cache: FsCache,
    random_image:Arc<RwLock<ImgIdStorage>>
}


async fn start_bookmark_task(storage:Arc<RwLock<ImgIdStorage>>){
    let uid = match env::var("PIXIV_UID") {
        Ok(_u)=>{
            _u
        }
        Err(_)=>{
            warn!("PIXIV_UID not set");
            return;
        }
    };

    let cookie = match env::var("PIXIV_COOKIE") {
        Ok(_u)=>{
            base64::decode(_u).map(|_x|{
                String::from_utf8(_x).unwrap()
            }).unwrap()
        }
        Err(_)=>{
            warn!("PIXIV_COOKIE not set");
            return;
        }
    };
    actix_web::rt::spawn(refresh_random(storage.clone(), uid, cookie));

}

#[actix_web::main]// or #[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    let port: u16 = match env::var("PORT") {
        Ok(p) => {
            p.parse().unwrap()
        }
        Err(_) => {
            8080
        }
    };
    let file_cache_folder = match env::var("CACHE") {
        Ok(c) =>{
            c
        }
        Err(_)=>{
            "./cache".to_string()
        }
    };

    info!("Run server on port {}",port);
    

    if !Path::new(&file_cache_folder).exists() {
        tokio::fs::create_dir_all(&file_cache_folder).await.unwrap();
    }
    let random_img = Arc::new(RwLock::new(ImgIdStorage::new()));

    start_bookmark_task(random_img.clone()).await;

    tokio::spawn(fs_cache::clean_task(file_cache_folder.clone()));

    HttpServer::new(move || {
        App::new()
        .app_data(web::Data::new(
            AppState{
               client: awc::Client::default(),
               cache: Mutex::new(cached::TimedSizedCache::with_size_and_lifespan(1000, 60*60)),
                fs_cache :  FsCache::new(&file_cache_folder),
                random_image:random_img.clone()
            }
        ))
            .wrap(actix_web::middleware::Logger::default().log_target("http_log"))
            .wrap(actix_web::middleware::ErrorHandlers::default())

            .service(services::index)
            .service(services::json_img)
            .service(services::web_img)
            .service(services::random)
            .service(services::pximg_proxy)
            

    })
        .bind(("0.0.0.0", port))?
        // .bind(("::",port))?
        
        .run()
        .await
}





// async fn evict_task(file_cache: Arc<forceps::Cache>){
//     let max_cache_size: usize = match env::var("MAX_CACHE_SIZE") {
//         Ok(p) => {
//             p.parse().unwrap()
//         }
//         Err(_) => {
//             100*1024*1024 // 100MB
//         }
//     };

//     loop {
//         file_cache.evict_with(LruEvictor::new(max_cache_size.try_into().unwrap())).await.map_err(|e|{
//             error!("evict cache error {:?}",e);
//         }).ok();
//         info!("evict cache success");
//         tokio::time::sleep(time::Duration::from_secs(60*60)).await;
//     }
// }