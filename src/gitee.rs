use std::collections::HashMap;

use poem::Result;
use serde_derive::{Deserialize, Serialize};

pub async fn get_access_token(code: String) -> Result<String> {
    let mut map = HashMap::new();
    map.insert("grant_type", "rust");
    map.insert("code", "json");
    map.insert("client_id", "json");
    map.insert("client_secret", "json");
    map.insert("redirect_uri", "json");

    let res = reqwest::Client::new().post("").body(&map).send().await?;

    Ok("x".to_string())
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
    todo!()
}
