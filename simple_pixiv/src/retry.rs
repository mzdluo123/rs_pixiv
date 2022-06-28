
#[macro_export]
macro_rules! retry {
    ( $req:expr,$count:expr) => {
        {
            use log::{ warn};
              let mut count = 0;
              loop {
                  let  rsp = $req().send().await;
                  match rsp {
                    Ok(_i) => {
                        break Ok(_i);
                    }
                    Err(_e) => {
                        if count < $count {
                            warn!("error {:?} ,retry...",_e);
                            count += 1;
                        }else {
                            break Err(_e);
                        }
                    }
              }
           }
         }
    };
}

