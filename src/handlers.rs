use actix_web::{post, delete, web, HttpResponse, Responder};
use mongodb::bson::{doc, oid::ObjectId};
use bcrypt::{hash, DEFAULT_COST};
use regex::Regex;
use log::error;
use crate::models::{User, MailingList};
use crate::db::MongoRepo;

fn is_valid_email(email: &str) -> bool {
    let email_regex = Regex::new(r"^[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\.[a-zA-Z0-9-.]+$").unwrap();
    email_regex.is_match(email)
}

#[post("/signup")]
async fn signup(repo: web::Data<MongoRepo>, user: web::Json<User>) -> impl Responder {
    if let Some(ref email) = user.email {
        if !is_valid_email(email) {
            return HttpResponse::BadRequest().json("Invalid email format");
        }

        let users_collection = repo.get_users_collection();
        let filter = doc! { "email": email };
        let existing_user = users_collection.find_one(filter, None).await;

        if let Ok(Some(_)) = existing_user {
            return HttpResponse::BadRequest().json("Email already in use");
        }
    }

    let users_collection = repo.get_users_collection();
    let filter = doc! { "username": &user.username };
    let existing_user = users_collection.find_one(filter, None).await;

    match existing_user {
        Ok(Some(_)) => HttpResponse::BadRequest().json("Username already in use"),
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
    let filter = doc! { "username": &credentials.username };
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

#[delete("/mailing_list/{id}")]
async fn delete_mailing_list(repo: web::Data<MongoRepo>, mailing_list_id: web::Path<String>) -> impl Responder {
    let mailing_lists_collection = repo.get_mailing_lists_collection();
    let id = mailing_list_id.into_inner();
    let object_id = match ObjectId::parse_str(&id) {
        Ok(oid) => oid,
        Err(_) => return HttpResponse::BadRequest().json("Invalid ID format"),
    };

    let filter = doc! { "_id": object_id };
    let result = mailing_lists_collection.delete_one(filter, None).await;

    match result {
        Ok(delete_result) => {
            if delete_result.deleted_count > 0 {
                HttpResponse::Ok().json("Mailing list deleted successfully")
            } else {
                HttpResponse::NotFound().json("Mailing list not found")
            }
        },
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}