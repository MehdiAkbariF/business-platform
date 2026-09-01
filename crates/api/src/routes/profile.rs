use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use uuid::Uuid;

use application::use_cases::profile::{
    get_public_presentation, PublicPresentationDto,
    SaveAttributesCommand, SaveSocialLinksCommand,
};
use domain::profile::{validate_safe_url, BusinessHoursInterval};
use shared::{AttributeId, BusinessId, ClientMetadata, MediaId};
use crate::errors::ApiError;
use crate::middleware::auth::AuthPrincipal;
use crate::state::AppState;

fn extract_client_metadata(headers: &HeaderMap) -> ClientMetadata {
    let ip_address = headers
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or("").trim().to_string());

    let user_agent = headers
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    ClientMetadata {
        ip_address,
        user_agent,
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/businesses/{slug}/presentation",
    responses(
        (status = 200, description = "Full public presentation profile", body = PublicPresentationDto),
        (status = 404, description = "Business not found or unpublished", body = crate::errors::ApiErrorResponse)
    )
)]
pub async fn get_presentation(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<PublicPresentationDto>, ApiError> {
    let presentation = get_public_presentation(
        state.business_repo.clone(),
        state.taxonomy_repo.clone(),
        state.profile_repo.clone(),
        &state.config.storage_endpoint,
        &state.config.storage_bucket,
        &slug,
    ).await?;

    Ok(Json(presentation))
}

#[utoipa::path(
    put,
    path = "/api/v1/businesses/{id}/hours",
    request_body = Vec<BusinessHoursInterval>,
    security(("bearer_auth" = [])),
    responses(
        (status = 204, description = "Hours updated"),
        (status = 400, description = "Validation error", body = crate::errors::ApiErrorResponse),
        (status = 403, description = "Forbidden", body = crate::errors::ApiErrorResponse)
    )
)]
pub async fn set_business_hours(
    State(state): State<AppState>,
    AuthPrincipal(principal): AuthPrincipal,
    Path(id): Path<Uuid>,
    Json(payload): Json<Vec<BusinessHoursInterval>>,
) -> Result<StatusCode, ApiError> {
    let business_id = BusinessId::from_uuid(id);
    let membership = state.membership_repo.find_membership(business_id, principal.user_id).await?
        .ok_or_else(|| ApiError::from(application::errors::AppError::Forbidden("Not a member".to_string())))?;

    if !membership.can_edit_profile() {
        return Err(ApiError::from(application::errors::AppError::Forbidden("Insufficient permissions".to_string())));
    }

    for interval in &payload {
        interval.validate().map_err(|e| ApiError::from(application::errors::AppError::Validation(e.to_string())))?;
    }

    state.profile_repo.save_hours(business_id, &payload).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    put,
    path = "/api/v1/businesses/{id}/attributes",
    request_body = SaveAttributesCommand,
    security(("bearer_auth" = [])),
    responses(
        (status = 204, description = "Attributes updated"),
        (status = 403, description = "Forbidden", body = crate::errors::ApiErrorResponse)
    )
)]
pub async fn set_business_attributes(
    State(state): State<AppState>,
    AuthPrincipal(principal): AuthPrincipal,
    Path(id): Path<Uuid>,
    Json(payload): Json<SaveAttributesCommand>,
) -> Result<StatusCode, ApiError> {
    let business_id = BusinessId::from_uuid(id);
    let membership = state.membership_repo.find_membership(business_id, principal.user_id).await?
        .ok_or_else(|| ApiError::from(application::errors::AppError::Forbidden("Not a member".to_string())))?;

    if !membership.can_edit_profile() {
        return Err(ApiError::from(application::errors::AppError::Forbidden("Insufficient permissions".to_string())));
    }

    let tuples: Vec<(AttributeId, serde_json::Value)> = payload.attributes.into_iter().map(|a| (a.attribute_id, a.value)).collect();
    state.profile_repo.save_attributes(business_id, &tuples).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    put,
    path = "/api/v1/businesses/{id}/social-links",
    request_body = SaveSocialLinksCommand,
    security(("bearer_auth" = [])),
    responses(
        (status = 204, description = "Social links updated"),
        (status = 400, description = "Invalid URL", body = crate::errors::ApiErrorResponse),
        (status = 403, description = "Forbidden", body = crate::errors::ApiErrorResponse)
    )
)]
pub async fn set_business_social_links(
    State(state): State<AppState>,
    AuthPrincipal(principal): AuthPrincipal,
    Path(id): Path<Uuid>,
    Json(payload): Json<SaveSocialLinksCommand>,
) -> Result<StatusCode, ApiError> {
    let business_id = BusinessId::from_uuid(id);
    let membership = state.membership_repo.find_membership(business_id, principal.user_id).await?
        .ok_or_else(|| ApiError::from(application::errors::AppError::Forbidden("Not a member".to_string())))?;

    if !membership.can_edit_profile() {
        return Err(ApiError::from(application::errors::AppError::Forbidden("Insufficient permissions".to_string())));
    }

    let mut tuples = Vec::new();
    for link in payload.links {
        let valid_url = validate_safe_url(&link.url).map_err(|e| ApiError::from(application::errors::AppError::Validation(e.to_string())))?;
        tuples.push((link.platform, valid_url));
    }

    state.profile_repo.save_social_links(business_id, &tuples).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete,
    path = "/api/v1/businesses/{id}/media/{media_id}",
    security(("bearer_auth" = [])),
    responses(
        (status = 204, description = "Media item deleted"),
        (status = 403, description = "Forbidden", body = crate::errors::ApiErrorResponse)
    )
)]
pub async fn delete_media_item(
    State(state): State<AppState>,
    AuthPrincipal(principal): AuthPrincipal,
    Path((id, media_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    let business_id = BusinessId::from_uuid(id);
    let membership = state.membership_repo.find_membership(business_id, principal.user_id).await?
        .ok_or_else(|| ApiError::from(application::errors::AppError::Forbidden("Not a member".to_string())))?;

    if !membership.can_edit_profile() {
        return Err(ApiError::from(application::errors::AppError::Forbidden("Insufficient permissions".to_string())));
    }

    let m_id = MediaId::from_uuid(media_id);
    state.profile_repo.delete_media(m_id).await?;
    Ok(StatusCode::NO_CONTENT)
}