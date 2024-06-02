use mongodb::{Client, Collection, Database};
use crate::models::{User, MailingList};

pub struct MongoRepo {
    pub db: Database,
}

impl MongoRepo {
    pub async fn init() -> Self {
        let client = Client::with_uri_str("mongodb://localhost:27017")
            .await
            .expect("Failed to initialize client.");
        let db = client.database("mailing_list");
        MongoRepo { db }
    }

    pub fn get_users_collection(&self) -> Collection<User> {
        self.db.collection("users")
    }

    pub fn get_mailing_lists_collection(&self) -> Collection<MailingList> {
        self.db.collection("mailing_lists")
    }
}
