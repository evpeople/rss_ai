// extern crate rss_ai;
#[cfg(test)]
mod tests {
    use axum::{
        routing::{get, post},
        Router,
    };
    // use axum_macros::debug_handler;

    use reqwest::Client;
    use rss_ai::{routes::*, types::CreateUser};
    use sqlx::sqlite::SqlitePool;
    use std::net::TcpListener;

    #[tokio::test]
    async fn integration_test_create_user() {
        // 1. Setup environment
        let listener = TcpListener::bind("127.0.0.1:0")
            // .await
            .expect("Bind to a random port");
        let port = listener.local_addr().unwrap().port();
        let database_url = "./test.db"; // Using an in-memory database for testing
        let pool = SqlitePool::connect(&database_url)
            .await
            .expect("Database setup");

        // 2. Start server
        tokio::spawn(async move {
            let app: Router = Router::new()
                // `GET /` goes to `root`
                .route("/", get(root))
                .route("/rss", get(modify_rss))
                // `POST /users` goes to `create_user`
                .route("/rss", post(add_rss))
                .route("/users", post(create_user))
                .with_state(pool);
            axum::Server::from_tcp(listener)
                .unwrap()
                .serve(app.into_make_service())
                .await
                .unwrap();
        });

        let client = Client::new();

        // 3. Send request
        let user = CreateUser {
            username: "test_user".to_string(),
        };
        let response = client
            .post(format!("http://127.0.0.1:{}/users", port))
            .json(&user)
            .send()
            .await
            .expect("Send request");

        // 4. Validate response
        assert_eq!(response.status(), reqwest::StatusCode::CREATED);
        let body = response.text().await.expect("Read response body");
        assert!(body.contains("创建用户成功"));

        // 5. Cleanup if necessary, server will automatically stop once test finishes
    }

    #[tokio::test]
    async fn integration_test_create_rss() {
        // 1. Setup environment
        let listener = TcpListener::bind("127.0.0.1:0")
            // .await
            .expect("Bind to a random port");
        let port = listener.local_addr().unwrap().port();
        let database_url = "./test.db"; // Using an in-memory database for testing
        let pool = SqlitePool::connect(&database_url)
            .await
            .expect("Database setup");

        // 2. Start server
        tokio::spawn(async move {
            let app: Router = Router::new()
                // `GET /` goes to `root`
                .route("/", get(root))
                .route("/rss", get(modify_rss))
                // `POST /users` goes to `create_user`
                .route("/rss", post(add_rss))
                .route("/users", post(create_user))
                .with_state(pool);
            axum::Server::from_tcp(listener)
                .unwrap()
                .serve(app.into_make_service())
                .await
                .unwrap();
        });

        let client = Client::new();

        // 3. Send request
        let user = CreateUser {
            username: "test_user".to_string(),
        };
        let response = client
            .post(format!("http://127.0.0.1:{}/users", port))
            .json(&user)
            .send()
            .await
            .expect("Send request");

        // 4. Validate response
        assert_eq!(response.status(), reqwest::StatusCode::CREATED);
        let body = response.text().await.expect("Read response body");
        assert!(body.contains("创建用户成功"));

        // 5. Cleanup if necessary, server will automatically stop once test finishes
    }
}
