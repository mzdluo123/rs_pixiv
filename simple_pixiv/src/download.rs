use actix_web::{http::{header::{REFERER, ContentType, USER_AGENT,CACHE_CONTROL,LAST_MODIFIED}, Error}, HttpResponse, web::{Bytes, self}, HttpMessage, error::PayloadError, dev::Payload};
use awc::{Client, ClientResponse};
use cached::Cached;

use ::futures::Stream;
use log::{error, info, warn};
use tokio::sync::futures;
use crate::AppState;



pub async fn get_info(id:i32,data: &web::Data<AppState>)->Option<Bytes>{
    let mut cache = data.cache.lock().unwrap();
    match cache.cache_get(&id) {
        Some(c) =>{
            return Some(c.clone());
        },
        None => {
            drop(cache);
            warn!("ram cache miss on {}",&id);
            let  rsp =  data.client.get(format!("https://www.pixiv.net/ajax/illust/{}", &id))
            .append_header((USER_AGENT, "PixivAndroidApp/5.0.115 (Android 6.0; PixivBot)"))
            .append_header((REFERER, "https://www.pixiv.net"))
            .send().await;
        match rsp {
            Ok(mut i)=>{
                let img_contant = i.body().await.ok()?;
                cache = data.cache.lock().unwrap();
                cache.cache_set(id, img_contant.clone());
                return Some(img_contant)
            }
            Err(e) =>{
                 error!("{:?} when download {}",&e,&id);
                 return None
            }
        };
        },
    }
  
}


pub async fn download_file(url:&str,client: &Client)->Option<impl Stream<Item = Result<actix_web::web::Bytes, PayloadError>>>{
    
        info!("download from {}",url);
        let rsp = client.get(url).append_header((REFERER, "https://www.pixiv.net")) .send().await;
        match rsp {
            Ok(i)=>{
                 Some(i)
            }
            Err(e) =>{
                warn!("download error on {} {:?}",url,&e);
               None
            }
        }
}