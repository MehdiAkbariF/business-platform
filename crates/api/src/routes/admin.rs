use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use domain::admin::{AdminDashboardMetrics, FeatureFlagItem, SystemConfigItem};
use domain::user::GlobalRole;
use shared::{AppealId, BusinessId, ClientMetadata, UserId};
use uuid::Uuid;

use application::ports::admin::{AdminUserDto, AuditLogEntryDto, BusinessAppealDto};
use application::use_cases::admin::{
    get_audit_trail, get_dashboard, get_feature_flags, get_system_configs, list_appeals,
    list_platform_users, resolve_appeal_cmd, restore_platform_user, revoke_user_sessions_admin,
    set_feature_flag_cmd, set_system_config_cmd, submit_business_appeal, suspend_platform_user,
    ResolveAppealCommand, SubmitAppealCommand, UpdateConfigCommand, UpdateFeatureFlagCommand,
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

fn require_admin(principal: &AuthPrincipal) -> Result<(), ApiError> {
    match principal.0.role {
        GlobalRole::Admin | GlobalRole::SuperAdmin => Ok(()),
        _ => Err(ApiError::from(application::errors::AppError::Forbidden("Admin permissions required".to_string()))),
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/admin/dashboard",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Admin dashboard metrics", body = AdminDashboardMetrics),
        (status = 403, description = "Forbidden", body = crate::errors::ApiErrorResponse)
    )
)]
pub async fn get_dashboard_endpoint(
    State(state): State<AppState>,
    AuthPrincipal(principal): AuthPrincipal,
) -> Result<Json<AdminDashboardMetrics>, ApiError> {
    require_admin(&AuthPrincipal(principal))?;
    let metrics = get_dashboard(state.admin_repo.clone()).await?;
    Ok(Json(metrics))
}

#[utoipa::path(
    get,
    path = "/api/v1/admin/users",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "List users", body = Vec<AdminUserDto>),
        (status = 403, description = "Forbidden", body = crate::errors::ApiErrorResponse)
    )
)]
pub async fn list_users_endpoint(
    State(state): State<AppState>,
    AuthPrincipal(principal): AuthPrincipal,
) -> Result<Json<Vec<AdminUserDto>>, ApiError> {
    require_admin(&AuthPrincipal(principal))?;
    let users = list_platform_users(state.admin_repo.clone(), 50, 0).await?;
    Ok(Json(users))
}

#[utoipa::path(
    post,
    path = "/api/v1/admin/users/{id}/suspend",
    security(("bearer_auth" = [])),
    responses(
        (status = 204, description = "User suspended and sessions revoked"),
        (status = 403, description = "Forbidden", body = crate::errors::ApiErrorResponse)
    )
)]
pub async fn suspend_user_endpoint(
    State(state): State<AppState>,
    AuthPrincipal(principal): AuthPrincipal,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<StatusCode, ApiError> {
    require_admin(&AuthPrincipal(principal.clone()))?;
    let metadata = extract_client_metadata(&headers);
    suspend_platform_user(
        state.admin_repo.clone(),
        state.session_repo.clone(),
        state.audit_repo.clone(),
        UserId::from_uuid(id),
        principal.user_id,
        "Administrative suspension",
        metadata,
    ).await?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/api/v1/admin/users/{id}/restore",
    security(("bearer_auth" = [])),
    responses(
        (status = 204, description = "User restored"),
        (status = 403, description = "Forbidden", body = crate::errors::ApiErrorResponse)
    )
)]
pub async fn restore_user_endpoint(
    State(state): State<AppState>,
    AuthPrincipal(principal): AuthPrincipal,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<StatusCode, ApiError> {
    require_admin(&AuthPrincipal(principal.clone()))?;
    let metadata = extract_client_metadata(&headers);
    restore_platform_user(
        state.admin_repo.clone(),
        state.audit_repo.clone(),
        UserId::from_uuid(id),
        principal.user_id,
        metadata,
    ).await?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/api/v1/admin/users/{id}/revoke-sessions",
    security(("bearer_auth" = [])),
    responses(
        (status = 204, description = "All active user sessions revoked"),
        (status = 403, description = "Forbidden", body = crate::errors::ApiErrorResponse)
    )
)]
pub async fn revoke_sessions_endpoint(
    State(state): State<AppState>,
    AuthPrincipal(principal): AuthPrincipal,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<StatusCode, ApiError> {
    require_admin(&AuthPrincipal(principal.clone()))?;
    let metadata = extract_client_metadata(&headers);
    revoke_user_sessions_admin(
        state.session_repo.clone(),
        state.audit_repo.clone(),
        UserId::from_uuid(id),
        principal.user_id,
        metadata,
    ).await?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/api/v1/businesses/{id}/appeals",
    request_body = SubmitAppealCommand,
    security(("bearer_auth" = [])),
    responses(
        (status = 201, description = "Appeal submitted")
    )
)]
pub async fn submit_appeal_endpoint(
    State(state): State<AppState>,
    AuthPrincipal(principal): AuthPrincipal,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    Json(payload): Json<SubmitAppealCommand>,
) -> Result<StatusCode, ApiError> {
    let metadata = extract_client_metadata(&headers);
    submit_business_appeal(
        state.admin_repo.clone(),
        state.audit_repo.clone(),
        BusinessId::from_uuid(id),
        principal.user_id,
        payload,
        metadata,
    ).await?;

    Ok(StatusCode::CREATED)
}

