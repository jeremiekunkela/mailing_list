use std::clone::Clone;
use serde::{Deserialize, Serialize};
use mongodb::bson::{doc, oid::ObjectId};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MailingList {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub list_name: String,
    pub owner: ObjectId,
    pub subscribers: Option<Vec<ObjectId>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub smtp_key: Option<String>,
}
impl Clone for MailingList {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            list_name: self.list_name.clone(),
            owner: self.owner.clone(),
            subscribers: self.subscribers.clone(),
            smtp_key: self.smtp_key.clone(),
        }
    }
}