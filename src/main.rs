mod tmplate;
mod services;


use log::{debug, info, LevelFilter};
use actix_web::{get, web, App, HttpServer, Responder};
use std::env;

#[tokio::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::builder().filter_level(LevelFilter::Info).try_init().unwrap();
    let port: u16 = match env::var("PORT") {
        Ok(p) => {
            p.parse().unwrap()
        }
        Err(_) => {
            8080
        }
    };
    info!("Run server on port {}",port);
    HttpServer::new(|| {
        App::new()
            .wrap(actix_web::middleware::Logger::default())
            .wrap(actix_web::middleware::ErrorHandlers::default())
            .service(services::greet)
            .service(services::index)

    })
        .bind(("0.0.0.0", port))?
        .bind(("::",port))?
        .run()
        .await
}

