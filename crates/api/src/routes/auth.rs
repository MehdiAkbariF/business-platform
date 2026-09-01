use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use application::use_cases::{
    login::{handle_login, LoginCommand},
    logout::{handle_logout, handle_logout_all},
    refresh::{handle_refresh, RefreshCommand},
    register::{handle_register, AuthResponse, RegisterCommand},
};
use shared::{ClientMetadata, UserId};
use crate::errors::ApiError;
use crate::middleware::auth::AuthPrincipal;
use crate::state::AppState;

#[derive(Deserialize, ToSchema)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize, ToSchema)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize, ToSchema)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Serialize, ToSchema)]
pub struct AuthSuccessResponse {
    pub user_id: UserId,
    pub email: String,
    pub role: String,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}

impl From<AuthResponse> for AuthSuccessResponse {
    fn from(r: AuthResponse) -> Self {
        Self {
            user_id: r.user_id,
            email: r.email,
            role: r.role,
            access_token: r.access_token,
            refresh_token: r.refresh_token,
            expires_in: r.expires_in,
        }
    }
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
    path = "/api/v1/auth/register",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "Registration successful", body = AuthSuccessResponse),
        (status = 400, description = "Validation error", body = crate::errors::ApiErrorResponse),
        (status = 409, description = "Email conflict", body = crate::errors::ApiErrorResponse)
    )
)]
pub async fn register(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<AuthSuccessResponse>), ApiError> {
    let metadata = extract_client_metadata(&headers);
    let ip = metadata.ip_address.clone().unwrap_or_else(|| "unknown".to_string());

    if !state.rate_limiter.check_rate_limit(&format!("reg:{}", ip), 10, 60).await? {
        return Err(ApiError::from(application::errors::AppError::RateLimited("Too many registration requests".to_string())));
    }

    let response = handle_register(
        state.user_repo.clone(),
        state.session_repo.clone(),
        state.audit_repo.clone(),
        state.password_hasher.clone(),
        state.token_service.clone(),
        state.config.refresh_token_ttl_seconds,
        state.config.access_token_ttl_seconds,
        RegisterCommand {
            email: payload.email,
            password: payload.password,
            metadata,
        },
    ).await?;

    Ok((StatusCode::CREATED, Json(response.into())))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Authentication successful", body = AuthSuccessResponse),
        (status = 401, description = "Invalid credentials", body = crate::errors::ApiErrorResponse),
        (status = 403, description = "Account suspended or deactivated", body = crate::errors::ApiErrorResponse)
    )
)]
pub async fn login(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthSuccessResponse>, ApiError> {
    let metadata = extract_client_metadata(&headers);
    let ip = metadata.ip_address.clone().unwrap_or_else(|| "unknown".to_string());

    if !state.rate_limiter.check_rate_limit(&format!("login:{}", ip), 15, 60).await? {
        return Err(ApiError::from(application::errors::AppError::RateLimited("Too many login attempts".to_string())));
    }

    let response = handle_login(
        state.user_repo.clone(),
        state.session_repo.clone(),
        state.audit_repo.clone(),
        state.password_hasher.clone(),
        state.token_service.clone(),
        state.config.refresh_token_ttl_seconds,
        state.config.access_token_ttl_seconds,
        LoginCommand {
            email: payload.email,
            password: payload.password,
            metadata,
        },
    ).await?;

    Ok(Json(response.into()))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/refresh",
    request_body = RefreshRequest,
    responses(
        (status = 200, description = "Token rotation successful", body = AuthSuccessResponse),
        (status = 401, description = "Invalid, expired, or reused token", body = crate::errors::ApiErrorResponse)
    )
)]
pub async fn refresh(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<RefreshRequest>,
) -> Result<Json<AuthSuccessResponse>, ApiError> {
    let metadata = extract_client_metadata(&headers);
    let ip = metadata.ip_address.clone().unwrap_or_else(|| "unknown".to_string());

    if !state.rate_limiter.check_rate_limit(&format!("refresh:{}", ip), 30, 60).await? {
        return Err(ApiError::from(application::errors::AppError::RateLimited("Too many refresh requests".to_string())));
    }

    let response = handle_refresh(
        state.user_repo.clone(),
        state.session_repo.clone(),
        state.audit_repo.clone(),
        state.token_service.clone(),
        state.config.refresh_token_ttl_seconds,
        state.config.access_token_ttl_seconds,
        RefreshCommand {
            refresh_token: payload.refresh_token,
            metadata,
        },
    ).await?;

    Ok(Json(response.into()))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/logout",
    security(("bearer_auth" = [])),
    responses(
        (status = 204, description = "Current session successfully revoked"),
        (status = 401, description = "Unauthorized", body = crate::errors::ApiErrorResponse)
    )
)]
pub async fn logout(
    State(state): State<AppState>,
    AuthPrincipal(principal): AuthPrincipal,
    headers: HeaderMap,
) -> Result<StatusCode, ApiError> {
    let metadata = extract_client_metadata(&headers);
    handle_logout(
        state.session_repo.clone(),
        state.audit_repo.clone(),
        principal.user_id,
        principal.session_id,
        metadata,
    ).await?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/logout-all",
    security(("bearer_auth" = [])),
    responses(
        (status = 204, description = "All active sessions revoked"),
        (status = 401, description = "Unauthorized", body = crate::errors::ApiErrorResponse)
    )
)]
pub async fn logout_all(
    State(state): State<AppState>,
    AuthPrincipal(principal): AuthPrincipal,
    headers: HeaderMap,
) -> Result<StatusCode, ApiError> {
    let metadata = extract_client_metadata(&headers);
    handle_logout_all(
        state.session_repo.clone(),
        state.audit_repo.clone(),
        principal.user_id,
        metadata,
    ).await?;

    Ok(StatusCode::NO_CONTENT)
}