#[utoipa::path(
    get,
    path = "/api/v1/admin/appeals",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "List appeals", body = Vec<BusinessAppealDto>),
        (status = 403, description = "Forbidden", body = crate::errors::ApiErrorResponse)
    )
)]
pub async fn list_appeals_endpoint(
    State(state): State<AppState>,
    AuthPrincipal(principal): AuthPrincipal,
) -> Result<Json<Vec<BusinessAppealDto>>, ApiError> {
    require_admin(&AuthPrincipal(principal))?;
    let list = list_appeals(state.admin_repo.clone(), 50).await?;
    Ok(Json(list))
}

#[utoipa::path(
    post,
    path = "/api/v1/admin/appeals/{id}/resolve",
    request_body = ResolveAppealCommand,
    security(("bearer_auth" = [])),
    responses(
        (status = 204, description = "Appeal resolved"),
        (status = 403, description = "Forbidden", body = crate::errors::ApiErrorResponse)
    )
)]
pub async fn resolve_appeal_endpoint(
    State(state): State<AppState>,
    AuthPrincipal(principal): AuthPrincipal,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    Json(payload): Json<ResolveAppealCommand>,
) -> Result<StatusCode, ApiError> {
    require_admin(&AuthPrincipal(principal.clone()))?;
    let metadata = extract_client_metadata(&headers);
    resolve_appeal_cmd(
        state.admin_repo.clone(),
        state.audit_repo.clone(),
        AppealId::from_uuid(id),
        principal.user_id,
        payload.status,
        metadata,
    ).await?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/api/v1/admin/feature-flags",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "List feature flags", body = Vec<FeatureFlagItem>),
        (status = 403, description = "Forbidden", body = crate::errors::ApiErrorResponse)
    )
)]
pub async fn list_flags_endpoint(
    State(state): State<AppState>,
    AuthPrincipal(principal): AuthPrincipal,
) -> Result<Json<Vec<FeatureFlagItem>>, ApiError> {
    require_admin(&AuthPrincipal(principal))?;
    let flags = get_feature_flags(state.admin_repo.clone()).await?;
    Ok(Json(flags))
}

#[utoipa::path(
    put,
    path = "/api/v1/admin/feature-flags/{key}",
    request_body = UpdateFeatureFlagCommand,
    security(("bearer_auth" = [])),
    responses(
        (status = 204, description = "Flag updated"),
        (status = 403, description = "Forbidden", body = crate::errors::ApiErrorResponse)
    )
)]
pub async fn set_flag_endpoint(
    State(state): State<AppState>,
    AuthPrincipal(principal): AuthPrincipal,
    Path(key): Path<String>,
    headers: HeaderMap,
    Json(payload): Json<UpdateFeatureFlagCommand>,
) -> Result<StatusCode, ApiError> {
    require_admin(&AuthPrincipal(principal.clone()))?;
    let metadata = extract_client_metadata(&headers);
    set_feature_flag_cmd(
        state.admin_repo.clone(),
        state.audit_repo.clone(),
        &key,
        payload.is_enabled,
        principal.user_id,
        metadata,
    ).await?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/api/v1/admin/configs",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "List runtime configs", body = Vec<SystemConfigItem>),
        (status = 403, description = "Forbidden", body = crate::errors::ApiErrorResponse)
    )
)]
pub async fn list_configs_endpoint(
    State(state): State<AppState>,
    AuthPrincipal(principal): AuthPrincipal,
) -> Result<Json<Vec<SystemConfigItem>>, ApiError> {
    require_admin(&AuthPrincipal(principal))?;
    let configs = get_system_configs(state.admin_repo.clone()).await?;
    Ok(Json(configs))
}

#[utoipa::path(
    put,
    path = "/api/v1/admin/configs/{key}",
    request_body = UpdateConfigCommand,
    security(("bearer_auth" = [])),
    responses(
        (status = 204, description = "Config updated"),
        (status = 403, description = "Forbidden", body = crate::errors::ApiErrorResponse)
    )
)]
pub async fn set_config_endpoint(
    State(state): State<AppState>,
    AuthPrincipal(principal): AuthPrincipal,
    Path(key): Path<String>,
    headers: HeaderMap,
    Json(payload): Json<UpdateConfigCommand>,
) -> Result<StatusCode, ApiError> {
    require_admin(&AuthPrincipal(principal.clone()))?;
    let metadata = extract_client_metadata(&headers);
    set_system_config_cmd(
        state.admin_repo.clone(),
        state.audit_repo.clone(),
        &key,
        payload,
        principal.user_id,
        metadata,
    ).await?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/api/v1/admin/audits",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Audit trail logs", body = Vec<AuditLogEntryDto>),
        (status = 403, description = "Forbidden", body = crate::errors::ApiErrorResponse)
    )
)]
pub async fn get_audits_endpoint(
    State(state): State<AppState>,
    AuthPrincipal(principal): AuthPrincipal,
) -> Result<Json<Vec<AuditLogEntryDto>>, ApiError> {
    require_admin(&AuthPrincipal(principal))?;
    let logs = get_audit_trail(state.admin_repo.clone(), 100).await?;
    Ok(Json(logs))
}