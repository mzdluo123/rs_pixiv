use actix_web::{http::{header::{REFERER, USER_AGENT}}, web::{Bytes, self}, error::PayloadError};
use awc::{Client};
use cached::Cached;

use ::futures::Stream;

use log::{error, info, warn};

use crate::{AppState, retry};



pub async fn get_info(id:i32,data: &web::Data<AppState>)->Option<Bytes>{
    let mut cache = data.cache.lock().unwrap();
    match cache.cache_get(&id) {
        Some(c) =>{
            Some(c.clone())
        },
        None => {
            drop(cache);
            warn!("ram cache miss on {}",&id);
            let req_builder = ||{
                data.client.get(format!("https://www.pixiv.net/ajax/illust/{}", &id))
                    .append_header((REFERER, "https://www.pixiv.net"))
                    .append_header(("App-OS-Version","15.5"))
                    .append_header(("App-Version","7.14.8"))
                    .append_header((USER_AGENT,"PixivIOSApp/7.14.8 (iOS 15.5; iPhone14,5)"))
            };
            let rsp = retry!(req_builder,3);
        return match rsp {
            Ok(mut i) => {
                let img_contant = i.body().await.ok()?;
                cache = data.cache.lock().unwrap();
                cache.cache_set(id, img_contant.clone());
                Some(img_contant)
            }
            Err(e) => {
                error!("{:?} when download {}",&e,&id);
                None
            }
            };
        },
    }
  
}


pub async fn download_file(url:&str,client: &Client)->Option<impl Stream<Item = Result<actix_web::web::Bytes, PayloadError>>>{
    
        info!("download from {}",url);
        let req_builder = ||{
            client.get(url)
                .append_header((REFERER, "https://www.pixiv.net"))
                .append_header(("App-OS-Version","15.5"))
                .append_header(("App-Version","7.14.8"))
                .append_header((USER_AGENT,"PixivIOSApp/7.14.8 (iOS 15.5; iPhone14,5)"))
        };
        match retry!(req_builder,3) {
            Ok(i)=>{
                 Some(i)
            }
            Err(e) =>{
                warn!("download error on {} {:?}",url,&e);
               None
            }
        }
}