use std::sync::TryLockResult;
use actix_web::{web::{Bytes, self}, error::PayloadError};
use awc::{Client};
use cached::Cached;

use ::futures::Stream;

use log::{error, info, warn};

use crate::{AppState, retry};


pub async fn get_info(id: i32, data: &web::Data<AppState>) -> Option<Bytes> {
    let mut cache = match data.cache.try_lock() {
        Ok(_c) => {_c}
        Err(_) => {return None;}
    };
    match cache.cache_get(&id) {
        Some(c) => {
            Some(c.clone())
        }
        None => {
            drop(cache);
            warn!("ram cache miss on {}",&id);
            let req_builder = || {
                data.client.get(format!("https://www.pixiv.net/ajax/illust/{}", &id))
            };
            let rsp = retry!(req_builder,3);
            return match rsp {
                Ok(mut i) => {
                    let img_content = i.body().await.ok()?;

                    match data.cache.try_lock() {
                        Ok(mut _c) => {
                            _c.cache_set(id, img_content.clone());
                            Some(img_content)
                        }
                        Err(_) => {
                            Some(img_content)
                        }
                    }
                }
                Err(e) => {
                    error!("{:?} when download {}",&e,&id);
                    None
                }
            };
        }
    }
}


pub async fn download_file(url: &str, client: &Client) -> Option<impl Stream<Item=Result<actix_web::web::Bytes, PayloadError>>> {
    info!("download from {}",url);
    let req_builder = || {
        client.get(url)
    };
    match retry!(req_builder,3) {
        Ok(i) => {
            Some(i)
        }
        Err(e) => {
            warn!("download error on {} {:?}",url,&e);
            None
        }
    }
}