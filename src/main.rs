//! Run with
//!
//! ```not_rust
//! cargo run -p example-readme
//! ```

use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts, State,Json},
    http::{request::Parts, StatusCode},
    routing::{get, post},
    response::IntoResponse,
    Router,
    debug_handler,
};
// use axum_macros::debug_handler;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::net::SocketAddr;
#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    let db_connection_str =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:users.db".to_string());
    let pool = sqlx::sqlite::SqlitePool::connect(&db_connection_str)
        .await
        .expect("Failed to connect to DB");
    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        .route("/rss", post(modify_rss))
        // `POST /users` goes to `create_user`
        .route("/users", post(create_user))
        .with_state(pool);

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World!"
}

async fn modify_rss(Json(payload): Json<CreateRss>) -> impl IntoResponse {
    // insert your application logic here
    let rss_reuslt = RSSResult {
        rssName: "new RSS".to_string(),
        id: 0,
        url: payload.rss,
    };

    // this will be converted into a JSON response
    // with a status code of `201 Created`
    (StatusCode::CREATED, Json(rss_reuslt))
}
#[debug_handler]
async fn create_user(
    // this argument tells axum to parse the request body
    // as JSON into a `CreateUser` type

    State(pool): State<SqlitePool>,
    Json(payload): Json<CreateUser>,
) ->  impl IntoResponse {
    // insert your application logic here
    let user = User {
        id: 123,
        username: "dasda".to_string(),
    };
    tracing::info!("inserting user into database");
    sqlx::query!("INSERT INTO users (username) VALUES (?)", user.username)
        .execute(&pool)
        .await
        .expect("Failed to insert user");
  (StatusCode::CREATED, Json(user))

}
// the input to our `create_user` handler
#[derive(Deserialize)]
struct CreateUser {
    username: String,
}

// the input to our `create_rss` handler
#[derive(Deserialize)]
struct CreateRss {
    username: String,
    rss: String,
}
// the output to our `create_user` handler
#[derive(Serialize)]
struct User {
    id: u64,
    username: String,
}
// the output to our `create_user` handler
#[derive(Serialize)]
struct RSSResult {
    id: u64,
    rssName: String,
    url: String,
}
