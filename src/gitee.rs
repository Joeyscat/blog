use std::collections::HashMap;

use poem::{Error, Result};
use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize)]
struct TokenResp {
    access_token: String,
    token_type: String,
    expires_in: i64,
    refresh_token: String,
    scope: String,
    created_at: i64,
}

pub async fn get_access_token(
    code: String,
    client_id: String,
    client_secret: String,
    redirect_uri: String,
) -> Result<String> {
    let mut map = HashMap::new();
    map.insert("grant_type", "authorization_code");
    map.insert("code", code.as_str());
    map.insert("client_id", client_id.as_str());
    map.insert("client_secret", client_secret.as_str());
    map.insert("redirect_uri", redirect_uri.as_str());

    let client = reqwest::Client::new();
    let res = client
        .post("https://gitee.com/oauth/token")
        .json(&map)
        .send()
        .await
        .map_err(poem::error::InternalServerError)?;

    match res.status() {
        reqwest::StatusCode::OK => {
            let token_resp = res
                .json::<TokenResp>()
                .await
                .map_err(poem::error::InternalServerError)?;
            Ok(token_resp.access_token)
        }
        _ => {
            let t = res
                .text_with_charset("utf-8")
                .await
                .map_err(poem::error::InternalServerError)?;
            Err(Error::from_string(
                t,
                poem::http::StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: i64,
    pub login: String,
    pub name: String,
    pub avatar_url: String,
    pub blog: String,
    pub created_at: String,
    pub email: Option<String>,
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
