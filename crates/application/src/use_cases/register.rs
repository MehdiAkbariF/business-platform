use std::sync::Arc;
use chrono::Utc;
use domain::user::{Email, User};
use domain::session::Session;
use shared::{ClientMetadata, SessionId, TokenFamilyId, UserId};
use crate::errors::AppError;
use crate::ports::repositories::{AuditRepository, SessionRepository, UserRepository};
use crate::ports::security::{AccessTokenClaims, PasswordHasherPort, TokenServicePort};

pub struct RegisterCommand {
    pub email: String,
    pub password: String,
    pub metadata: ClientMetadata,
}

pub struct AuthResponse {
    pub user_id: UserId,
    pub email: String,
    pub role: String,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}

pub async fn handle_register(
    user_repo: Arc<dyn UserRepository>,
    session_repo: Arc<dyn SessionRepository>,
    audit_repo: Arc<dyn AuditRepository>,
    hasher: Arc<dyn PasswordHasherPort>,
    token_service: Arc<dyn TokenServicePort>,
    refresh_ttl_seconds: i64,
    access_ttl_seconds: i64,
    cmd: RegisterCommand,
) -> Result<AuthResponse, AppError> {
    let email = Email::parse(&cmd.email).map_err(|e| AppError::Validation(e.to_string()))?;
    
    if cmd.password.len() < 8 {
        return Err(AppError::Validation("Password must be at least 8 characters long".to_string()));
    }
    if cmd.password.len() > 128 {
        return Err(AppError::Validation("Password must not exceed 128 characters".to_string()));
    }

    if let Some(_) = user_repo.find_by_email(email.as_str()).await? {
        return Err(AppError::Conflict("An account with this email address already exists".to_string()));
    }

    let password_hash = hasher.hash_password(&cmd.password)?;
    let user_id = UserId::new();
    let user = User::new(user_id, email.clone(), password_hash);

    user_repo.create(&user).await?;

    let session_id = SessionId::new();
    let family_id = TokenFamilyId::new();
    let raw_refresh_token = token_service.generate_refresh_token();
    let refresh_hash = token_service.hash_refresh_token(&raw_refresh_token);
    let expires_at = Utc::now() + chrono::Duration::seconds(refresh_ttl_seconds);

    let session = Session::new(
        session_id,
        user_id,
        family_id,
        refresh_hash,
        expires_at,
        cmd.metadata.ip_address.clone(),
        cmd.metadata.user_agent.clone(),
    );

    session_repo.create(&session).await?;

    let access_token = token_service.generate_access_token(AccessTokenClaims {
        user_id,
        session_id,
        role: user.role,
    })?;

    let _ = audit_repo.record(
        Some(user_id),
        "USER_REGISTERED",
        cmd.metadata.ip_address,
        cmd.metadata.user_agent,
        None,
    ).await;

    Ok(AuthResponse {
        user_id,
        email: email.as_str().to_string(),
        role: user.role.to_string(),
        access_token,
        refresh_token: raw_refresh_token,
        expires_in: access_ttl_seconds,
    })
}