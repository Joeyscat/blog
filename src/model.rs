use chrono::{DateTime, Utc};
use serde_derive::{Deserialize, Serialize};
use uuid::Uuid;

///
/// Model: Article
/// Db table: article
///
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Article {
    pub id: Uuid,
    pub title: String,
    pub raw_content: String,
    pub tags: String,
    pub author_id: Uuid,
    pub created_time: DateTime<Utc>,
    pub updated_time: Option<DateTime<Utc>>,
    pub status: i16,
}

impl Default for Article {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
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

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let id = uuid::Uuid::new_v4();

        println!("{}", id);
    }
}
