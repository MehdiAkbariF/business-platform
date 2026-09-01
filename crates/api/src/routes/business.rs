use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;

use application::use_cases::business::{
    archive_business, create_business, get_management_profile, get_public_profile, submit_business,
    update_business_profile, BusinessDto, CreateBusinessCommand, PublicBusinessProfileDto,
};
use application::use_cases::members::{
    add_member, change_member_role, list_members, remove_member, AddMemberCommand, ChangeRoleCommand, MemberDto,
};
use shared::{BusinessId, ClientMetadata, UserId};
use crate::errors::ApiError;
use crate::middleware::auth::AuthPrincipal;
use crate::state::AppState;

#[derive(Deserialize, ToSchema)]
pub struct UpdateProfileRequest {
    pub name: String,
    pub description: Option<String>,
}

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
    post,
    path = "/api/v1/businesses",
    request_body = CreateBusinessCommand,
    security(("bearer_auth" = [])),
    responses(
        (status = 201, description = "Business created successfully", body = BusinessDto),
        (status = 400, description = "Validation error", body = crate::errors::ApiErrorResponse),
        (status = 409, description = "Slug conflict", body = crate::errors::ApiErrorResponse)
    )
)]
pub async fn create(
    State(state): State<AppState>,
    AuthPrincipal(principal): AuthPrincipal,
    headers: HeaderMap,
    Json(payload): Json<CreateBusinessCommand>,
) -> Result<(StatusCode, Json<BusinessDto>), ApiError> {
    let metadata = extract_client_metadata(&headers);
    let dto = create_business(
        state.business_repo.clone(),
        state.audit_repo.clone(),
        principal.user_id,
        payload,
        metadata,
    ).await?;

    Ok((StatusCode::CREATED, Json(dto)))
}

#[utoipa::path(
    get,
    path = "/api/v1/businesses/{slug}",
    responses(
        (status = 200, description = "Public business profile", body = PublicBusinessProfileDto),
        (status = 404, description = "Business not found or not published", body = crate::errors::ApiErrorResponse)
    )
)]
pub async fn get_public(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<PublicBusinessProfileDto>, ApiError> {
    let profile = get_public_profile(state.business_repo.clone(), &slug).await?;
    Ok(Json(profile))
}

#[utoipa::path(
    get,
    path = "/api/v1/businesses/manage/{id}",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Management business profile", body = BusinessDto),
        (status = 403, description = "Forbidden", body = crate::errors::ApiErrorResponse)
    )
)]
pub async fn get_management(
    State(state): State<AppState>,
    AuthPrincipal(principal): AuthPrincipal,
    Path(id): Path<Uuid>,
) -> Result<Json<BusinessDto>, ApiError> {
    let business_id = BusinessId::from_uuid(id);
    let profile = get_management_profile(
        state.business_repo.clone(),
        state.membership_repo.clone(),
        business_id,
        principal.user_id,
    ).await?;

    Ok(Json(profile))
}

