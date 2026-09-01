use api::server::build_router;
use api::state::AppState;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;
use std::sync::Arc;
use infrastructure::config::AppConfig;

#[tokio::test]
async fn test_liveness_endpoint_returns_ok() {
    let mock_config = AppConfig {
        app_env: "test".to_string(),
        server_host: "127.0.0.1".to_string(),
        server_port: 8080,
        database_url: "postgres://mock".to_string(),
        database_max_connections: 1,
        database_min_connections: 1,
        redis_url: "redis://mock".to_string(),
        storage_endpoint: "http://mock".to_string(),
        storage_bucket: "test-bucket".to_string(),
        storage_access_key: "key".to_string(),
        storage_secret_key: "secret".to_string(),
        storage_region: "us-east-1".to_string(),
        cors_allowed_origins: vec!["*".to_string()],
    };

    // Construct mock / test harness router
    let app = axum::Router::new().route("/health/live", axum::routing::get(api::routes::health::liveness));

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health/live")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}