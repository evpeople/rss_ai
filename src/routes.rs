use crate::types::{CreateRss, CreateUser, RSSResult, User};
use axum::{extract::Json, extract::State, http::StatusCode, response::IntoResponse};
use sqlx::SqlitePool;

// basic handler that responds with a static string
pub async fn root() -> &'static str {
    "Hello, World!"
}

pub async fn modify_rss(Json(payload): Json<CreateRss>) -> impl IntoResponse {
    // insert your application logic here
    let rss_result = RSSResult {
        rss_name: "new RSS".to_string(),
        id: 0,
        url: payload.rss,
    };

    // this will be converted into a JSON response
    // with a status code of `201 Created`
    (StatusCode::CREATED, Json(rss_result))
}
// #[debug_handler]
pub async fn create_user(
    // this argument tells axum to parse the request body
    // as JSON into a `CreateUser` type
    State(pool): State<SqlitePool>,
    Json(payload): Json<CreateUser>,
) -> impl IntoResponse {
    // insert your application logic here
    let user = User {
        error_msg: "创建用户成功".to_string(),
        username: payload.username,
    };
    tracing::info!("inserting user into database");
    sqlx::query!("INSERT INTO users (username) VALUES (?)", user.username)
        .execute(&pool)
        .await
        .map_or((StatusCode::CREATED, Json(user)), |_e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(User {
                    error_msg: ("创建用户失败").to_string(),
                    username: "".to_string(),
                }),
            )
        })
}
