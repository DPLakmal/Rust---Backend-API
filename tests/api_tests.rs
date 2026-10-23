use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use sqlx::sqlite::SqlitePoolOptions;
use tower::ServiceExt;

use rust_backend_api::{
    config::Config,
    routes::create_router,
    state::AppState,
};

async fn setup_test_app() -> axum::Router {
    // In-memory SQLite for testing
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .expect("Failed to connect to in-memory SQLite");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    let config = Config {
        host: "127.0.0.1".to_string(),
        port: 3000,
        database_url: "sqlite::memory:".to_string(),
        api_key: "test-secret-key".to_string(),
    };

    let state = AppState::new(pool, config);
    create_router(state)
}

#[tokio::test]
async fn test_health_check() {
    let app = setup_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(body_json["status"], "healthy");
    assert_eq!(body_json["database"], "connected");
}

#[tokio::test]
async fn test_auth_guard_blocks_unauthorized() {
    let app = setup_test_app().await;

    // Call protected endpoint without token
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/items")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_crud_lifecycle() {
    let app = setup_test_app().await;

    // 1. Create item
    let create_payload = json!({ "title": "Buy groceries" }).to_string();
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/items")
                .header(header::AUTHORIZATION, "Bearer test-secret-key")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(create_payload))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::CREATED);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let created_item: Value = serde_json::from_slice(&body).unwrap();
    let item_id = created_item["id"].as_str().unwrap().to_string();
    assert_eq!(created_item["title"], "Buy groceries");
    assert_eq!(created_item["completed"], false);

    // 2. Fetch created item by ID
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/items/{}", item_id))
                .header(header::AUTHORIZATION, "Bearer test-secret-key")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    // 3. Update item
    let update_payload = json!({ "completed": true }).to_string();
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/items/{}", item_id))
                .header(header::AUTHORIZATION, "Bearer test-secret-key")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(update_payload))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let updated: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(updated["completed"], true);

    // 4. Delete item
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/items/{}", item_id))
                .header(header::AUTHORIZATION, "Bearer test-secret-key")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    // 5. Verify deleted item returns 404
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/items/{}", item_id))
                .header(header::AUTHORIZATION, "Bearer test-secret-key")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}
