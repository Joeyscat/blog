use poem::error::InternalServerError;
use poem::Result;
use uuid::Uuid;

use crate::model::Article;

use crate::DBPool;

pub async fn create_article(article: Article, pool: &DBPool) -> Result<Uuid, poem::Error> {
    let rec = sqlx::query!(
        r#"
insert into article (id, title, raw_content, author_id, tags, status, created_time)
values ($1, $2, $3, $4, $5, $6, $7)
RETURNING id
"#,
        article.id,
        article.title,
        article.raw_content,
        article.author_id,
        article.tags,
        article.status,
        article.created_time,
    )
    .fetch_one(pool)
    .await
    .map_err(InternalServerError)?;

    Ok(rec.id)
}

pub async fn update_article(article: Article, pool: &DBPool) -> Result<bool, poem::Error> {
    let rows_affected = sqlx::query!(
        r#"
update article set title = $1, raw_content = $2, tags = $3, status = $4, updated_time = $5
where id = $6
"#,
        article.title,
        article.raw_content,
        article.tags,
        article.status,
        article.updated_time,
        article.id,
    )
    .execute(pool)
    .await
    .map_err(InternalServerError)?
    .rows_affected();

    Ok(rows_affected > 0)
}

pub async fn get_article(article_id: Uuid, pool: &DBPool) -> Result<Article, poem::Error> {
    let rec = sqlx::query!(
        r#"
select id, title, raw_content, author_id, tags, status, created_time, updated_time
from article
where id = $1
"#,
        article_id,
    )
    .fetch_one(pool)
    .await
    .map_err(InternalServerError)?;

    Ok(Article {
        id: rec.id,
        title: rec.title,
        raw_content: rec.raw_content,
        author_id: rec.author_id,
        tags: rec.tags,
        status: rec.status,
        created_time: rec.created_time,
        updated_time: rec.updated_time,
    })
}

pub async fn list_article(pool: &DBPool) -> Result<Vec<Article>, poem::Error> {
    let recs = sqlx::query!(
        r#"
select id, title, raw_content, author_id, tags, status, created_time, updated_time
from article
"#,
    )
    .fetch_all(pool)
    .await
    .map_err(InternalServerError)?;

    let mut articles = Vec::new();
    for rec in recs {
        articles.push(Article {
            id: rec.id,
            title: rec.title,
            raw_content: rec.raw_content,
            author_id: rec.author_id,
            tags: rec.tags,
            status: rec.status,
            created_time: rec.created_time,
            updated_time: rec.updated_time,
        });
    }

    Ok(articles)
}
