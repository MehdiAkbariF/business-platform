use axum::{
    http::StatusCode,
    middleware,
    routing::{delete, get, patch, post},
    Router,
};
use std::time::Duration;
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    middleware::request_id::trace_request_id,
    openapi::ApiDoc,
    routes::{auth, business, health, user},
    state::AppState,
};

pub fn build_router(state: AppState) -> Router {
    let is_dev = state.config.app_env == "development";

    let cors = if is_dev {
        CorsLayer::permissive()
    } else {
        CorsLayer::new()
    };

    let auth_routes = Router::new()
        .route("/register", post(auth::register))
        .route("/login", post(auth::login))
        .route("/refresh", post(auth::refresh))
        .route("/logout", post(auth::logout))
        .route("/logout-all", post(auth::logout_all));

    let user_routes = Router::new()
        .route("/me", get(user::get_me));

    let business_routes = Router::new()
        .route("/", post(business::create))
        .route("/manage/{id}", get(business::get_management))
        .route(
            "/{id}",
            get(business::get_public).patch(business::update_profile),
        )
        .route("/{id}/submit", post(business::submit))
        .route("/{id}/archive", post(business::archive))
        .route(
            "/{id}/members",
            get(business::get_members).post(business::add_business_member),
        )
        .route(
            "/{id}/members/{user_id}",
            delete(business::remove_business_member),
        )
        .route(
            "/{id}/members/{user_id}/role",
            patch(business::change_role),
        );

    let mut router = Router::new()
        .route("/health/live", get(health::liveness))
        .route("/health/ready", get(health::readiness))
        .nest("/api/v1/auth", auth_routes)
        .nest("/api/v1/businesses", business_routes)
        .nest("/api/v1", user_routes)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(middleware::from_fn(trace_request_id))
                .layer(TimeoutLayer::with_status_code(
                    StatusCode::REQUEST_TIMEOUT,
                    Duration::from_secs(30),
                ))
                .layer(cors),
        )
        .with_state(state);

    if is_dev {
        router = router.merge(
            SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()),
        );
    }

    router
}

pub async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C signal handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received Ctrl+C, starting graceful shutdown");
        },
        _ = terminate => {
            tracing::info!("Received SIGTERM, starting graceful shutdown");
        },
    }
}