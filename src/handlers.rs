use actix_web::{delete, get, HttpResponse, post, Responder, web};
use bcrypt::{DEFAULT_COST, hash};
use futures::TryStreamExt;
use log::error;
use mongodb::bson::{doc, oid::ObjectId};
use regex::Regex;

use crate::db::MongoRepo;
use crate::models::{MailingList, User};
use crate::mailerService::wait_for_email;

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

#[get("/mailing_lists")]
async fn get_all_mailing_lists(repo: web::Data<MongoRepo>) -> impl Responder {
    let mailing_lists_collection = repo.get_mailing_lists_collection();

    let cursor = mailing_lists_collection.find(None, None).await;
    match cursor {
        Ok(mut docs) => {
            let mut mailing_lists = vec![];
            while let Some(doc) = docs.try_next().await.unwrap() {
                mailing_lists.push(doc);
            }
            HttpResponse::Ok().json(mailing_lists)
        },
        Err(e) => {
            error!("Error retrieving mailing lists : {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[get("/user/{user_id}/mailing_lists")]
async fn get_mailing_lists_by_user(
    repo: web::Data<MongoRepo>,
    user_id: web::Path<String>,
) -> impl Responder {
    let mailing_lists_collection = repo.get_mailing_lists_collection();
    let id = match ObjectId::parse_str(&user_id.into_inner()) {
        Ok(oid) => oid,
        Err(_) => return HttpResponse::BadRequest().json("Invalid user ID format"),
    };

    let filter = doc! { "owner": id };
    let cursor = mailing_lists_collection.find(filter, None).await;

    match cursor {
        Ok(mut docs) => {
            let mut mailing_lists = vec![];
            while let Some(doc) = docs.try_next().await.unwrap() {
                mailing_lists.push(doc);
            }
            HttpResponse::Ok().json(mailing_lists)
        },
        Err(e) => {
            error!("Error retrieving user's mailing lists : {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[post("/mailing_list")]
async fn create_mailing_list(repo: web::Data<MongoRepo>, mailing_list: web::Json<MailingList>) -> impl Responder {
    let mailing_lists_collection = repo.get_mailing_lists_collection();
    let users_collection = repo.get_users_collection();

    let owner_filter = doc! { "_id": &mailing_list.owner };
    let owner_exists = match users_collection.find_one(owner_filter, None).await {
        Ok(Some(_)) => true,
        Ok(None) => false,
        Err(e) => {
            error!("Error when verifying owner's existence : {:?}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    if !owner_exists {
        return HttpResponse::BadRequest().json("Owner does not exist");
    }

    if let Some(subscribers) = &mailing_list.subscribers {
        for subscriber_id in subscribers {
            let subscriber_filter = doc! { "_id": subscriber_id };
            let subscriber_exists = match users_collection.find_one(subscriber_filter, None).await {
                Ok(Some(_)) => true,
                Ok(None) => false,
                Err(e) => {
                    error!("Error verifying subscriber existence : {:?}", e);
                    return HttpResponse::InternalServerError().finish();
                }
            };

            if !subscriber_exists {
                return HttpResponse::BadRequest().json(format!("Subscriber with ID {} does not exist", subscriber_id));
            }
        }
    }

    let new_mailing_list = MailingList {
        id: None,
        list_name: mailing_list.list_name.clone(),
        owner: mailing_list.owner.clone(),
        subscribers: mailing_list.subscribers.clone(),
        smtp_key: mailing_list.smtp_key.clone(),
    };

    let result = mailing_lists_collection.insert_one(new_mailing_list, None).await;

    match result {
        Ok(insert_result) => {
            tokio::spawn(wait_for_email(repo.clone(), mailing_list.clone()));
            HttpResponse::Ok().json(insert_result.inserted_id)
        },
        Err(e) => {
            error!("Error inserting mailing list : {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
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