use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CreateUser {
    pub username: String,
}

#[derive(Deserialize)]
pub struct CreateRss {
    pub username: String,
    pub rss: String,
}

#[derive(Serialize)]
pub struct User {
    pub error_msg: String,
    pub username: String,
}

#[derive(Serialize)]
pub struct RSSResult {
    pub id: u64,
    pub rss_name: String,
    pub url: String,
}
