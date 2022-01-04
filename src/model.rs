use chrono::{DateTime, Utc};
use serde_derive::{Deserialize, Serialize};
use mongodb::bson::oid::ObjectId;
use mongodb::bson::Document;

///
/// Model: Article
/// Db table: article
///
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Article {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub title: String,
    pub raw_content: String,
    pub tags: String,
    pub author_id: ObjectId,
    pub created_time: DateTime<Utc>,
    pub updated_time: Option<DateTime<Utc>>,
    pub status: i16,
}

impl Default for Article {
    fn default() -> Self {
        Self {
            id: ObjectId::new(),
            title: "".to_owned(),
            raw_content: "".to_owned(),
            tags: Default::default(),
            author_id: Default::default(),
            created_time: Utc::now(),
            updated_time: None,
            status: 1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub from: String,
    pub from_doc: Document,
    pub created_time: DateTime<Utc>,
    pub updated_time: Option<DateTime<Utc>>,
    pub status: i16,
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let id = ObjectId::new();

        println!("{}", id);
    }
}
