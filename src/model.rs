use chrono::{DateTime, Utc, MIN_DATETIME};
use mongodb::bson::oid::ObjectId;
use mongodb::bson::Document;
use serde_derive::{Deserialize, Serialize};
use std::str::FromStr;

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
    pub comments: Option<Vec<Comment>>,
    pub total_comments: Option<i32>,
    pub author_name: Option<String>,
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
            updated_time: Utc::now(),
            status: 1,
            comments: None,
            total_comments: None,
            author_name: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub content: String,
    pub author_id: ObjectId,
    pub author_name: String,
    pub reply_to: Option<ObjectId>,
    pub reply_to_name: Option<String>,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub created_time: DateTime<Utc>,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub updated_time: DateTime<Utc>,
    pub status: i16,
}

impl Comment {
    pub fn new(
        content: String,
        author_id: String,
        author_name: String,
        reply_to: Option<String>,
        reply_to_name: Option<String>,
    ) -> Self {
        let author_id = ObjectId::from_str(author_id.as_str()).unwrap();
        let reply_to = reply_to
            .as_ref()
            .map(|s| ObjectId::from_str(s.as_str()).unwrap());

        Self {
            content,
            author_name,
            author_id,
            reply_to,
            reply_to_name,
            created_time: Utc::now(),
            updated_time: Utc::now(),
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
        use mongodb::bson::doc;

        let now_utc = Utc::now();
        let now_east_8 = Utc::now().with_timezone(&FixedOffset::east(8 * 3600));

        println!("now_utc:\t{}", now_utc);
        println!("now_east_8:\t{}", now_east_8);

        let d = doc! {
            "now":now_east_8
        };
        println!("doc:\t\t{}", d);

        let dd = mongodb::bson::Bson::DateTime(now_east_8.into());
        println!("dd:\t{}", dd);
    }

    #[test]
    fn test_div() {
        let d = (8 as f32 / 3 as f32).ceil() as i32;
        println!("{}",d);

        let mut v = vec![1];
        for i in 2..d + 1 {
            v.push(i);
        }

        println!("{:?}", v);

        let d = (8 as f32 / 2 as f32).ceil() as i32;
        println!("{}",d);

        let mut v = vec![1];
        for i in 2..d + 1 {
            v.push(i);
        }

        println!("{:?}", v);
    }
}
