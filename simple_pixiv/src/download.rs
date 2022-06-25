use actix_web::{http::{header::{REFERER, ContentType, USER_AGENT,CACHE_CONTROL,LAST_MODIFIED}, Error}, HttpResponse, web::{Bytes, self}, HttpMessage};
use awc::{Client};
use cached::Cached;

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
            warn!("ram cache miss on {}",id);
            let  rsp =  data.client.get(format!("https://www.pixiv.net/ajax/illust/{}", id))
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
                 error!("{:?}",&e);
                 return None
            }
        };
        },
    }
  
}

pub async fn download_file(url:&str,client: &Client)->Option<Bytes>{
        info!("download from {}",url);
        let rsp = client.get(url).append_header((REFERER, "https://www.pixiv.net")) .send().await;
        match rsp {
            Ok(mut i)=>{
                 Some(i.body().limit(10*1024*1024).await.ok()?)
            }
            Err(e) =>{
                warn!("download errpr on {} {:?}",url,&e);
               None
            }
        }
}