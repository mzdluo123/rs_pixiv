use crate::download::{download_file, get_info};
use crate::ill_struct::Root;
use crate::tmplate::IndexTemp;
use crate::{fs_cache, AppState};
use actix_web::http;
use actix_web::http::header::CACHE_CONTROL;
use actix_web::{
    get,
    http::header::{ContentType, LAST_MODIFIED,USER_AGENT,COOKIE,REFERER},
    web, App, HttpRequest, HttpResponse, Responder,
};
use askama::Template;
use askama::filters::format;
use cached::Cached;
use log::{error, info, warn};

// static allowsType: [&str; 5] = ["mini","original","regular","small","thumb"];

#[get("/")]
pub async fn index(data: web::Data<AppState>) -> impl Responder {
    let cache = data.cache.lock().unwrap();

    let rsp = IndexTemp {
        meta_cache: cache.cache_size(),
        bookmark: data.random_image.read().unwrap().id_set.len(),
    };
    return match rsp.render() {
        Ok(rsp) => HttpResponse::Ok()
            .content_type(ContentType::html())
            .body(rsp),
        Err(_) => HttpResponse::InternalServerError().finish(),
    };
}

#[get("/json/{id}")]
pub async fn json_img(id: web::Path<i32>, data: web::Data<AppState>) -> impl Responder {
    let content = get_info(id.into_inner(), &data).await;
    match content {
        Some(i) => HttpResponse::Ok().content_type(ContentType::json()).body(i),
        None => HttpResponse::NotFound().finish(),
    }
}

#[get("/img/{img_type}/{id}")]
pub async fn web_img(
    info: web::Path<(String, i32)>,
    data: web::Data<AppState>,
    req: HttpRequest,
) -> impl Responder {
    if req.headers().contains_key("if-modified-since") {
        return HttpResponse::NotModified().finish();
    }
    // if !allowsType.contains(&img_type.as_str()){
    //     return HttpResponse::NotFound().finish();
    // }
    let cache_key = format!("{}{}", info.0, info.1);
    let ref fs_cache = data.fs_cache;
    match fs_cache.read(&cache_key).await {
        Some(i) => {
            return HttpResponse::Ok()
                .content_type(ContentType::jpeg())
                .append_header((CACHE_CONTROL, "max-age=31536000"))
                .append_header((LAST_MODIFIED, "1"))
                .body(i);
        }
        None => {
            warn!("disk cache miss on {}", cache_key)
        }
    }

    let content = get_info(info.1, &data).await;
    match content {
        Some(i) => {
            let obj: Root = serde_json::from_str(&String::from_utf8(i.to_vec()).unwrap()).unwrap();
            let url = match info.0.as_str() {
                "mini" => obj.body.urls.mini,
                "original" => obj.body.urls.original,
                "regular" => obj.body.urls.regular,
                "small" => obj.body.urls.small,
                "thumb" => obj.body.urls.thumb,
                _ => {
                    error!("img_type error {}", info.0);
                    return HttpResponse::NotFound().finish();
                }
            };
            match download_file(&url, &data.client).await {
                Some(mut i) => {
                    fs_cache.write_cache(&cache_key, &mut i).await.unwrap();

                    let rsp = fs_cache.read_stream(&req, &cache_key).await;
                    match rsp {
                        Some(_i) => {
                            return _i;
                        }
                        None => HttpResponse::NotFound().finish(),
                    }
                }
                None => HttpResponse::NotFound().finish(),
            }
        }
        None => HttpResponse::NotFound().finish(),
    }
}

#[get("/random")]
pub async fn random(data: web::Data<AppState>) -> impl Responder {
    if let Ok(random) = data.random_image.read() {
        if let Some(id) = random.random_img() {
            return HttpResponse::TemporaryRedirect()
                .append_header((http::header::LOCATION, format!("/img/small/{id}")))
                .finish();
        }
        return HttpResponse::NotFound().finish();
    }
    return HttpResponse::NotFound().finish();
}

#[get("/{path:(img-(master|original)|c).*}")]
pub async fn pximg_proxy(
    parm: web::Path<String>,
    data: web::Data<AppState>,
    req: HttpRequest,
) -> impl Responder {
    let url = format!("https://i.pximg.net/{}",parm.as_ref());
    let client = &data.client;
    let mut req_builder = client.get(&url)
    .append_header((REFERER, "https://www.pixiv.net"));

    if let Some(ua) =  req.headers().get(USER_AGENT){
        req_builder = req_builder.append_header((USER_AGENT,ua));
    }
    
    if let Some(cookie) =  req.headers().get(COOKIE){
        req_builder = req_builder.append_header((COOKIE,cookie));
    }

    match req_builder.send().await {
        Ok(i)=>{
            return HttpResponse::Ok().content_type(ContentType::jpeg()).streaming(i);
        }
        Err(e) =>{
            warn!("download error on {} {:?}",&url,&e);
            return HttpResponse::NotFound().finish();
        }
    }
   

}
