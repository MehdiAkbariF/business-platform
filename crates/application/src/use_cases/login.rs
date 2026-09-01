use std::sync::Arc;
use chrono::Utc;
use domain::session::Session;
use domain::user::UserStatus;
use shared::{ClientMetadata, SessionId, TokenFamilyId};
use crate::errors::AppError;
use crate::ports::repositories::{AuditRepository, SessionRepository, UserRepository};
use crate::ports::security::{AccessTokenClaims, PasswordHasherPort, TokenServicePort};
use crate::use_cases::register::AuthResponse;

pub struct LoginCommand {
    pub email: String,
    pub password: String,
    pub metadata: ClientMetadata,
}

pub async fn handle_login(
    user_repo: Arc<dyn UserRepository>,
    session_repo: Arc<dyn SessionRepository>,
    audit_repo: Arc<dyn AuditRepository>,
    hasher: Arc<dyn PasswordHasherPort>,
    token_service: Arc<dyn TokenServicePort>,
    refresh_ttl_seconds: i64,
    access_ttl_seconds: i64,
    cmd: LoginCommand,
) -> Result<AuthResponse, AppError> {
    let normalized_email = cmd.email.trim().to_lowercase();
    let generic_error = AppError::Unauthorized("Invalid email or password".to_string());

    let user = match user_repo.find_by_email(&normalized_email).await? {
        Some(u) => u,
        None => {
            let _ = audit_repo.record(
                None,
                "LOGIN_FAILED",
                cmd.metadata.ip_address,
                cmd.metadata.user_agent,
                Some(serde_json::json!({ "reason": "user_not_found" })),
            ).await;
            return Err(generic_error);
        }
    };

    if !hasher.verify_password(&cmd.password, &user.password_hash)? {
        let _ = audit_repo.record(
            Some(user.id),
            "LOGIN_FAILED",
            cmd.metadata.ip_address,
            cmd.metadata.user_agent,
            Some(serde_json::json!({ "reason": "invalid_password" })),
        ).await;
        return Err(generic_error);
    }

    match user.status {
        UserStatus::Active => {},
        UserStatus::Suspended => return Err(AppError::Forbidden("Account suspended".to_string())),
        UserStatus::Deactivated => return Err(AppError::Forbidden("Account deactivated".to_string())),
    }

    let session_id = SessionId::new();
    let family_id = TokenFamilyId::new();
    let raw_refresh_token = token_service.generate_refresh_token();
    let refresh_hash = token_service.hash_refresh_token(&raw_refresh_token);
    let expires_at = Utc::now() + chrono::Duration::seconds(refresh_ttl_seconds);

    let session = Session::new(
        session_id,
        user.id,
        family_id,
        refresh_hash,
        expires_at,
        cmd.metadata.ip_address.clone(),
        cmd.metadata.user_agent.clone(),
    );

    session_repo.create(&session).await?;
    user_repo.update_last_login(user.id, Utc::now()).await?;

    let access_token = token_service.generate_access_token(AccessTokenClaims {
        user_id: user.id,
        session_id,
        role: user.role,
    })?;

    let _ = audit_repo.record(
        Some(user.id),
        "USER_LOGGED_IN",
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