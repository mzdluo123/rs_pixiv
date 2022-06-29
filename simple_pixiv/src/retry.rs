
#[macro_export]
macro_rules! retry {
    ( $req:expr,$count:expr) => {
        {
            use log::{warn};
            use std::{time::Duration};
            use awc::error::SendRequestError;
              let mut count = 0;
              loop {
                  let  rsp = $req().send().await;
                  match rsp {
                    Ok(_i) => {
                        break Ok(_i);
                    }

                    Err(_e) => {
                        match _e {
                            SendRequestError::H2(ref h2_error)=>{
                                     if count < $count {
                                    warn!("error {:?} ,retry...",h2_error);
                                    count += 1;
                                    tokio::time::sleep(Duration::from_millis(500)).await;
                                    }else {
                                        break Err(_e);
                                    }
                            }
                            _=> {  break Err(_e); }
                        }

                    }
              }
           }
         }
    };
}

