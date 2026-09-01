use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Serialize;
use utoipa::ToSchema;
use crate::state::AppState;

#[derive(Serialize, ToSchema)]
pub struct LivenessResponse {
    pub status: &'static str,
}

#[derive(Serialize, ToSchema)]
pub struct DependencyHealth {
    pub database: bool,
    pub redis: bool,
    pub storage: bool,
}

#[derive(Serialize, ToSchema)]
pub struct ReadinessResponse {
    pub status: &'static str,
    pub checks: DependencyHealth,
}

#[utoipa::path(
    get,
    path = "/health/live",
    responses(
        (status = 200, description = "Liveness probe success", body = LivenessResponse)
    )
)]
pub async fn liveness() -> Json<LivenessResponse> {
    Json(LivenessResponse { status: "alive" })
}

#[utoipa::path(
    get,
    path = "/health/ready",
    responses(
        (status = 200, description = "All infrastructure dependencies ready", body = ReadinessResponse),
        (status = 503, description = "One or more infrastructure services unavailable", body = ReadinessResponse)
    )
)]
pub async fn readiness(State(state): State<AppState>) -> impl IntoResponse {
    let db_ok = state.db.check_health().await.is_ok();
    let redis_ok = state.redis.check_health().await.is_ok();
    let storage_ok = state.storage.check_health().await.is_ok();

    let all_ready = db_ok && redis_ok && storage_ok;

    let response = ReadinessResponse {
        status: if all_ready { "ready" } else { "unhealthy" },
        checks: DependencyHealth {
            database: db_ok,
            redis: redis_ok,
            storage: storage_ok,
        },
    };

    let status_code = if all_ready {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (status_code, Json(response))
}