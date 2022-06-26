use std::{collections::HashSet, sync::{Mutex, Arc, RwLock}};

use actix_web::http::header::USER_AGENT;
use askama::filters::format;

use log::{error, info};
use rand::Rng;

use crate::bookmark_struct::{BookmarkData, self};



struct ImgIdStorage{
     id_set: Arc<Mutex<HashSet<String>>>
}


impl ImgIdStorage {
    fn new()->ImgIdStorage{
        ImgIdStorage { id_set: Arc::new(Mutex::new(HashSet::new())) }
    }


    async fn init_id_set(self:&mut Self,user_id:&str,cookie:&str){
            let client = awc::Client::default();
            let mut page = 0;
            loop {
                info!("page {} start download",&page);
                let url = format!("https://www.pixiv.net/ajax/user/{}/illusts/bookmarks?tag=&offset={}&limit=30&rest=show&lang=zh",user_id,page*30);
                
                let rsp = client.get(url)
                .append_header((USER_AGENT,"Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/102.0.5005.63 Safari/537.36 Edg/102.0.1245.33"))
                .append_header(("cookie",cookie))
                .append_header(("referer","https://www.pixiv.net/"))
                .send().await;

                match rsp {
                    Ok(mut i) =>{
                        let content = i.body().await.map_err(|_e|{
                            error!("download bookmark error {}",_e);
                        }).unwrap();
                        
                        let data_obj = serde_json::from_str::<bookmark_struct::Root>(&String::from_utf8(content.to_vec()).unwrap()).map_err(|_e|{
                            error!("parse bookmark error {}",_e);
                        }).unwrap();
                        
                        if data_obj.error {
                            error!("response error {}",data_obj.message);
                            return;
                        }
                        if data_obj.body.works.len() == 0{
                            info!("download bookmark success, img count: {}",self.id_set.lock().unwrap().len());
                            return;
                        }
                        {
                            let mut id_set = self.id_set.lock().unwrap();
                            for work in data_obj.body.works  {
                                if work.restrict==0 && work.x_restrict==0 {
                                    id_set.insert(work.id);
                                }
                            }
                        }
                        page += 1;
                        
                    }
                    Err(_e)=>{
                        error!("download bookmark error {}",_e);
                        return;
                    }
                }
            }
    }

    fn random_img(self:&Self)->Option<String>{
        let  readable = self.id_set.lock().ok()?;
        let mut rand = rand::thread_rng();

        let id_list:Vec<String> = readable.clone().into_iter().collect();
    
        let index = rand.gen_range(0..id_list.len()-1);
        return id_list.get(index).map(|x|{
            x.clone()
        });

    }
}