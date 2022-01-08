use chrono::prelude::*;
use futures::StreamExt;
use poem::Result;
use std::{ops::SubAssign, str::FromStr};

use mongodb::{
    bson::{bson, doc, oid::ObjectId},
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
                "author_id": comment.author_id,
                "author_name": comment.author_name,
                "reply_to": comment.reply_to,
                "reply_to_name": comment.reply_to_name,
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

pub async fn get_article(
    article_id: String,
    mut comment_page_size: Option<i32>,
    mut comment_page: Option<i32>,
    mongo: &Database,
) -> Result<Article> {
    let comment_page_size = comment_page_size.get_or_insert(1); // Third argument to $slice must be positive: 0)
    let comment_page = comment_page.get_or_insert(0);
    if comment_page > &mut 0 {
        comment_page.sub_assign(1);
    }

    // db.article.aggregate([{$match:{_id: ObjectId("61d70cfa4a138b2ed4f4b088")}}, {$project: {comments:{$slice:["$comments",2,1]}}}]);
    let pipeline = vec![
        doc! {
            "$match":{"_id":ObjectId::from_str(article_id.as_str()).unwrap()},
        },
        doc! {
            "$lookup":{"from":"user","localField":"author_id","foreignField":"_id","as":"fromAuthors"},
        },
        doc! {
            "$unwind":"$fromAuthors",
        },
        doc! {
            "$project":{
                "_id":1,
                "title":1,
                "raw_content":1,
                "tags":1,
                "author_id":1,
                "created_time":1,
                "updated_time":1,
                "status":1,
                "comments":{"$slice":vec![bson!("$comments"), bson!(comment_page.clone() * comment_page_size.clone()), bson!(comment_page_size.clone())]},
                "total_comments":{"$size": {"$ifNull": vec![bson!("$comments"), bson!(Vec::<i32>::new())]}},
                "author_name":"$fromAuthors.username",
            },
        },
    ];
    let mut cursor = mongo
        .collection::<Article>("article")
        .aggregate(pipeline, None)
        .await
        .map_err(poem::error::InternalServerError)?;

    if let Some(c) = cursor.next().await {
        let article: Article = bson::from_document(c.map_err(poem::error::InternalServerError)?)
            .map_err(poem::error::InternalServerError)?;
        debug!("get article result: {}", article.title);
        Ok(article)
    } else {
        Err(poem::error::NotFoundError.into())
    }
}

pub async fn list_article(mongo: &Database) -> Result<Vec<Article>> {
    let pipeline = vec![
        doc! {
            "$match":{},
        },
        doc! {
            "$lookup":{"from":"user","localField":"author_id","foreignField":"_id","as":"fromAuthors"},
        },
        doc! {
            "$unwind":"$fromAuthors",
        },
        doc! {
            "$project":{
                "_id":1,
                "title":1,
                "raw_content":1,
                "tags":1,
                "author_id":1,
                "created_time":1,
                "updated_time":1,
                "status":1,
                "total_comments":{"$size": {"$ifNull": vec![bson!("$comments"), bson!(Vec::<i32>::new())]}},
                "author_name":"$fromAuthors.username",
            },
        },
    ];
    let mut cursor = mongo
        .collection::<Article>("article")
        .aggregate(pipeline, None)
        .await
        .map_err(poem::error::InternalServerError)?;

    let mut result = Vec::new();
    while let Some(c) = cursor.next().await {
        let article: Article = bson::from_document(c.map_err(poem::error::InternalServerError)?)
            .map_err(poem::error::InternalServerError)?;

        result.push(article);
    }
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

pub async fn find_user_by_id(id: &str, mongo: &Database) -> Result<User> {
    info!("find_user_by_id : {}", id);
    let oid = ObjectId::from_str(id).unwrap();
    let user = mongo
        .collection::<User>("user")
        .find_one(doc! {"_id":oid}, None)
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
