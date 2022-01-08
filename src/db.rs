use chrono::prelude::*;
use poem::Result;
use std::str::FromStr;

use mongodb::{
    bson::{doc, oid::ObjectId},
    Database,
};
use tracing::{debug, info};

use crate::gitee;
use crate::model::{Article, Comment, User};

pub async fn create_article(article: Article, mongo: &Database) -> Result<String> {
    let now = Utc::now().with_timezone(&FixedOffset::east(8 * 3600));
    let new_article = doc! {
        "_id": article.id,
        "title": article.title,
        "raw_content": article.raw_content,
        "tags": article.tags,
        "author_id": article.author_id,
        "created_time": now,
        "updated_time": now,
        "status": 1,
    };

    let id = mongo
        .collection("article")
        .insert_one(new_article, None)
        .await
        .map_err(poem::error::InternalServerError)?
        .inserted_id
        .as_object_id()
        .unwrap()
        .to_string();

    Ok(id)
}

pub async fn update_article(article: Article, mongo: &Database) -> Result<bool> {
    let query = doc! {"_id":article.id};
    let update = doc! {
        "$set":{
            "title":article.title,
            "raw_content":article.raw_content,
            "tags":article.tags,
            "status":article.status as i32,
            "updated_time":Utc::now().with_timezone(&FixedOffset::east(8 * 3600)),
        }
    };

    let matched_count = mongo
        .collection::<Article>("article")
        .update_one(query, update, None)
        .await
        .map_err(poem::error::InternalServerError)?
        .matched_count;

    Ok(matched_count > 0)
}

pub async fn append_comment(
    article_id: String,
    comment: Comment,
    mongo: &Database,
) -> Result<bool> {
    let query = doc! {"_id":ObjectId::from_str(article_id.as_str()).unwrap()};
    let update = doc! {
        "$push":{
            "comments":{
                "content": comment.content,
                "article_id": comment.article_id,
                "author_id": comment.author_id,
                "reply_to": comment.reply_to,
                "created_time": Utc::now().with_timezone(&FixedOffset::east(8 * 3600)),
                "updated_time": Utc::now().with_timezone(&FixedOffset::east(8 * 3600)),
                "status": comment.status as i32,
            },
        }
    };

    debug!("append comment: query={:?}, update={:?}", query, update);

    let matched_count = mongo
        .collection::<Article>("article")
        .update_one(query, update, None)
        .await
        .map_err(poem::error::InternalServerError)?
        .matched_count;

    Ok(matched_count > 0)
}

pub async fn get_article(article_id: String, mongo: &Database) -> Result<Article> {
    let oid = ObjectId::from_str(article_id.as_str()).unwrap();
    let article = mongo
        .collection::<Article>("article")
        .find_one(doc! {"_id":oid}, None)
        .await
        .map_err(poem::error::InternalServerError)?;

    match article {
        Some(article) => {
            debug!("get article result: {}", article.title);
            Ok(article)
        }
        None => Err(poem::error::NotFoundError.into()),
    }
}

use futures::stream::TryStreamExt;

pub async fn list_article(mongo: &Database) -> Result<Vec<Article>> {
    let cursor: mongodb::Cursor<Article> = mongo
        .collection::<Article>("article")
        .find(doc! {}, None)
        .await
        .map_err(poem::error::InternalServerError)?;

    let result: Vec<Article> = cursor.try_collect().await.unwrap();

    Ok(result)
}

pub async fn find_user_by_giteeid(mongo: &Database, id: i64) -> Result<User> {
    info!("find_user_by_giteeid : {}", id);

    let user = mongo
        .collection::<User>("user")
        .find_one(doc! {"inner.id":id}, None)
        .await
        .map_err(poem::error::InternalServerError)?;

    match user {
        Some(user) => Ok(user),
        None => Err(poem::error::NotFoundError.into()),
    }
}

pub async fn create_giteeuser(mongo: &Database, gitee_user: gitee::UserInfo) -> Result<String> {
    let now = Utc::now().with_timezone(&FixedOffset::east(8 * 3600));
    let new_user = doc! {
        "_id": ObjectId::new(),
        "username": &gitee_user.name,
        "auth_type": "gitee".to_owned(),
        "inner": {
            "id": gitee_user.id,
            "login": gitee_user.login,
            "name": gitee_user.name,
            "avatar_url": gitee_user.avatar_url,
            "blog": gitee_user.blog,
            "created_at": gitee_user.created_at,
            "email": gitee_user.email,
        },
        "created_time": now,
        "updated_time": now,
        "status": 1,
    };

    let id = mongo
        .collection("user")
        .insert_one(new_user, None)
        .await
        .map_err(poem::error::InternalServerError)?
        .inserted_id
        .as_object_id()
        .unwrap()
        .to_string();

    Ok(id)
}
