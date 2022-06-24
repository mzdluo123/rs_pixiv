use crate::download::{get_info, stream_file};
use crate::json_struct::Root;
use crate::tmplate::IndexTemp;
use actix_web::dev::Payload;
use actix_web::{get, http::header::ContentType, web, App, HttpResponse, HttpServer, Responder};
use askama::Template;
use awc::http::header::{REFERER, USER_AGENT};
use awc::Client;
use log::error;
use serde::__private::de::Content;

#[get("/hello/{name}")]
pub async fn greet(name: web::Path<String>) -> impl Responder {
    format!("Hello {name}!")
}

#[get("/")]
pub async fn index() -> impl Responder {
    let rsp = IndexTemp {};
    return match rsp.render() {
        Ok(rsp) => HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(rsp),
        Err(_) => HttpResponse::InternalServerError().finish(),
    };
}

#[get("/json/{id}")]
pub async fn json_img(id: web::Path<i32>) -> impl Responder {
    let content = get_info(id.into_inner()).await;
    match content {
        Some(mut i) => HttpResponse::Ok().content_type(ContentType::json()).body(i),
        None => HttpResponse::NotFound().finish(),
    }
}

#[get("/web/{id}")]
pub async fn web_img(id: web::Path<i32>) ->  impl Responder {
    let content = get_info(id.into_inner()).await;
    match content {
        Some(i) => {
            let obj: Root = serde_json::from_str(&String::from_utf8(i.to_vec()).unwrap()).unwrap();
            let regular = obj.body.urls.regular;
            stream_file(&regular).await
        }
     None=>{
        
            HttpResponse::NotFound().finish()
        }
     }  
 }

