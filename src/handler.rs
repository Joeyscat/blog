use std::str::FromStr;

use askama::Template;
use markdown;
use poem::{
    handler,
    http::{header, StatusCode},
    session::Session,
    web::{Data, Form, Html, Query},
    IntoResponse, Response,
};
use serde::Deserialize;

use crate::DBPool;

use crate::db;
use crate::model::Article;

#[handler]
pub fn signin_ui() -> impl IntoResponse {
    Html(
        r#"
    <!DOCTYPE html>
    <html>
    <head><meta charset="UTF-8"><title>Example CSRF</title></head>
    <body>
    <form action="/signin" method="post">
        <div>
            <label>Username:<input type="text" name="username" value="test" /></label>
        </div>
        <div>
            <label>Password:<input type="password" name="password" value="123456" /></label>
        </div>
        <button type="submit">Login</button>
    </form>
    </body>
    </html>
    "#,
    )
}

#[derive(Deserialize)]
pub struct SigninParams {
    username: String,
    password: String,
}

#[handler]
pub fn signin(Form(params): Form<SigninParams>, session: &Session) -> impl IntoResponse {
    if params.username == "test" && params.password == "123456" {
        session.set("username", params.username);
        Response::builder()
            .status(StatusCode::FOUND)
            .header(header::LOCATION, "/")
            .finish()
    } else {
        Html(
            r#"
    <!DOCTYPE html>
    <html>
    <head><meta charset="UTF-8"><title>Example CSRF</title></head>
    <body>
    no such user
    </body>
    </html>
    "#,
        )
        .into_response()
    }
}

#[handler]
pub fn logout(session: &Session) -> impl IntoResponse {
    session.purge();
    Response::builder()
        .status(StatusCode::FOUND)
        .header(header::LOCATION, "/signin")
        .finish()
}

#[derive(Template)]
#[template(path = "404.html")]
pub struct NotFoundTemplate {
    pub title: String,
}

#[derive(Template)]
#[template(path = "error.html")]
pub struct ErrorTemplate {
    pub title: String,
    pub msg: String,
}

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub title: String,
    pub article_list: Vec<Article>,
}

#[derive(Template)]
#[template(path = "article.html")]
pub struct ArticleTemplate {
    pub id: String,
    pub title: String,
    pub author: String,
    pub created_time: String,
    pub content: String,
    pub tags: String,
}

#[derive(Template)]
#[template(path = "publish_article.html")]
pub struct PublishArticleTemplate {
    pub title: String,
}

#[derive(Template)]
#[template(path = "edit_article.html")]
pub struct EditArticleTemplate {
    pub id: String,
    pub title: String,
    pub tags: String,
    pub content: String,
}

#[handler]
pub async fn index(_session: &Session, pool: Data<&DBPool>) -> impl IntoResponse {
    let articles = db::list_article(&pool).await;

    match articles {
        Ok(articles) => {
            let tpl = IndexTemplate {
                title: "首页".to_string(),
                article_list: articles,
            };

            Html(tpl.render().unwrap()).into_response()
        }
        Err(err) => {
            let tpl = ErrorTemplate {
                title: "错误".to_string(),
                msg: err.to_string(),
            };

            Html(tpl.render().unwrap()).into_response()
        }
    }
}

#[derive(Deserialize)]
pub struct FindArticle {
    id: Option<String>,
    title: Option<String>,
}

#[handler]
pub async fn article_details(
    Query(FindArticle { id, title: _ }): Query<FindArticle>,
    _session: &Session,
    pool: Data<&DBPool>,
) -> impl IntoResponse {
    let article_id = uuid::Uuid::from_str(id.unwrap().as_str()).unwrap();

    let article_r = db::get_article(article_id, &pool).await;

    match article_r {
        Ok(article) => {
            let content = markdown::to_html(article.raw_content.as_str());

            let tpl = ArticleTemplate {
                id: article.id.to_string(),
                title: article.title,
                author: article.author_id.to_string(),
                created_time: article.created_time.to_string(),
                tags: article.tags,
                content: content,
            };

            Html(tpl.render().unwrap()).into_response()
        }
        Err(err) => {
            if err.to_string().contains("no rows returned") {
                let tpl = NotFoundTemplate {
                    title: "404".to_string(),
                };

                return Html(tpl.render().unwrap()).into_response();
            }
            let tpl = ErrorTemplate {
                title: "错误".to_string(),
                msg: err.to_string(),
            };

            Html(tpl.render().unwrap()).into_response()
        }
    }
}

