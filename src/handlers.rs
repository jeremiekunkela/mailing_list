use actix_web::{post, web, App, HttpServer, HttpResponse, Responder};
use mongodb::bson::doc;
use bcrypt::{hash, DEFAULT_COST};
use crate::models::{User, MailingList};
use crate::db::MongoRepo;

#[post("/signup")]
async fn signup(repo: web::Data<MongoRepo>, user: web::Json<User>) -> impl Responder {
    let hashed_password = hash(&user.password, DEFAULT_COST).unwrap();
    let user = User {
        id: None,
        username: user.username.clone(),
        email: user.email.clone(),
        password: hashed_password,
    };
    let users_collection = repo.get_users_collection();
    let result = users_collection.insert_one(user, None).await;

    match result {
        Ok(insert_result) => HttpResponse::Ok().json(insert_result.inserted_id),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[post("/login")]
async fn login(repo: web::Data<MongoRepo>, credentials: web::Json<User>) -> impl Responder {
    let users_collection = repo.get_users_collection();
    let filter = doc! { "email": &credentials.email };
    let result = users_collection.find_one(filter, None).await;

    match result {
        Ok(Some(user)) => {
            if bcrypt::verify(&credentials.password, &user.password).unwrap() {
                HttpResponse::Ok().json("Login successful")
            } else {
                HttpResponse::Unauthorized().json("Invalid credentials")
            }
        },
        Ok(None) => HttpResponse::Unauthorized().json("Invalid credentials"),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
