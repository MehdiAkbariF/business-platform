use std::sync::Arc;
use shared::{ClientMetadata, SessionId, UserId};
use crate::errors::AppError;
use crate::ports::repositories::{AuditRepository, SessionRepository};

pub async fn handle_logout(
    session_repo: Arc<dyn SessionRepository>,
    audit_repo: Arc<dyn AuditRepository>,
    user_id: UserId,
    session_id: SessionId,
    metadata: ClientMetadata,
) -> Result<(), AppError> {
    session_repo.revoke_session(session_id).await?;
    let _ = audit_repo.record(
        Some(user_id),
        "USER_LOGGED_OUT",
        metadata.ip_address,
        metadata.user_agent,
        None,
    ).await;
    Ok(())
}

pub async fn handle_logout_all(
    session_repo: Arc<dyn SessionRepository>,
    audit_repo: Arc<dyn AuditRepository>,
    user_id: UserId,
    metadata: ClientMetadata,
) -> Result<(), AppError> {
    session_repo.revoke_all_user_sessions(user_id).await?;
    let _ = audit_repo.record(
        Some(user_id),
        "USER_LOGGED_OUT_ALL",
        metadata.ip_address,
        metadata.user_agent,
        None,
    ).await;
    Ok(())
}