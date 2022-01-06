use lazy_static::lazy_static;
use markdown;
use mongodb::{bson::oid::ObjectId, Database};
use poem::{
    handler,
    http::{header, StatusCode},
    session::Session,
    web::{Data, Form, Html, Query},
    IntoResponse, Response, Result,
};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use tera::{Context, Tera};
use tracing::info;

use crate::db;
use crate::gitee;
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

    pub static ref CLIENT_ID:String = {
        std::env::var("GITEE_CLIENT_ID").expect("GITEE_CLIENT_ID")
    };
    pub static ref CLIENT_SECRET:String = {
        std::env::var("GITEE_CLIENT_SECRET").expect("GITEE_CLIENT_SECRET")
    };
    pub static ref REDIRECT_URI:String = {
        std::env::var("GITEE_REDIRECT_URI").expect("GITEE_REDIRECT_URI")
    };
}

#[handler]
pub fn signin_ui() -> impl IntoResponse {
    let mut context = Context::new();
    context.insert("title", "登录");
    let gitee_signin_uri = format!(
        "https://gitee.com/oauth/authorize?client_id={}&redirect_uri={}&response_type=code",
        CLIENT_ID.to_string(),
        REDIRECT_URI.to_string()
    );
    context.insert("gitee_signin_uri", &gitee_signin_uri);
    let s = TEMPLATES.render("signin.html", &context).unwrap();
    Html(s).into_response()
}

#[derive(Deserialize)]
pub struct SigninParams {
    username: String,
    password: String,
}

#[handler]
pub fn signin(Form(params): Form<SigninParams>, session: &Session) -> impl IntoResponse {
    if params.username == "jojo" && params.password == "123456" {
        session.set("username", "61cdbe3c9b146b6a8d851aff");
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

#[derive(Deserialize)]
pub struct GiteeSignin {
    state: Option<String>,
    code: String,
}

#[handler]
pub async fn gitee_signin(
    Query(GiteeSignin { state: _, code }): Query<GiteeSignin>,
    session: &Session,
    pool: Data<&Database>,
) -> Result<impl IntoResponse> {
    info!("code: {}", code);

    // get access_token
    let access_token = gitee::get_access_token(
        code,
        CLIENT_ID.to_string(),
        CLIENT_SECRET.to_string(),
        REDIRECT_URI.to_string(),
    )
    .await?;
    info!("access_token: {}", access_token);

    // get user info
    let gitee_user = gitee::get_user_info(access_token).await?;
    info!("gitee_user: {:?}", gitee_user);

    let user = db::find_user_by_giteeid(&pool, gitee_user.id).await;
    info!("find user result: {:?}", user);
    match user {
        Ok(user) => {
            // update session
            session.set("username", user.id.to_string());
            return Ok(Response::builder()
                .status(StatusCode::FOUND)
                .header(header::LOCATION, "/")
                .finish());
        }
        Err(e) => {
            info!("error: {}", e.to_string());
            // save user if new
            if e.to_string().contains("not found") {
                info!("creating new user =====> {:?}", gitee_user);
                let nid = db::create_giteeuser(&pool, gitee_user).await?;

                session.set("username", nid);
                return Ok(Response::builder()
                    .status(StatusCode::FOUND)
                    .header(header::LOCATION, "/")
                    .finish());
            } else {
                let mut context = Context::new();
                context.insert("title", "错误");
                context.insert("msg", &e.to_string());
                let s = TEMPLATES.render("error.html", &context).unwrap();
                Ok(Html(s).into_response())
            }
        }
    }
}

#[handler]
pub fn account(session: &Session) -> impl IntoResponse {
    match session.get::<String>("username") {
        Some(username) => {
            let mut context = Context::new();
            context.insert("title", &username);
            context.insert("current_user", &username);
            let s = TEMPLATES.render("account.html", &context).unwrap();
            Html(s).into_response()
        }
        None => Response::builder()
            .status(StatusCode::FOUND)
            .header(header::LOCATION, "/signin")
            .finish(),
    }
}

#[handler]
pub fn signout(session: &Session) -> impl IntoResponse {
    session.purge();
    Response::builder()
        .status(StatusCode::FOUND)
        .header(header::LOCATION, "/signin")
        .finish()
}

#[derive(Debug, Clone, Serialize, Deserialize)]

pub struct ArticleDetailView {
    pub id: String,
    pub title: String,
    pub raw_content: String,
    pub tags: String,
    pub author_id: String,
    pub created_time: String,
    // pub updated_time: String,
    pub status: i16,
}

#[handler]
pub async fn index(_session: &Session, pool: Data<&Database>) -> impl IntoResponse {
    let articles = db::list_article(&pool).await;

    match articles {
        Ok(articles) => {
            let article_views: Vec<ArticleDetailView> =
                articles.into_iter().map(|a| a.into()).collect();
            let mut context = Context::new();
            context.insert("title", "首页");
            context.insert("article_list", &article_views);
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

impl From<Article> for ArticleDetailView {
    fn from(a: Article) -> Self {
        ArticleDetailView {
            id: a.id.to_string(),
            title: a.title,
            raw_content: a.raw_content,
            tags: a.tags,
            author_id: a.author_id.to_string(),
            created_time: a.created_time.to_string(),
            status: a.status,
        }
    }
}
#[handler]
pub async fn article_details(
    Query(FindArticle { id, title: _ }): Query<FindArticle>,
    session: &Session,
    pool: Data<&Database>,
) -> impl IntoResponse {
    let article_r = db::get_article(id.unwrap(), &pool).await;

    match article_r {
        Ok(mut article) => {
            let author = (&article).author_id.to_string();
            let mut articlev: ArticleDetailView = article.into();
            articlev.raw_content = markdown::to_html(articlev.raw_content.as_str());

            let mut context = Context::new();
            context.insert("title", &articlev.title);
            context.insert("article", &articlev);

            let username = session.get::<String>("username");
            if username.is_some() && username.unwrap() == author {
                context.insert("is_author", &true);
            }
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
    pool: Data<&Database>,
) -> impl IntoResponse {
    match session.get::<String>("username") {
        Some(username) => {
            let author_id = ObjectId::from_str(username.as_str()).unwrap();

            let mut new_article = Article::default();
            new_article.author_id = author_id;
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
    pool: Data<&Database>,
) -> impl IntoResponse {
    match session.get::<String>("username") {
        Some(_username) => {
            let article_r = db::get_article(id.unwrap(), &pool).await;

            match article_r {
                Ok(article) => {
                    let articlev: ArticleDetailView = article.into();

                    let mut context = Context::new();
                    context.insert("title", &articlev.title);
                    context.insert("article", &articlev);

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
    pool: Data<&Database>,
) -> impl IntoResponse {
    match session.get::<String>("username") {
        Some(_username) => {
            let article_r = db::get_article(params.id.clone(), &pool).await;

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
