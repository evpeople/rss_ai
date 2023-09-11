//! Run with
//!
//! ```not_rust
//! cargo run -p example-readme
//! ```

use axum::{
    routing::{get, post},
    Router,
};
// use axum_macros::debug_handler;

use std::net::SocketAddr;

mod routes;
mod types;
use routes::{add_rss, create_user, modify_rss, root};

// mod models; // 当您有与数据库相关的逻辑时启用这一行

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
        .route("/rss", get(modify_rss))
        // `POST /users` goes to `create_user`
        .route("/rss", post(add_rss))
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
