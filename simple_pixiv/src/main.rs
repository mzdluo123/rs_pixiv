mod tmplate;
mod services;
mod download;
mod json_struct;

use log::{ info};
use actix_web::{ web::{self, Bytes}, App, HttpServer};
use std::{env, sync::{Mutex, Arc}};


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

