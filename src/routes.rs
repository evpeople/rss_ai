use crate::types::{CreateRss, CreateUser, RssQuery, User};
use axum::{
    body::Body,
    extract::Json,
    extract::{Query, State},
    http::{Response, StatusCode},
    response::IntoResponse,
};
use chrono::{DateTime, Local};
use sqlx::SqlitePool;

// basic handler that responds with a static string
pub async fn root() -> &'static str {
    "Hello, World!"
}
pub async fn add_rss(
    State(pool): State<SqlitePool>,
    Json(payload): Json<CreateRss>,
) -> impl IntoResponse {
    let rss = &payload.rss;
    let user_name = &payload.username;
    let feed_id = sqlx::query!("select feed_id from rss_feeds where feed_url = ?", rss)
        .fetch_one(&pool)
        .await;
    let user_id = sqlx::query!("select user_id from users where username =?", user_name)
        .fetch_one(&pool)
        .await;

    let user_id = match user_id {
        Ok(user_id) => user_id.user_id,
        Err(err) => {
            tracing::error!("user_id error {}", user_name);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(User {
                    error_msg: err.to_string(),
                    username: payload.username.to_string(),
                }),
            );
        }
    };

    let feed_id = match feed_id {
        Ok(feed_id) => feed_id.feed_id,
        Err(_) => {
            let now: DateTime<Local> = Local::now();
            let formatted_time = now.format("%Y-%m-%d %H:%M:%S").to_string();
            match sqlx::query!(
                "insert into rss_feeds (feed_url,last_time,add_time)Values (?,?,?)",
                rss,
                formatted_time,
                formatted_time
            )
            .fetch_one(&pool)
            .await
            {
                Ok(_) => {
                    let last_id: (i64,) = sqlx::query_as("SELECT last_insert_rowid()")
                        .fetch_one(&pool)
                        .await
                        .expect("Failed to retrieve last insert rowid");
                    Some(last_id.0)
                }
                Err(e) => {
                    tracing::error!("fail add rss {}", e);
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(User {
                            error_msg: "RSS添加失败".to_string(),
                            username: payload.username.to_string(),
                        }),
                    );
                }
            }
        }
    };

    let user_feed = sqlx::query!(
        "select * from user_feeds where user_id=? and feed_id=?",
        user_id,
        feed_id
    )
    .fetch_one(&pool)
    .await;

    match user_feed {
        Ok(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(User {
                    error_msg: "已经添加了相应RSS".to_string(),
                    username: payload.username.to_string(),
                }),
            );
        }
        Err(_) => {
            match sqlx::query!(
                "INSERT INTO user_feeds (user_id, feed_id) VALUES (?, ?)",
                user_id,
                feed_id
            )
            .execute(&pool)
            .await
            {
                Ok(_) => (
                    StatusCode::CREATED,
                    Json(User {
                        error_msg: "添加成功".to_string(),
                        username: payload.username,
                    }),
                ),
                Err(_e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(User {
                        error_msg: "添加RSS 失败".to_string(),
                        username: "".to_string(),
                    }),
                ),
            }
        }
    }
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
    match sqlx::query!("INSERT INTO users (username) VALUES (?)", user.username)
        .execute(&pool)
        .await
    {
        Ok(_) => (StatusCode::CREATED, Json(user)),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(User {
                error_msg: ("创建用户失败").to_string(),
                username: "".to_string(),
            }),
        ),
    }
}
pub async fn modify_rss(query: Query<RssQuery>) -> impl IntoResponse {
    tracing::error!(" the rss_url 是{}", &query.rss_url);
    let rss_content = reqwest::get(&query.rss_url).await;
    // let rss_content = reqwest::get("https://www.bbc.com/zhongwen/simp/index.xml").await;
    let rss_content = match rss_content {
        Ok(rss_content) => match rss_content.text().await {
            Ok(rss_content) => rss_content,
            Err(e) => {
                return Response::builder()
                    .status(StatusCode::OK)
                    .body(Body::from(
                        serde_json::to_string(&User {
                            error_msg: format!(
                                "读取rss响应 {} 失败, error_msg是 {}",
                                query.rss_url,
                                e.to_string()
                            ),
                            username: "".to_string(),
                        })
                        .unwrap_or("json序列化失败".to_string()),
                    ))
                    .unwrap();
            }
        },
        Err(e) => {
            return Response::builder()
                .status(StatusCode::OK)
                .body(Body::from(
                    serde_json::to_string(&User {
                        error_msg: format!(
                            "抓取rss {} 失败, error_msg是 {}",
                            query.rss_url,
                            e.to_string()
                        ),
                        username: "".to_string(),
                    })
                    .unwrap_or("json序列化失败".to_string()),
                ))
                .unwrap();
        }
    };

    let channel = rss::Channel::read_from(rss_content.as_bytes());
    let channel = match channel {
        Ok(channel) => channel,
        Err(e) => {
            return Response::builder()
                .status(StatusCode::OK)
                .body(Body::from(
                    serde_json::to_string(&User {
                        error_msg: format!(
                            "解析 rss {} 失败, error_msg是 {}",
                            query.rss_url,
                            e.to_string()
                        ),
                        username: "".to_string(),
                    })
                    .unwrap(),
                ))
                .unwrap();
        }
    };
    // 修改 RSS 数据
    let modified_items: Vec<rss::Item> = channel
        .items()
        .iter()
        .map(|item| {
            let mut new_item = item.clone();
            new_item.set_title(format!("Modified: {}", item.title().unwrap_or_default()));
            new_item
        })
        .collect();
    let mut modified_channel = channel.clone();
    modified_channel.set_items(modified_items);
    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(modified_channel.to_string()))
        .unwrap()
}
