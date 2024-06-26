extern crate alloc;

mod models;
mod db;
mod handlers;
mod mailerService;

use actix_web::{web, App, HttpServer};
use crate::db::MongoRepo;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let repo = MongoRepo::init().await;
    let data = web::Data::new(repo);

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .service(handlers::signup)
            .service(handlers::login)
            .service(handlers::create_mailing_list)
            .service(handlers::delete_mailing_list)
            .service(handlers::get_mailing_lists_by_user)
            .service(handlers::get_all_mailing_lists)
    })
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
