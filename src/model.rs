use chrono::{DateTime, Utc, MIN_DATETIME};
use mongodb::bson::oid::ObjectId;
use mongodb::bson::Document;
use serde_derive::{Deserialize, Serialize};

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
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub created_time: DateTime<Utc>,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub updated_time: DateTime<Utc>,
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
            updated_time: MIN_DATETIME,
            status: 1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub username: String,
    pub auth_type: String,
    pub inner: Document,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub created_time: DateTime<Utc>,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub updated_time: DateTime<Utc>,
    pub status: i16,
}

impl Default for User {
    fn default() -> Self {
        Self {
            id: ObjectId::new(),
            auth_type: "".to_owned(),
            username: "".to_owned(),
            inner: Document::new(),
            created_time: Utc::now(),
            updated_time: MIN_DATETIME,
            status: 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let id = ObjectId::new();

        println!("{}", id);
    }

    #[test]
    fn test_datetime() {
        use chrono::prelude::*;

        println!("{}", Utc::now());
        println!("{}", Utc::now().with_timezone(&FixedOffset::east(8 * 3600)));
    }
}
