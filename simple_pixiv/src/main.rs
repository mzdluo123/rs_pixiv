mod tmplate;
mod services;
mod download;
mod json_struct;

use log::{ info, error};
use actix_web::{ web::{self, Bytes}, App, HttpServer};
use core::time;
use std::{env, sync::{Mutex, Arc}};
use forceps::evictors::LruEvictor;


pub struct AppState {
    client:awc::Client,
    cache: Mutex<cached::TimedSizedCache<i32,Bytes>>,
    file_cache: Arc<forceps::Cache>
}


#[tokio::main] // or #[tokio::main]
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
    let file_cache = Arc::new(forceps::CacheBuilder::new(file_cache_folder).track_access(true).build().await.unwrap());
    tokio::spawn(evict_task(file_cache.clone()));
    info!("Run server on port {}",port);
    HttpServer::new(move || {
        App::new()
        .app_data(web::Data::new(
            AppState{
               client: awc::Client::default(),
               cache: Mutex::new(cached::TimedSizedCache::with_size_and_lifespan(1000, 60*60)),
               file_cache:file_cache.clone()
            }
        ))
            .wrap(actix_web::middleware::Logger::default().log_target("http_log"))
            .wrap(actix_web::middleware::ErrorHandlers::default())

            .service(services::index)
            .service(services::json_img)
            .service(services::web_img)
            

    })
        .bind(("0.0.0.0", port))?
        // .bind(("::",port))?
        
        .run()
        .await
}


async fn evict_task(file_cache: Arc<forceps::Cache>){
    let max_cache_size: usize = match env::var("MAX_CACHE_SIZE") {
        Ok(p) => {
            p.parse().unwrap()
        }
        Err(_) => {
            100*1024*1024 // 100MB
        }
    };

    loop {
        file_cache.evict_with(LruEvictor::new(max_cache_size.try_into().unwrap())).await.map_err(|e|{
            error!("evict cache error {:?}",e);
        }).ok();
        info!("evict cache success");
        tokio::time::sleep(time::Duration::from_secs(60*60)).await;
    }
}