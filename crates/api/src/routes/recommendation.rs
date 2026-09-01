use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderMap},
    Json,
};
use uuid::Uuid;

use application::ports::recommendation::RecommendationResponseDto;
use application::use_cases::recommendation::{
    execute_recommendations, execute_similar_businesses, RecommendationRequestQuery,
};
use shared::BusinessId;
use crate::errors::ApiError;
use crate::state::AppState;

#[utoipa::path(
    get,
    path = "/api/v1/recommendations",
    params(
        RecommendationRequestQuery
    ),
    responses(
        (status = 200, description = "Discovery recommendations", body = RecommendationResponseDto),
        (status = 400, description = "Validation error", body = crate::errors::ApiErrorResponse)
    )
)]
pub async fn get_recommendations(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<RecommendationRequestQuery>,
) -> Result<Json<RecommendationResponseDto>, ApiError> {
    // Extract user_id if valid Bearer token exists, otherwise treat as anonymous discovery
    let user_id = headers
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .and_then(|token| state.token_service.verify_access_token(token).ok())
        .map(|claims| claims.user_id);

    let resp = execute_recommendations(
        state.rec_engine.clone(),
        state.ranking_engine.clone(),
        state.search_port.clone(),
        user_id,
        query,
    ).await?;

    Ok(Json(resp))
}

#[utoipa::path(
    get,
    path = "/api/v1/businesses/{id}/similar",
    responses(
        (status = 200, description = "Similar businesses", body = RecommendationResponseDto),
        (status = 404, description = "Business not found", body = crate::errors::ApiErrorResponse)
    )
)]
pub async fn get_similar_businesses(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<RecommendationResponseDto>, ApiError> {
    let business_id = BusinessId::from_uuid(id);
    let resp = execute_similar_businesses(
        state.rec_engine.clone(),
        state.ranking_engine.clone(),
        state.search_port.clone(),
        business_id,
        6,
    ).await?;

    Ok(Json(resp))
}