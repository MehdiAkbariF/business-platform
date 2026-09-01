use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use domain::user::GlobalRole;
use shared::{BusinessId, CaseId, ClientMetadata};
use uuid::Uuid;

use application::use_cases::moderation::{
    approve_case, escalate_case, get_moderation_case, list_moderation_cases, reject_case,
    restore_business_cmd, start_case_review, submit_claim, submit_report, suspend_business_cmd,
    CreateReportCommand, DecisionCommand, ModerationCaseDto, SubmitClaimCommand,
};
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

fn require_moderator(principal: &AuthPrincipal) -> Result<(), ApiError> {
    match principal.0.role {
        GlobalRole::Moderator | GlobalRole::Admin | GlobalRole::SuperAdmin => Ok(()),
        _ => Err(ApiError::from(application::errors::AppError::Forbidden("Moderator permissions required".to_string()))),
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/admin/moderation/cases",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "List of moderation cases", body = Vec<ModerationCaseDto>),
        (status = 403, description = "Forbidden", body = crate::errors::ApiErrorResponse)
    )
)]
pub async fn get_cases(
    State(state): State<AppState>,
    AuthPrincipal(principal): AuthPrincipal,
) -> Result<Json<Vec<ModerationCaseDto>>, ApiError> {
    require_moderator(&AuthPrincipal(principal))?;
    let list = list_moderation_cases(state.moderation_repo.clone(), 50).await?;
    Ok(Json(list))
}

#[utoipa::path(
    get,
    path = "/api/v1/admin/moderation/cases/{id}",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Case detail", body = ModerationCaseDto),
        (status = 404, description = "Case not found", body = crate::errors::ApiErrorResponse)
    )
)]
pub async fn get_case_detail(
    State(state): State<AppState>,
    AuthPrincipal(principal): AuthPrincipal,
    Path(id): Path<Uuid>,
) -> Result<Json<ModerationCaseDto>, ApiError> {
    require_moderator(&AuthPrincipal(principal))?;
    let c = get_moderation_case(state.moderation_repo.clone(), CaseId::from_uuid(id)).await?;
    Ok(Json(c))
}

