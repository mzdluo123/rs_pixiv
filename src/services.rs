use crate::AppState;
use crate::download::{get_info, stream_file};
use crate::json_struct::Root;
use crate::tmplate::IndexTemp;
use actix_web::dev::Payload;
use actix_web::http;
use actix_web::{get, http::header::{ContentType,LAST_MODIFIED}, web, App, HttpResponse, HttpRequest, Responder};
use askama::Template;
use awc::http::header::{REFERER, USER_AGENT};
use awc::Client;
use cached::Cached;
use log::error;
use serde::__private::de::Content;


// static allowsType: [&str; 5] = ["mini","original","regular","small","thumb"];

#[get("/")]
pub async fn index(data: web::Data<AppState>) -> impl Responder {
    let cache = data.cache.lock().unwrap();
    let rsp = IndexTemp {
        cache_count:cache.cache_size()
    };
    return match rsp.render() {
        Ok(rsp) => HttpResponse::Ok()
            .content_type(ContentType::html())
            .body(rsp),
        Err(_) => HttpResponse::InternalServerError().finish(),
    };
}

#[get("/json/{id}")]
pub async fn json_img(id: web::Path<i32>,data: web::Data<AppState>) -> impl Responder {
            let content = get_info(id.into_inner(),&data).await;
            match content {
                Some(i) => {

                    HttpResponse::Ok().content_type(ContentType::json()).body(i)
                },
                None => HttpResponse::NotFound().finish(),
            }
    }
 


#[get("/img/{img_type}/{id}")]
pub async fn web_img(info: web::Path<(String,i32)>,data: web::Data<AppState>,req : HttpRequest) ->  impl Responder {
    if  req.headers().contains_key("if-modified-since") {
       return HttpResponse::NotModified().finish();
    }
    // if !allowsType.contains(&img_type.as_str()){
    //     return HttpResponse::NotFound().finish();
    // }

    let content = get_info(info.1,&data).await;
    match content {
        Some(i) => {
            let obj: Root = serde_json::from_str(&String::from_utf8(i.to_vec()).unwrap()).unwrap();
            let url = match info.0.as_str() {
                "mini" => {
                    obj.body.urls.mini
                }
                "original" =>{
                    obj.body.urls.original
                }
                "regular"=>{
                    obj.body.urls.regular
                }
                "small" =>{
                    obj.body.urls.small
                }
                "thumb" =>{
                    obj.body.urls.thumb
                }
                _=>{
                    error!("img_type error {}",info.0);
                    return  HttpResponse::NotFound().finish();
                }
            };
            stream_file(&url,&data.client).await
        }
     None=>{
            HttpResponse::NotFound().finish()
        }
     }  
 }

