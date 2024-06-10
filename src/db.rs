use mongodb::{Client, Collection, Database};
use std::sync::Arc;
use crate::models::{User, MailingList};
use mongodb::bson::doc;
use mongodb::bson::oid::ObjectId;
#[derive(Clone)]
pub struct MongoRepo {
    pub db: Arc<Database>,
}

impl MongoRepo {
    pub async fn init() -> Self {
        let client = Client::with_uri_str("mongodb://localhost:27017")
            .await
            .expect("Failed to initialize client.");
        let db = client.database("mailing_list");
        MongoRepo { db: Arc::new(db) }
    }

    pub fn get_users_collection(&self) -> Collection<User> {
        self.db.collection("users")
    }

    pub fn get_mailing_lists_collection(&self) -> Collection<MailingList> {
        self.db.collection("mailing_lists")
    }

    pub fn get_user_by_id(&self, user_id: ObjectId) -> mongodb::error::Result<Option<User>> {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(self.get_users_collection().find_one(doc! { "_id": user_id }, None))
    }

    pub fn get_mailing_list_by_id(&self, mailing_list_id: ObjectId) -> mongodb::error::Result<Option<MailingList>> {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(self.get_mailing_lists_collection().find_one(doc! { "_id": mailing_list_id }, None))
    }
}
