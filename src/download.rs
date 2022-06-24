use actix_web::{http::header::{REFERER, ContentType, USER_AGENT,CACHE_CONTROL}, HttpResponse, Responder, web::Bytes};
use awc::error;
use log::{error, info};
use tokio::task::futures;



pub async fn get_info(id:i32)->Option<Bytes>{
    let client = awc::Client::default();
    let  rsp =  client.get(format!("https://www.pixiv.net/ajax/illust/{}", id))
        .append_header((USER_AGENT, "PixivAndroidApp/5.0.115 (Android 6.0; PixivBot)"))
        .append_header((REFERER, "https://www.pixiv.net"))
        .send().await;
    match rsp {
        Ok(mut i)=>{
            
            return Some(i.body().await.ok()?)
        }
        Err(e) =>{
            error!("{:?}",&e);
             return None
        }
    };
}

pub async fn stream_file(url:&str)->HttpResponse{
        let client = awc::Client::default();
        info!("{}",url);
        let rsp = client.get(url).append_header((REFERER, "https://www.pixiv.net")) .send().await;
        match rsp {
            Ok(i)=>{
                HttpResponse::Ok().content_type(ContentType::jpeg())
                .append_header((CACHE_CONTROL,"max-age=999999"))
                .streaming(i)
            }
            Err(e) =>{
                error!("{:?}",&e);
               HttpResponse::NotFound().finish()
            }
        }
}