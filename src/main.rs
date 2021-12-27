use core::time;
use std::str::FromStr;

use poem::{
    get,
    listener::TcpListener,
    session::{CookieConfig, CookieSession},
    EndpointExt, Result, Route, Server,
};

use sqlx::postgres::PgPoolOptions;

mod db;
mod handler;
mod middleware;
mod model;

pub type DBPool = sqlx::postgres::PgPool;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "debug");
    }
    let lvl = tracing::Level::from_str(std::env::var("RUST_LOG").unwrap().as_str())
        .expect("请将环境变量RUST_LOG设置为可用的日志等级");
    tracing_subscriber::fmt().with_max_level(lvl).init();

    let database_uri = std::env::var("DATABASE_URL").map_err(|err| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("获取环境变量 DATABASE_URI 错误: {}", err),
        )
    })?;

    let pool: DBPool = PgPoolOptions::new()
        .max_connections(200)
        .connect_timeout(time::Duration::from_secs(2))
        .connect(database_uri.as_str())
        .await
        .map_err(|err| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("初始化数据库连接错误: {}", err),
            )
        })?;

    let app = Route::new()
        .at("/", get(handler::index))
        .at("/article", get(handler::article_details))
        .at("/signin", get(handler::signin_ui).post(handler::signin))
        .at("/logout", get(handler::logout))
        .at(
            "/article/publish",
            get(handler::publish_article_page).post(handler::publish_article),
        )
        .at(
            "/article/edit",
            get(handler::edit_article_page).post(handler::edit_article),
        )
        .with(CookieSession::new(CookieConfig::new()))
        .data(pool)
        .around(middleware::log);
    Server::new(TcpListener::bind("0.0.0.0:9527"))
        .run(app)
        .await
}
