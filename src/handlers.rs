use actix_web::{post, web, App, HttpServer, HttpResponse, Responder};
use mongodb::bson::doc;
use bcrypt::{hash, DEFAULT_COST};
use regex::Regex;
use crate::models::{User, MailingList};
use crate::db::MongoRepo;

fn is_valid_email(email: &str) -> bool {
    let email_regex = Regex::new(r"^[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\.[a-zA-Z0-9-.]+$").unwrap();
    email_regex.is_match(email)
}

#[post("/signup")]
async fn signup(repo: web::Data<MongoRepo>, user: web::Json<User>) -> impl Responder {
    if !is_valid_email(&user.email) {
        return HttpResponse::BadRequest().json("Invalid email format");
    }

    let users_collection = repo.get_users_collection();
    let filter = doc! { "email": &user.email };
    let existing_user = users_collection.find_one(filter, None).await;

    match existing_user {
        Ok(Some(_)) => HttpResponse::BadRequest().json("Email already in use"),
        Ok(None) => {
            let hashed_password = hash(&user.password, DEFAULT_COST).unwrap();
            let new_user = User {
                id: None,
                username: user.username.clone(),
                email: user.email.clone(),
                password: hashed_password,
            };
            let result = users_collection.insert_one(new_user, None).await;

            match result {
                Ok(insert_result) => HttpResponse::Ok().json(insert_result.inserted_id),
                Err(_) => HttpResponse::InternalServerError().finish(),
            }
        },
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mongo_repo = MongoRepo::init().await;
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(mongo_repo.clone()))
            .service(signup)
            .service(login)
    })
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
