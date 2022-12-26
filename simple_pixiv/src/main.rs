mod tmplate;
mod services;
mod download;
mod ill_struct;
mod bookmark_struct;
mod random_img;
mod retry;

use log::{error, info, warn};
use actix_web::{ web::{self, Bytes}, App, HttpServer};
use random_img::refresh_random;

use std::{env, sync::{Mutex, Arc, RwLock},path::Path};
use std::time::Duration;
use awc::Connector;
use awc::http::header::{REFERER, USER_AGENT};

use crate::random_img::ImgIdStorage;


pub struct AppState {
    client:awc::Client,
    cache: Arc<Mutex<cached::TimedSizedCache<i32,Bytes>>>,
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
    actix_web::rt::spawn(refresh_random(storage, uid, cookie));

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

    info!("Run server on port {}",port);

    let random_img = Arc::new(RwLock::new(ImgIdStorage::new()));

    start_bookmark_task(random_img.clone()).await;

    let client_builder = || {awc::ClientBuilder::new()
        .add_default_header((REFERER, "https://www.pixiv.net"))
        .add_default_header(("App-OS-Version","15.5"))
        .add_default_header(("App-Version","7.14.8"))
        .add_default_header((USER_AGENT,"PixivIOSApp/7.14.8 (iOS 15.5; iPhone14,5)"))
        .connector(
            Connector::new()
                .conn_lifetime(Duration::from_secs(30))
        )
        .finish()
    };

    let cache = Arc::new(Mutex::new(cached::TimedSizedCache::with_size_and_lifespan(5000, 60*60*2)));
    let cache_refresh_task =   |c:Arc<Mutex<cached::TimedSizedCache<i32,Bytes>>>| async move {
        loop {
            info!("refresh cache");
            match c.clone().try_lock() {

                Ok(_c) =>{
                    _c.refresh();
                }
                Err(_) => {
                    error!("refresh cache error");
                }
            }
            tokio::time::sleep(Duration::from_secs(60 * 60)).await;
        }
    };

    actix_web::rt::spawn(
      cache_refresh_task(cache.clone())
    );

    HttpServer::new(move || {
        App::new()
        .app_data(web::Data::new(
            AppState{
               client: client_builder(),
                cache: cache.clone(),
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