#[handler]
pub async fn publish_article_page(session: &Session) -> impl IntoResponse {
    match session.get::<String>("username") {
        Some(_username) => {
            let tpl = PublishArticleTemplate {
                title: "写文章".to_string(),
            };

            Html(tpl.render().unwrap()).into_response()
        }
        None => Response::builder()
            .status(StatusCode::FOUND)
            .header(header::LOCATION, "/signin")
            .finish(),
    }
}

#[derive(Deserialize)]
pub struct PublishArticleParams {
    title: String,
    raw_content: String,
    tags: String,
}

#[handler]
pub async fn publish_article(
    Form(params): Form<PublishArticleParams>,
    session: &Session,
    pool: Data<&DBPool>,
) -> impl IntoResponse {
    match session.get::<String>("username") {
        Some(_username) => {
            let mut new_article = Article::default();
            new_article.title = params.title;
            new_article.raw_content = params.raw_content;
            new_article.tags = params.tags;

            let r = db::create_article(new_article, &pool).await.unwrap();

            Response::builder()
                .status(StatusCode::FOUND)
                .header(header::LOCATION, format!("/article?id={}", r.to_string()))
                .finish()
        }
        None => Response::builder()
            .status(StatusCode::FOUND)
            .header(header::LOCATION, "/signin")
            .finish(),
    }
}

#[handler]
pub async fn edit_article_page(
    Query(FindArticle { id, title: _ }): Query<FindArticle>,
    session: &Session,
    pool: Data<&DBPool>,
) -> impl IntoResponse {
    match session.get::<String>("username") {
        Some(_username) => {
            let article_id = uuid::Uuid::from_str(id.unwrap().as_str()).unwrap();

            let article_r = db::get_article(article_id, &pool).await;

            match article_r {
                Ok(article) => {
                    let tpl = EditArticleTemplate {
                        id: article.id.to_string(),
                        title: article.title.to_string(),
                        tags: article.tags.to_string(),
                        content: article.raw_content.to_string(),
                    };

                    Html(tpl.render().unwrap()).into_response()
                }
                Err(err) => {
                    if err.to_string().contains("no rows returned") {
                        let tpl = NotFoundTemplate {
                            title: "404".to_string(),
                        };

                        return Html(tpl.render().unwrap()).into_response();
                    }
                    let tpl = ErrorTemplate {
                        title: "错误".to_string(),
                        msg: err.to_string(),
                    };

                    Html(tpl.render().unwrap()).into_response()
                }
            }
        }
        None => Response::builder()
            .status(StatusCode::FOUND)
            .header(header::LOCATION, "/signin")
            .finish(),
    }
}

#[derive(Deserialize)]
pub struct EditArticleParams {
    id: String,
    title: String,
    raw_content: String,
    tags: String,
}

#[handler]
pub async fn edit_article(
    Form(params): Form<EditArticleParams>,
    session: &Session,
    pool: Data<&DBPool>,
) -> impl IntoResponse {
    match session.get::<String>("username") {
        Some(_username) => {
            let article_id = uuid::Uuid::from_str(params.id.as_str()).unwrap();

            let article_r = db::get_article(article_id, &pool).await;

            match article_r {
                Ok(mut article) => {
                    article.title = params.title;
                    article.raw_content = params.raw_content;
                    article.tags = params.tags;
                    let _ok = db::update_article(article, &pool).await.unwrap();

                    Response::builder()
                        .status(StatusCode::FOUND)
                        .header(header::LOCATION, format!("/article?id={}", params.id))
                        .finish()
                }
                Err(err) => {
                    if err.to_string().contains("no rows returned") {
                        let tpl = NotFoundTemplate {
                            title: "404".to_string(),
                        };

                        return Html(tpl.render().unwrap()).into_response();
                    }
                    let tpl = ErrorTemplate {
                        title: "错误".to_string(),
                        msg: err.to_string(),
                    };

                    Html(tpl.render().unwrap()).into_response()
                }
            }
        }
        None => Response::builder()
            .status(StatusCode::FOUND)
            .header(header::LOCATION, "/signin")
            .finish(),
    }
}