#[utoipa::path(
    patch,
    path = "/api/v1/businesses/{id}",
    request_body = UpdateProfileRequest,
    security(("bearer_auth" = [])),
    responses(
        (status = 204, description = "Profile updated successfully"),
        (status = 403, description = "Forbidden", body = crate::errors::ApiErrorResponse)
    )
)]
pub async fn update_profile(
    State(state): State<AppState>,
    AuthPrincipal(principal): AuthPrincipal,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    Json(payload): Json<UpdateProfileRequest>,
) -> Result<StatusCode, ApiError> {
    let metadata = extract_client_metadata(&headers);
    let business_id = BusinessId::from_uuid(id);
    update_business_profile(
        state.business_repo.clone(),
        state.membership_repo.clone(),
        state.audit_repo.clone(),
        business_id,
        principal.user_id,
        payload.name,
        payload.description,
        metadata,
    ).await?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/api/v1/businesses/{id}/submit",
    security(("bearer_auth" = [])),
    responses(
        (status = 204, description = "Business submitted for review"),
        (status = 400, description = "Missing required data", body = crate::errors::ApiErrorResponse),
        (status = 403, description = "Forbidden", body = crate::errors::ApiErrorResponse)
    )
)]
pub async fn submit(
    State(state): State<AppState>,
    AuthPrincipal(principal): AuthPrincipal,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<StatusCode, ApiError> {
    let metadata = extract_client_metadata(&headers);
    let business_id = BusinessId::from_uuid(id);
    submit_business(
        state.business_repo.clone(),
        state.membership_repo.clone(),
        state.taxonomy_repo.clone(),
        state.audit_repo.clone(),
        business_id,
        principal.user_id,
        metadata,
    ).await?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/api/v1/businesses/{id}/archive",
    security(("bearer_auth" = [])),
    responses(
        (status = 204, description = "Business archived"),
        (status = 403, description = "Forbidden", body = crate::errors::ApiErrorResponse)
    )
)]
pub async fn archive(
    State(state): State<AppState>,
    AuthPrincipal(principal): AuthPrincipal,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<StatusCode, ApiError> {
    let metadata = extract_client_metadata(&headers);
    let business_id = BusinessId::from_uuid(id);
    archive_business(
        state.business_repo.clone(),
        state.membership_repo.clone(),
        state.audit_repo.clone(),
        business_id,
        principal.user_id,
        metadata,
    ).await?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/api/v1/businesses/{id}/members",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Members list", body = Vec<MemberDto>),
        (status = 403, description = "Forbidden", body = crate::errors::ApiErrorResponse)
    )
)]
pub async fn get_members(
    State(state): State<AppState>,
    AuthPrincipal(principal): AuthPrincipal,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<MemberDto>>, ApiError> {
    let business_id = BusinessId::from_uuid(id);
    let members = list_members(state.membership_repo.clone(), business_id, principal.user_id).await?;
    Ok(Json(members))
}

#[utoipa::path(
    post,
    path = "/api/v1/businesses/{id}/members",
    request_body = AddMemberCommand,
    security(("bearer_auth" = [])),
    responses(
        (status = 201, description = "Member added"),
        (status = 403, description = "Forbidden", body = crate::errors::ApiErrorResponse),
        (status = 409, description = "Member already exists", body = crate::errors::ApiErrorResponse)
    )
)]
pub async fn add_business_member(
    State(state): State<AppState>,
    AuthPrincipal(principal): AuthPrincipal,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    Json(payload): Json<AddMemberCommand>,
) -> Result<StatusCode, ApiError> {
    let metadata = extract_client_metadata(&headers);
    let business_id = BusinessId::from_uuid(id);
    add_member(
        state.user_repo.clone(),
        state.membership_repo.clone(),
        state.audit_repo.clone(),
        business_id,
        principal.user_id,
        payload,
        metadata,
    ).await?;

    Ok(StatusCode::CREATED)
}

#[utoipa::path(
    delete,
    path = "/api/v1/businesses/{id}/members/{user_id}",
    security(("bearer_auth" = [])),
    responses(
        (status = 204, description = "Member removed"),
        (status = 403, description = "Forbidden", body = crate::errors::ApiErrorResponse)
    )
)]
pub async fn remove_business_member(
    State(state): State<AppState>,
    AuthPrincipal(principal): AuthPrincipal,
    Path((id, target_user_id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> Result<StatusCode, ApiError> {
    let metadata = extract_client_metadata(&headers);
    let business_id = BusinessId::from_uuid(id);
    let target_id = UserId::from_uuid(target_user_id);
    remove_member(
        state.membership_repo.clone(),
        state.audit_repo.clone(),
        business_id,
        principal.user_id,
        target_id,
        metadata,
    ).await?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    patch,
    path = "/api/v1/businesses/{id}/members/{user_id}/role",
    request_body = ChangeRoleCommand,
    security(("bearer_auth" = [])),
    responses(
        (status = 204, description = "Member role changed"),
        (status = 403, description = "Forbidden", body = crate::errors::ApiErrorResponse)
    )
)]
pub async fn change_role(
    State(state): State<AppState>,
    AuthPrincipal(principal): AuthPrincipal,
    Path((id, target_user_id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
    Json(payload): Json<ChangeRoleCommand>,
) -> Result<StatusCode, ApiError> {
    let metadata = extract_client_metadata(&headers);
    let business_id = BusinessId::from_uuid(id);
    let target_id = UserId::from_uuid(target_user_id);
    change_member_role(
        state.membership_repo.clone(),
        state.audit_repo.clone(),
        business_id,
        principal.user_id,
        target_id,
        payload,
        metadata,
    ).await?;

    Ok(StatusCode::NO_CONTENT)
}