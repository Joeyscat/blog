use std::str::FromStr;

use poem::Result;
use poem::{error::NotFoundError, http::StatusCode};

use mongodb::{
    bson::{doc, oid::ObjectId, Bson, Document},
    Database,
};
use tracing::debug;

use crate::model::Article;

pub async fn create_article(article: Article, mongo: &Database) -> Result<String, poem::Error> {
    let id = mongo
        .collection::<Article>("article")
        .insert_one(article, None)
        .await
        .map_err(|e| poem::Error::new(e, StatusCode::from_u16(500).unwrap()))?
        .inserted_id
        .as_object_id()
        .unwrap()
        .to_string();

    Ok(id)
}

pub async fn update_article(article: Article, mongo: &Database) -> Result<bool, poem::Error> {
    let query = doc! {"_id":""};
    let update = doc! {
        "title":"",
        "raw_content":"",
        "tags":"",
        "status":"",
        "updated_time":"",
    };

    let matched_count = mongo
        .collection::<Article>("article")
        .update_one(query, update, None)
        .await
        .map_err(|e| poem::Error::new(e, StatusCode::from_u16(500).unwrap()))?
        .matched_count;

    Ok(matched_count > 0)
}

pub async fn get_article(article_id: String, mongo: &Database) -> Result<Article, poem::Error> {
    debug!("get article: {}", article_id);
    let oid = ObjectId::from_str(article_id.as_str()).unwrap();
    let article = mongo
        .collection::<Article>("article")
        .find_one(doc! {"_id":oid}, None)
        .await
        .map_err(|e| poem::Error::new(e, StatusCode::from_u16(500).unwrap()))?;

    match article {
        Some(article) => Ok(article),
        None => Err(poem::error::NotFoundError.into()),
    }
}

use futures::stream::TryStreamExt;

pub async fn list_article(mongo: &Database) -> Result<Vec<Article>, poem::Error> {
    let cursor: mongodb::Cursor<Article> = mongo
        .collection::<Article>("article")
        .find(doc! {}, None)
        .await
        .map_err(|e| poem::Error::new(e, StatusCode::from_u16(500).unwrap()))?;

    let result: Vec<Article> = cursor.try_collect().await.unwrap();

    Ok(result)
}

// fn doc_to_article(doc: &Document) -> poem::Result<Article> {
//     // let article = Article {
//     //     id
//     // };
//     unimplemented!()
// }
