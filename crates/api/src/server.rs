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
    middleware::{request_id::trace_request_id, security_headers::apply_security_headers},
    openapi::ApiDoc,
    routes::{admin, auth, business, health, metrics, moderation, monetization, profile, recommendation, search, seo, taxonomy, user},
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

    let rec_routes = Router::new()
        .route("/", get(recommendation::get_recommendations));

    let billing_routes = Router::new()
        .route("/plans", get(monetization::get_plans))
        .route("/payments/{id}/verify", post(monetization::verify_payment_endpoint))
        .route("/ads/sponsored", get(monetization::get_sponsored_ads));

    let seo_routes = Router::new()
        .route("/business/{slug}", get(seo::get_business_seo_endpoint))
        .route("/landing/{city}/{category_slug}", get(seo::get_landing_page_endpoint));

    let admin_routes = Router::new()
        .route("/dashboard", get(admin::get_dashboard_endpoint))
        .route("/users", get(admin::list_users_endpoint))
        .route("/users/{id}/suspend", post(admin::suspend_user_endpoint))
        .route("/users/{id}/restore", post(admin::restore_user_endpoint))
        .route("/users/{id}/revoke-sessions", post(admin::revoke_sessions_endpoint))
        .route("/appeals", get(admin::list_appeals_endpoint))
        .route("/appeals/{id}/resolve", post(admin::resolve_appeal_endpoint))
        .route("/feature-flags", get(admin::list_flags_endpoint))
        .route("/feature-flags/{key}", put(admin::set_flag_endpoint))
        .route("/configs", get(admin::list_configs_endpoint))
        .route("/configs/{key}", put(admin::set_config_endpoint))
        .route("/audits", get(admin::get_audits_endpoint))
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
        .route("/{id}/reports", post(moderation::report_business_endpoint))
        .route("/{id}/similar", get(recommendation::get_similar_businesses))
        .route("/{id}/subscribe", post(monetization::subscribe_business_endpoint))
        .route("/{id}/campaigns", post(monetization::create_campaign_endpoint))
        .route("/{id}/appeals", post(admin::submit_appeal_endpoint));

    let mut router = Router::new()
        .route("/health/live", get(health::liveness))
        .route("/health/ready", get(health::readiness))
        .route("/metrics", get(metrics::get_prometheus_metrics))
        .route("/robots.txt", get(seo::get_robots_txt))
        .route("/sitemap.xml", get(seo::get_sitemap_index))
        .route("/sitemaps/businesses.xml", get(seo::get_businesses_sitemap))
        .nest("/api/v1/auth", auth_routes)
        .nest("/api/v1/businesses", business_routes)
        .nest("/api/v1/search", search_routes)
        .nest("/api/v1/recommendations", rec_routes)
        .nest("/api/v1/billing", billing_routes)
        .nest("/api/v1/seo", seo_routes)
        .nest("/api/v1/admin", admin_routes)
        .nest("/api/v1", taxonomy_routes)
        .nest("/api/v1", user_routes)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(middleware::from_fn(trace_request_id))
                .layer(middleware::from_fn(apply_security_headers))
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