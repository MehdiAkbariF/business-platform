use axum::{
    http::StatusCode,
    middleware,
    routing::{delete, get, patch, post, put},
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
    routes::{auth, business, health, moderation, profile, search, taxonomy, user},
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

    let taxonomy_routes = Router::new()
        .route("/categories", get(taxonomy::get_categories))
        .route("/categories/{slug}", get(taxonomy::get_category))
        .route("/services", get(taxonomy::get_services))
        .route("/services/{slug}", get(taxonomy::get_service));

    let search_routes = Router::new()
        .route("/", get(search::search_businesses))
        .route("/suggestions", get(search::autocomplete_suggestions));

    let admin_routes = Router::new()
        .route("/moderation/cases", get(moderation::get_cases))
        .route("/moderation/cases/{id}", get(moderation::get_case_detail))
        .route("/moderation/cases/{id}/start", post(moderation::start_review_endpoint))
        .route("/moderation/cases/{id}/approve", post(moderation::approve_endpoint))
        .route("/moderation/cases/{id}/reject", post(moderation::reject_endpoint))
        .route("/moderation/cases/{id}/escalate", post(moderation::escalate_endpoint))
        .route("/businesses/{id}/suspend", post(moderation::suspend_business_endpoint))
        .route("/businesses/{id}/restore", post(moderation::restore_business_endpoint));

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
        )
        .route(
            "/{id}/categories",
            get(taxonomy::get_business_categories).post(taxonomy::add_business_category),
        )
        .route(
            "/{id}/categories/{category_id}",
            delete(taxonomy::remove_business_category_endpoint),
        )
        .route(
            "/{id}/categories/{category_id}/primary",
            put(taxonomy::set_primary_category_endpoint),
        )
        .route(
            "/{id}/services",
            get(taxonomy::get_business_services).post(taxonomy::add_business_service_endpoint),
        )
        .route(
            "/{id}/services/{service_id}",
            delete(taxonomy::remove_business_service_endpoint),
        )
        .route("/{slug}/presentation", get(profile::get_presentation))
        .route("/{id}/hours", put(profile::set_business_hours))
        .route("/{id}/attributes", put(profile::set_business_attributes))
        .route("/{id}/social-links", put(profile::set_business_social_links))
        .route("/{id}/media/{media_id}", delete(profile::delete_media_item))
        .route("/{id}/claim", post(moderation::claim_business_endpoint))
        .route("/{id}/reports", post(moderation::report_business_endpoint));

    let mut router = Router::new()
        .route("/health/live", get(health::liveness))
        .route("/health/ready", get(health::readiness))
        .nest("/api/v1/auth", auth_routes)
        .nest("/api/v1/businesses", business_routes)
        .nest("/api/v1/search", search_routes)
        .nest("/api/v1/admin", admin_routes)
        .nest("/api/v1", taxonomy_routes)
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