#[utoipa::path(
    post,
    path = "/api/v1/admin/moderation/cases/{id}/start",
    security(("bearer_auth" = [])),
    responses(
        (status = 204, description = "Review started"),
        (status = 409, description = "Version conflict", body = crate::errors::ApiErrorResponse)
    )
)]
pub async fn start_review_endpoint(
    State(state): State<AppState>,
    AuthPrincipal(principal): AuthPrincipal,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<StatusCode, ApiError> {
    require_moderator(&AuthPrincipal(principal.clone()))?;
    let metadata = extract_client_metadata(&headers);
    start_case_review(
        state.moderation_repo.clone(),
        state.audit_repo.clone(),
        CaseId::from_uuid(id),
        principal.user_id,
        1,
        metadata,
    ).await?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/api/v1/admin/moderation/cases/{id}/approve",
    request_body = DecisionCommand,
    security(("bearer_auth" = [])),
    responses(
        (status = 204, description = "Approved"),
        (status = 409, description = "Conflict", body = crate::errors::ApiErrorResponse)
    )
)]
pub async fn approve_endpoint(
    State(state): State<AppState>,
    AuthPrincipal(principal): AuthPrincipal,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    Json(payload): Json<DecisionCommand>,
) -> Result<StatusCode, ApiError> {
    require_moderator(&AuthPrincipal(principal.clone()))?;
    let metadata = extract_client_metadata(&headers);
    approve_case(
        state.moderation_repo.clone(),
        state.audit_repo.clone(),
        CaseId::from_uuid(id),
        principal.user_id,
        payload,
        metadata,
    ).await?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/api/v1/admin/moderation/cases/{id}/reject",
    request_body = DecisionCommand,
    security(("bearer_auth" = [])),
    responses(
        (status = 204, description = "Rejected"),
        (status = 409, description = "Conflict", body = crate::errors::ApiErrorResponse)
    )
)]
pub async fn reject_endpoint(
    State(state): State<AppState>,
    AuthPrincipal(principal): AuthPrincipal,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    Json(payload): Json<DecisionCommand>,
) -> Result<StatusCode, ApiError> {
    require_moderator(&AuthPrincipal(principal.clone()))?;
    let metadata = extract_client_metadata(&headers);
    reject_case(
        state.moderation_repo.clone(),
        state.audit_repo.clone(),
        CaseId::from_uuid(id),
        principal.user_id,
        payload,
        metadata,
    ).await?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/api/v1/admin/moderation/cases/{id}/escalate",
    request_body = DecisionCommand,
    security(("bearer_auth" = [])),
    responses(
        (status = 204, description = "Escalated"),
        (status = 409, description = "Conflict", body = crate::errors::ApiErrorResponse)
    )
)]
pub async fn escalate_endpoint(
    State(state): State<AppState>,
    AuthPrincipal(principal): AuthPrincipal,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    Json(payload): Json<DecisionCommand>,
) -> Result<StatusCode, ApiError> {
    require_moderator(&AuthPrincipal(principal.clone()))?;
    let metadata = extract_client_metadata(&headers);
    escalate_case(
        state.moderation_repo.clone(),
        state.audit_repo.clone(),
        CaseId::from_uuid(id),
        principal.user_id,
        payload,
        metadata,
    ).await?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/api/v1/admin/businesses/{id}/suspend",
    security(("bearer_auth" = [])),
    responses(
        (status = 204, description = "Business suspended"),
        (status = 403, description = "Forbidden", body = crate::errors::ApiErrorResponse)
    )
)]
pub async fn suspend_business_endpoint(
    State(state): State<AppState>,
    AuthPrincipal(principal): AuthPrincipal,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<StatusCode, ApiError> {
    require_moderator(&AuthPrincipal(principal.clone()))?;
    let metadata = extract_client_metadata(&headers);
    suspend_business_cmd(
        state.moderation_repo.clone(),
        state.audit_repo.clone(),
        BusinessId::from_uuid(id),
        principal.user_id,
        None,
        metadata,
    ).await?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/api/v1/admin/businesses/{id}/restore",
    security(("bearer_auth" = [])),
    responses(
        (status = 204, description = "Business restored"),
        (status = 403, description = "Forbidden", body = crate::errors::ApiErrorResponse)
    )
)]
pub async fn restore_business_endpoint(
    State(state): State<AppState>,
    AuthPrincipal(principal): AuthPrincipal,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<StatusCode, ApiError> {
    require_moderator(&AuthPrincipal(principal.clone()))?;
    let metadata = extract_client_metadata(&headers);
    restore_business_cmd(
        state.moderation_repo.clone(),
        state.audit_repo.clone(),
        BusinessId::from_uuid(id),
        principal.user_id,
        metadata,
    ).await?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/api/v1/businesses/{id}/claim",
    request_body = SubmitClaimCommand,
    security(("bearer_auth" = [])),
    responses(
        (status = 201, description = "Claim submitted")
    )
)]
pub async fn claim_business_endpoint(
    State(state): State<AppState>,
    AuthPrincipal(principal): AuthPrincipal,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    Json(payload): Json<SubmitClaimCommand>,
) -> Result<StatusCode, ApiError> {
    let metadata = extract_client_metadata(&headers);
    submit_claim(
        state.moderation_repo.clone(),
        state.audit_repo.clone(),
        BusinessId::from_uuid(id),
        principal.user_id,
        payload,
        metadata,
    ).await?;

    Ok(StatusCode::CREATED)
}

#[utoipa::path(
    post,
    path = "/api/v1/businesses/{id}/reports",
    request_body = CreateReportCommand,
    security(("bearer_auth" = [])),
    responses(
        (status = 201, description = "Report submitted")
    )
)]
pub async fn report_business_endpoint(
    State(state): State<AppState>,
    AuthPrincipal(principal): AuthPrincipal,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    Json(payload): Json<CreateReportCommand>,
) -> Result<StatusCode, ApiError> {
    let metadata = extract_client_metadata(&headers);
    submit_report(
        state.moderation_repo.clone(),
        state.audit_repo.clone(),
        BusinessId::from_uuid(id),
        principal.user_id,
        payload,
        metadata,
    ).await?;

    Ok(StatusCode::CREATED)
}