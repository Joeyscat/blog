use std::collections::HashMap;

use poem::{Error, Result};
use serde_derive::{Deserialize, Serialize};

// #[derive(Debug, Serialize, Deserialize)]
// struct TokenReq {
//     grant_type: String,
//     code: String,
//     client_id: String,
//     client_secret: String,
//     redirect_uri: String,
// }

#[derive(Deserialize)]
struct TokenResp {
    access_token: String,
    token_type: String,
    expires_in: i64,
    refresh_token: String,
    scope: String,
    created_at: i64,
}

pub async fn get_access_token(code: String) -> Result<String> {
    let mut map = HashMap::new();
    map.insert("grant_type", "rust");
    map.insert("code", "json");
    map.insert("client_id", "json");
    map.insert("client_secret", "json");
    map.insert("redirect_uri", "json");

    let client = reqwest::Client::new();
    let res = client
        .post("https://gitee.com/oauth/token")
        .json(&map)
        .send()
        .await
        .map_err(poem::error::InternalServerError)?
        .json::<TokenResp>()
        .await
        .map_err(poem::error::InternalServerError)?;

    Ok(res.access_token)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: i64,
    pub login: String,
    pub name: String,
    pub avatar_url: String,
    pub blog: String,
    pub created_at: String,
    pub email: String,
}

pub async fn get_user_info(access_token: String) -> Result<UserInfo> {
    let user = reqwest::get(format!(
        "https://gitee.com/api/v5/user?access_token={}",
        access_token
    ))
    .await
    .map_err(poem::error::InternalServerError)?
    .json::<UserInfo>()
    .await
    .map_err(poem::error::InternalServerError)?;

    Ok(user)
}
