use chrono::FixedOffset;
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

use crate::gitee;
use crate::model::Article;
use crate::{db, model::Comment};

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
pub struct GiteeSignin {
    code: String,
}

#[handler]
pub async fn gitee_signin(
    Query(GiteeSignin { code }): Query<GiteeSignin>,
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
            session.set("uid", user.id.to_string());
            session.set("username", user.username.to_string());
            return Ok(Response::builder()
                .status(StatusCode::FOUND)
                .header(header::LOCATION, "/")
                .finish());
        }
        Err(e) => {
            info!("error: {}", e.to_string());
            // save user if new
            if e.to_string().contains("not found") {
                let username = gitee_user.name.clone();
                info!("creating new user =====> {}", username);
                let nid = db::create_giteeuser(&pool, gitee_user).await?;

                session.set("uid", nid);
                session.set("username", username);
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
            context.insert("username", &username);
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
    pub author_name: String,
    pub created_time: String,
    // pub updated_time: String,
    pub status: i16,
    pub comments: Vec<CommentView>,
    pub total_comments: i32,
    pub comment_page_nums: Vec<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]

pub struct CommentView {
    pub content: String,
    pub author_id: String,
    pub author_name: String,
    pub reply_to: Option<String>,
    pub reply_to_name: Option<String>,
    pub created_time: String,
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
    comment_page: Option<i32>,
}

impl From<Article> for ArticleDetailView {
    fn from(a: Article) -> Self {
        let comments = match a.comments {
            Some(cs) => cs.into_iter().map(|c| c.into()).collect(),
            None => Vec::new(),
        };
        let mut comment_page_nums = vec![1];
        if a.total_comments.is_some() {
            let pages = (a.total_comments.unwrap() as f32 / comment_page_size() as f32).ceil() as i32;
            for i in 2..pages + 1 {
                comment_page_nums.push(i);
            }
        }
        ArticleDetailView {
            id: a.id.to_string(),
            title: a.title,
            raw_content: a.raw_content,
            tags: a.tags,
            author_id: a.author_id.to_string(),
            author_name: a.author_name.unwrap(),
            created_time: a
                .created_time
                .with_timezone(&FixedOffset::east(8 * 3600))
                .format("%Y-%m-%d %H:%M:%S")
                .to_string(),
            status: a.status,
            comments,
            total_comments: a.total_comments.unwrap(),
            comment_page_nums,
        }
    }
}

impl From<Comment> for CommentView {
    fn from(c: Comment) -> Self {
        CommentView {
            content: c.content,
            author_id: c.author_id.to_string(),
            author_name: c.author_name,
            reply_to: c.reply_to.as_ref().map(ObjectId::to_string),
            reply_to_name: c.reply_to_name,
            created_time: c
                .created_time
                .with_timezone(&FixedOffset::east(8 * 3600))
                .format("%Y-%m-%d %H:%M:%S")
                .to_string(),
            status: c.status,
        }
    }
}

#[handler]
pub async fn article_details(
    Query(FindArticle { id, comment_page }): Query<FindArticle>,
    session: &Session,
    pool: Data<&Database>,
) -> impl IntoResponse {
    let article_r =
        db::get_article(id.unwrap(), Some(comment_page_size()), comment_page, &pool).await;

    match article_r {
        Ok(mut article) => {
            let author_id = (&article).author_id.to_string();
            let mut articlev: ArticleDetailView = article.into();
            articlev.raw_content = markdown::to_html(articlev.raw_content.as_str());

            let mut context = Context::new();
            context.insert("title", &articlev.title);
            context.insert("article", &articlev);

            // 标识是否是当前用户发表的文章，如果是则提供编辑按钮等
            let uid = session.get::<String>("uid");
            if uid.is_some() && uid.unwrap() == author_id {
                context.insert("is_author", &true);
            }

            // 评论分页
            let mut cp = 1;
            if comment_page.is_some() {
                cp = comment_page.unwrap();
            }
            context.insert("comment_current_page", &cp);

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
    match session.get::<String>("uid") {
        Some(_uid) => {
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
    match session.get::<String>("uid") {
        Some(uid) => {
            let author_id = ObjectId::from_str(uid.as_str()).unwrap();

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
    Query(FindArticle {
        id,
        comment_page: _,
    }): Query<FindArticle>,
    session: &Session,
    pool: Data<&Database>,
) -> impl IntoResponse {
    match session.get::<String>("uid") {
        Some(_uid) => {
            let article_r = db::get_article(id.unwrap(), None, None, &pool).await;

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
    match session.get::<String>("uid") {
        Some(_uid) => {
            let article_r = db::get_article(params.id.clone(), None, None, &pool).await;

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

#[derive(Deserialize)]
pub struct NewCommentPageReq {
    article_id: String,
    reply_to: Option<String>,
}

#[handler]
pub async fn new_comment_page(
    Query(NewCommentPageReq {
        article_id,
        reply_to,
    }): Query<NewCommentPageReq>,
    session: &Session,
    pool: Data<&Database>,
) -> impl IntoResponse {
    match session.get::<String>("uid") {
        Some(_uid) => {
            let article_r = db::get_article(article_id, None, None, &pool).await;

            match article_r {
                Ok(article) => {
                    let articlev: ArticleDetailView = article.into();
                    let title = format!("评论: {}", &articlev.title.as_str());
                    let mut context = Context::new();
                    context.insert("title", &title);
                    context.insert("reply_to", &reply_to);
                    context.insert("article", &articlev);

                    let s = TEMPLATES.render("new_comment.html", &context).unwrap();
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
pub struct CommentArticleParams {
    reply_to: Option<String>,
    article_id: String,
    content: String,
}

#[handler]
pub async fn new_comment(
    Form(CommentArticleParams {
        reply_to,
        article_id,
        content,
    }): Form<CommentArticleParams>,
    session: &Session,
    pool: Data<&Database>,
) -> impl IntoResponse {
    match session.get::<String>("uid") {
        Some(uid) => {
            let reply_to_name = match reply_to.clone() {
                Some(to) => {
                    let u = db::find_user_by_id(to.as_str(), &pool).await.unwrap();
                    Some(u.username)
                }
                None => None,
            };
            let comment = Comment::new(
                content,
                uid,
                session.get::<String>("username").unwrap(),
                reply_to,
                reply_to_name,
            );
            let _r = db::append_comment(article_id.clone(), comment, &pool)
                .await
                .unwrap();

            Response::builder()
                .status(StatusCode::FOUND)
                .header(header::LOCATION, format!("/article?id={}", article_id))
                .finish()
        }
        None => Response::builder()
            .status(StatusCode::FOUND)
            .header(header::LOCATION, "/signin")
            .finish(),
    }
}

fn comment_page_size() ->i32{
    20
}