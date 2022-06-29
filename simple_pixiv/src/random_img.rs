use std::{
    collections::HashSet,
    sync::{Arc, RwLock},
    time::Duration,
};

use actix_web::http::header::USER_AGENT;

use log::{error, info, debug};
use rand::Rng;

use crate::bookmark_struct::{self};

pub struct ImgIdStorage {
    pub id_set: HashSet<String>,
    pub id_list: Vec<String>
}

impl ImgIdStorage {
    pub fn new() -> ImgIdStorage {
        ImgIdStorage {
            id_set: HashSet::new(),
            id_list: Vec::new()
        }
    }

    pub fn refresh_list(&mut self){
        let readable = &self.id_set;
        if readable.is_empty() {
            return;
        }
        let id_list: Vec<String> = readable.clone().into_iter().collect();
        self.id_list = id_list;
    }
    pub fn random_img(&self) -> Option<String> {
        if self.id_list.is_empty() {
            return None;
        }
        let mut rand = rand::thread_rng();
        let index = rand.gen_range(0.. self.id_list.len()- 1);
        return self.id_list.get(index).cloned();
    }
}

pub async fn init_id_set(storage: &Arc<RwLock<ImgIdStorage>>, user_id: &str, cookie: &str) {
    let client = awc::Client::default();
    let mut page = 0;
    'l: loop {
        info!("page {} start download", &page);
        let url = format!("https://www.pixiv.net/ajax/user/{}/illusts/bookmarks?tag=&offset={}&limit=30&rest=show&lang=zh",user_id,page*30);

        let rsp = client.get(url)
        .append_header((USER_AGENT,"Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/102.0.5005.63 Safari/537.36 Edg/102.0.1245.33"))
        .append_header(("cookie",cookie))
        .append_header(("referer","https://www.pixiv.net/"))
        .send().await;

        match rsp {
            Ok(mut i) => {
                let content = i
                    .body()
                    .await
                    .map_err(|_e| {
                        error!("download bookmark error {}", _e);
                    })
                    .unwrap();
                let content =  &String::from_utf8(content.to_vec()).unwrap();
                debug!("{content}");
                let data_obj = match serde_json::from_str::<bookmark_struct::Root>(
                    &content
                 ) {
                    Ok(_c)=>_c,
                    Err(_e)=>{
                        error!("parse bookmark error: {}", _e);
                        page +=1;
                        tokio::time::sleep(Duration::from_secs(2)).await;
                        continue 'l;
                    }
                };
            
                if data_obj.error {
                    error!("response error {}", data_obj.message);
                    return;
                }
                if data_obj.body.works.is_empty() {
                    if let Ok(readable) = storage.try_read() {
                        info!(
                            "download bookmark finish, img count: {}",
                            &readable.id_set.len()
                        );
                    }

                    return;
                }

                if let Ok(mut writeable) = storage.try_write() {
                    for work in data_obj.body.works {
                        if work.restrict == 0 && work.x_restrict == 0 {
                            if let Some(work_id) = work.id.as_str() {
                                writeable.id_set.insert(work_id.to_string());
                            }else if let Some(work_id) = work.id.as_u64()  {
                                writeable.id_set.insert(work_id.to_string());
                            }
                        
                        }
                    }
                    info!(
                        "download page success img count: {}",
                        &writeable.id_set.len()
                    );
                }

                page += 1;
                if let Ok(mut _i) = storage.try_write(){
                    _i.refresh_list()
                }
            }
            Err(_e) => {
                error!("download bookmark error {}", _e);
                return;
            }
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}

pub async fn refresh_random(storage: Arc<RwLock<ImgIdStorage>>, user_id: String, cookie: String) {
    loop {
        info!("start download bookmark");

        init_id_set(&storage, &user_id, &cookie).await;

        tokio::time::sleep(Duration::from_secs(60 * 60)).await;
    }
}
