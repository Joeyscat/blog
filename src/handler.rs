use lazy_static::lazy_static;
use poem::{
    handler,
    http::{header, StatusCode},
    session::Session,
    web::{Data, Form, Html, Query},
    IntoResponse, Response,
};
use serde::Deserialize;
use std::str::FromStr;
use tera::{Context, Tera};
use markdown;

use crate::DBPool;

use crate::db;
use crate::model::Article;

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let mut tera = match Tera::new("templates/**/*") {
            Ok(t) => t,
            Err(e) => {
                println!("Parsing error(s): {}", e);
                ::std::process::exit(1);
            }
        };
        tera.autoescape_on(vec!["html", ".sql"]);
        // tera.register_filter("do_nothing", do_nothing_filter);
        tera
    };
}

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

#[handler]
pub async fn index(_session: &Session, pool: Data<&DBPool>) -> impl IntoResponse {
    let articles = db::list_article(&pool).await;

    match articles {
        Ok(articles) => {
            let mut context = Context::new();
            context.insert("title", "首页");
            context.insert("article_list", &articles);
            let s = TEMPLATES.render("index.html", &context).unwrap();
            Html(s).into_response()
        }
        Err(err) => {
            let mut context = Context::new();
            context.insert("title", "错误");
            context.insert("msg", &err.to_string());
            let s = TEMPLATES.render("error.html", &context).unwrap();
            Html(s).into_response()
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
        Ok(mut article) => {
            let content = markdown::to_html(article.raw_content.as_str());
            article.raw_content = content;

            let mut context = Context::new();
            context.insert("title", &article.title);
            context.insert("article", &article);
            let s = TEMPLATES.render("article.html", &context).unwrap();
            Html(s).into_response()
        }
        Err(err) => {
            let mut context = Context::new();

            if err.to_string().contains("no rows returned") {
                context.insert("title", "404");
                let s = TEMPLATES.render("404.html", &context).unwrap();
                return Html(s).into_response();
            }

            context.insert("title", "错误");
            context.insert("msg", &err.to_string());
            let s = TEMPLATES.render("error.html", &context).unwrap();
            Html(s).into_response()
        }
    }
}

#[handler]
pub async fn publish_article_page(session: &Session) -> impl IntoResponse {
    match session.get::<String>("username") {
        Some(_username) => {
            let mut context = Context::new();
            context.insert("title", "写文章");
            let s = TEMPLATES.render("publish_article.html", &context).unwrap();
            Html(s).into_response()
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
                    let mut context = Context::new();
                    context.insert("title", &article.title);
                    context.insert("article", &article);
                    let s = TEMPLATES.render("edit_article.html", &context).unwrap();
                    Html(s).into_response()
                }
                Err(err) => {
                    let mut context = Context::new();

                    if err.to_string().contains("no rows returned") {
                        context.insert("title", "404");
                        let s = TEMPLATES.render("404.html", &context).unwrap();
                        return Html(s).into_response();
                    }
        
                    context.insert("title", "错误");
                    context.insert("msg", &err.to_string());
                    let s = TEMPLATES.render("error.html", &context).unwrap();
                    Html(s).into_response()
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
                    let mut context = Context::new();

                    if err.to_string().contains("no rows returned") {
                        context.insert("title", "404");
                        let s = TEMPLATES.render("404.html", &context).unwrap();
                        return Html(s).into_response();
                    }
        
                    context.insert("title", "错误");
                    context.insert("msg", &err.to_string());
                    let s = TEMPLATES.render("error.html", &context).unwrap();
                    Html(s).into_response()
                }
            }
        }
        None => Response::builder()
            .status(StatusCode::FOUND)
            .header(header::LOCATION, "/signin")
            .finish(),
    }
}
