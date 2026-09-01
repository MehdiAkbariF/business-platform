use std::sync::Arc;
use chrono::Utc;
use domain::session::Session;
use domain::user::UserStatus;
use shared::{ClientMetadata, SessionId};
use crate::errors::AppError;
use crate::ports::repositories::{AuditRepository, SessionRepository, UserRepository};
use crate::ports::security::{AccessTokenClaims, TokenServicePort};
use crate::use_cases::register::AuthResponse;

pub struct RefreshCommand {
    pub refresh_token: String,
    pub metadata: ClientMetadata,
}

pub async fn handle_refresh(
    user_repo: Arc<dyn UserRepository>,
    session_repo: Arc<dyn SessionRepository>,
    audit_repo: Arc<dyn AuditRepository>,
    token_service: Arc<dyn TokenServicePort>,
    refresh_ttl_seconds: i64,
    access_ttl_seconds: i64,
    cmd: RefreshCommand,
) -> Result<AuthResponse, AppError> {
    let hash = token_service.hash_refresh_token(&cmd.refresh_token);

    let session = match session_repo.find_by_refresh_token_hash_for_update(&hash).await? {
        Some(s) => s,
        None => {
            return Err(AppError::Unauthorized("Invalid or expired refresh token".to_string()));
        }
    };

    // Reuse detection: If token was already revoked
    if session.revoked_at.is_some() {
        // Severe security event: Revoke entire token family
        session_repo.revoke_token_family(session.token_family_id).await?;
        let _ = audit_repo.record(
            Some(session.user_id),
            "REFRESH_TOKEN_REUSE_DETECTED",
            cmd.metadata.ip_address,
            cmd.metadata.user_agent,
            Some(serde_json::json!({ "token_family_id": session.token_family_id.to_string() })),
        ).await;
        return Err(AppError::Unauthorized("Token reuse detected. All related sessions have been revoked.".to_string()));
    }

    if session.expires_at <= Utc::now() {
        return Err(AppError::Unauthorized("Refresh token expired".to_string()));
    }

    let user = match user_repo.find_by_id(session.user_id).await? {
        Some(u) if u.status == UserStatus::Active => u,
        _ => return Err(AppError::Forbidden("User account is not active".to_string())),
    };

    let new_session_id = SessionId::new();
    let raw_refresh_token = token_service.generate_refresh_token();
    let new_refresh_hash = token_service.hash_refresh_token(&raw_refresh_token);
    let expires_at = Utc::now() + chrono::Duration::seconds(refresh_ttl_seconds);

    let new_session = Session::new(
        new_session_id,
        user.id,
        session.token_family_id,
        new_refresh_hash,
        expires_at,
        cmd.metadata.ip_address.clone(),
        cmd.metadata.user_agent.clone(),
    );

    // Atomic rotation in single database transaction
    session_repo.rotate_token(session.id, &new_session).await?;

    let access_token = token_service.generate_access_token(AccessTokenClaims {
        user_id: user.id,
        session_id: new_session_id,
        role: user.role,
    })?;

    let _ = audit_repo.record(
        Some(user.id),
        "REFRESH_TOKEN_ROTATED",
        cmd.metadata.ip_address,
        cmd.metadata.user_agent,
        None,
    ).await;

    Ok(AuthResponse {
        user_id: user.id,
        email: user.email.as_str().to_string(),
        role: user.role.to_string(),
        access_token,
        refresh_token: raw_refresh_token,
        expires_in: access_ttl_seconds,
    })
}