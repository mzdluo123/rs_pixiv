
use actix_web::{
    get,
    http::header::{CONTENT_LENGTH, ContentType, COOKIE},
    HttpRequest, HttpResponse, Responder, web,
};
use actix_web::body::SizedStream;
use actix_web::http;
use actix_web::http::header::LAST_MODIFIED;
use askama::Template;
use cached::Cached;
use log::{error, warn};

use crate::{AppState, retry};
use crate::download::{download_file, get_info};
use crate::ill_struct::{ PageStruct};
use crate::tmplate::IndexTemp;
use crate::utils::make_redirect;

// static allowsType: [&str; 5] = ["mini","original","regular","small","thumb"];

#[get("/")]
pub async fn index(data: web::Data<AppState>) -> impl Responder {
    return match data.cache.try_lock() {
        Ok(_c) => {
            let rsp = IndexTemp {
                meta_cache: _c.cache_size(),
                bookmark: data.random_image.read().unwrap().id_set.len(),
            };
            match rsp.render() {
                Ok(rsp) => HttpResponse::Ok()
                    .content_type(ContentType::html())
                    .body(rsp),
                Err(_) => HttpResponse::InternalServerError().finish(),
            }
        }
        _ => {
            HttpResponse::InternalServerError().finish()
        }
    };
}

#[get("/json/{id}")]
pub async fn json_pages(id: web::Path<i32>, data: web::Data<AppState>) -> impl Responder {
    let content = get_info(id.into_inner(), &data).await;
    match content {
        Some(i) => HttpResponse::Ok().content_type(ContentType::json()).body(i),
        None => HttpResponse::NotFound().finish(),
    }
}

#[get("/img/{id}")]
pub async fn fast_small_img(info: web::Path<String>) -> HttpResponse {
    return HttpResponse::TemporaryRedirect().append_header((http::header::LOCATION, format!("/img/small/{}",info))).finish();
}

#[get("/img/{img_type}/{id}")]
pub async fn web_img(
    info: web::Path<(String, i32)>,
) -> impl Responder {
    let (img_type, id) = info.into_inner();
    return make_redirect(format!("/img/{img_type}/{id}/1"));
}


#[get("/img/{img_type}/{id}/{page}")]
pub async fn web_img_with_page(
    info: web::Path<(String, i32,usize)>,
    data: web::Data<AppState>,
    req: HttpRequest,
) -> impl Responder {
    let (img_type, id,page) = info.into_inner();

    if req.headers().contains_key("if-modified-since") {
        return HttpResponse::NotModified().finish();
    }
    // if !allowsType.contains(&img_type.as_str()){
    //     return HttpResponse::NotFound().finish();
    // }
    // redirect()
    return find_image(id, page, img_type.as_str(), data).await;
}


async fn find_image(img_id:i32, page:usize, img_type:&str, data: web::Data<AppState>,) ->HttpResponse{
    let content = get_info(img_id, &data).await;
    match content {
        Some(i) => {
            let obj: PageStruct = match serde_json::from_str(&String::from_utf8(i.to_vec()).unwrap()) {
                Ok(_i) => _i,
                Err(_e) => {
                    error!("解析json时出错 {}",_e);
                    return HttpResponse::InternalServerError().body("please contact administrator");
                }
            };

            if obj.body.len() < page {
                return HttpResponse::NotFound().body("not exist");
            }

            let item = &obj.body[page-1];

            let url = match img_type {
                "mini" => &item.urls.thumb_mini,
                "original" => &item.urls.original,
                "regular" => &item.urls.regular,
                "small" => &item.urls.small,
                "thumb" => &item.urls.thumb_mini,
                _ => {
                    error ! ("img_type error {}", img_type);
                    return HttpResponse::NotFound().finish();
                }
            };
            match download_file(url, &data.client).await {
                Some(i) => HttpResponse::Ok().streaming(i),
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
            return make_redirect(format!("/img/small/{id}"));
        }
        return HttpResponse::NotFound().finish();
    }
    HttpResponse::NotFound().finish()
}


#[get("/{path:(img-(master|original)|c|user-profile).*}")]
pub async fn pximg_proxy(
    parm: web::Path<String>,
    data: web::Data<AppState>,
    req: HttpRequest,
) -> impl Responder {
    if req.headers().contains_key("if-modified-since") {
        return HttpResponse::NotModified().finish();
    }

    let url = format!("https://i.pximg.net/{}", parm.as_ref());

    let req_factory = || {
        let client = &data.client;
        let mut req_builder = client.get(&url);

        // if let Some(ua) =  req.headers().get(USER_AGENT){
        //     req_builder = req_builder.append_header((USER_AGENT,ua));
        // }

        if let Some(cookie) = req.headers().get(COOKIE) {
            req_builder = req_builder.append_header((COOKIE, cookie));
        }
        req_builder
    };

    let res = retry!(req_factory,3);
    return match res {
        Ok(i) => {
            let mut b = HttpResponse::Ok();
            b.content_type(ContentType::jpeg());
            if let Some(last) = i.headers().get(LAST_MODIFIED) {
                b.append_header((LAST_MODIFIED, last));
            }
            if let Some(l) = i.headers().get(CONTENT_LENGTH) {
                let size_s = SizedStream::new(l.to_str().unwrap().parse::<u64>().unwrap(), i);
                return b.body(size_s);
            }
            b.streaming(i)
        }

        Err(e) => {
            warn!("download error on {} {:?}",&url,&e);
            HttpResponse::NotFound().finish()
        }
    };
}
