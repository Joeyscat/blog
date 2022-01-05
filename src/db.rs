use std::str::FromStr;

use poem::Result;
use poem::{error::NotFoundError, http::StatusCode};

use mongodb::{
    bson::{doc, oid::ObjectId, Bson, Document},
    Database,
};
use tracing::info;

use crate::gitee;
use crate::model::{Article, User};

pub async fn create_article(article: Article, mongo: &Database) -> Result<String> {
    let id = mongo
        .collection::<Article>("article")
        .insert_one(article, None)
        .await
        .map_err(|e| poem::Error::new(e, StatusCode::INTERNAL_SERVER_ERROR))?
        .inserted_id
        .as_object_id()
        .unwrap()
        .to_string();

    Ok(id)
}

pub async fn update_article(article: Article, mongo: &Database) -> Result<bool> {
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
        .map_err(|e| poem::Error::new(e, StatusCode::INTERNAL_SERVER_ERROR))?
        .matched_count;

    Ok(matched_count > 0)
}

pub async fn get_article(article_id: String, mongo: &Database) -> Result<Article> {
    info!("get article: {}", article_id);
    let oid = ObjectId::from_str(article_id.as_str()).unwrap();
    let article = mongo
        .collection::<Article>("article")
        .find_one(doc! {"_id":oid}, None)
        .await
        .map_err(|e| poem::Error::new(e, StatusCode::INTERNAL_SERVER_ERROR))?;

    match article {
        Some(article) => Ok(article),
        None => Err(poem::error::NotFoundError.into()),
    }
}

use futures::stream::TryStreamExt;

pub async fn list_article(mongo: &Database) -> Result<Vec<Article>> {
    let cursor: mongodb::Cursor<Article> = mongo
        .collection::<Article>("article")
        .find(doc! {}, None)
        .await
        .map_err(|e| poem::Error::new(e, StatusCode::INTERNAL_SERVER_ERROR))?;

    let result: Vec<Article> = cursor.try_collect().await.unwrap();

    Ok(result)
}

pub async fn find_user_by_giteeid(mongo: &Database, id: i64) -> Result<User> {
    info!("find_user_by_giteeid : {}", id);

    let user = mongo
        .collection::<User>("user")
        .find_one(doc! {"inner.id":id}, None)
        .await
        .map_err(|e| poem::Error::new(e, StatusCode::INTERNAL_SERVER_ERROR))?;

    match user {
        Some(user) => Ok(user),
        None => Err(poem::error::NotFoundError.into()),
    }
}

pub async fn create_giteeuser(mongo: &Database, gitee_user: gitee::UserInfo) -> Result<String> {
    let mut user = User::default();
    user.auth_type = "gitee".to_owned();
    user.username = gitee_user.name.clone();
    user.inner = doc! {
        "id": gitee_user.id,
        "login": gitee_user.login,
        "name": gitee_user.name,
        "avatar_url": gitee_user.avatar_url,
        "blog": gitee_user.blog,
        "created_at": gitee_user.created_at,
        "email": gitee_user.email,
    };

    let id = mongo
        .collection::<User>("user")
        .insert_one(user, None)
        .await
        .map_err(|e| poem::Error::new(e, StatusCode::INTERNAL_SERVER_ERROR))?
        .inserted_id
        .as_object_id()
        .unwrap()
        .to_string();

    Ok(id)
}

// fn doc_to_article(doc: &Document) -> poem::Result<Article> {
//     // let article = Article {
//     //     id
//     // };
//     unimplemented!()
// }
