use std::sync::Arc;
use domain::security::{sanitize_storage_key, validate_public_destination_url, SecuritySeverity};
use shared::{IncidentId, UserId};
use crate::errors::AppError;
use crate::ports::repositories::{AuditRepository, SessionRepository, UserRepository};
use crate::ports::security_ops::{SecurityIncidentDto, SecurityOperationsRepository};

pub async fn validate_outbound_url_guard(
    sec_repo: Arc<dyn SecurityOperationsRepository>,
    raw_url: &str,
    actor: Option<UserId>,
    ip: &str,
) -> Result<String, AppError> {
    match validate_public_destination_url(raw_url) {
        Ok(clean_url) => Ok(clean_url),
        Err(e) => {
            let _ = sec_repo.record_incident(
                IncidentId::new(),
                SecuritySeverity::High,
                "SSRF_ATTEMPT",
                actor,
                ip,
                serde_json::json!({ "attempted_url": raw_url, "error": e.to_string() }),
            ).await;
            Err(AppError::Validation("Disallowed target destination URL".to_string()))
        }
    }
}

pub async fn validate_storage_key_guard(
    sec_repo: Arc<dyn SecurityOperationsRepository>,
    raw_key: &str,
    actor: Option<UserId>,
    ip: &str,
) -> Result<String, AppError> {
    match sanitize_storage_key(raw_key) {
        Ok(clean_key) => Ok(clean_key),
        Err(e) => {
            let _ = sec_repo.record_incident(
                IncidentId::new(),
                SecuritySeverity::High,
                "PATH_TRAVERSAL_ATTEMPT",
                actor,
                ip,
                serde_json::json!({ "attempted_key": raw_key, "error": e.to_string() }),
            ).await;
            Err(AppError::Validation("Invalid storage path key".to_string()))
        }
    }
}

pub async fn anonymize_user_account(
    user_repo: Arc<dyn UserRepository>,
    session_repo: Arc<dyn SessionRepository>,
    audit_repo: Arc<dyn AuditRepository>,
    target_user_id: UserId,
) -> Result<(), AppError> {
    // 1. Revoke all active sessions
    session_repo.revoke_all_user_sessions(target_user_id).await?;

    // 2. Perform Anonymization (Clearing PII while preserving referential integrity for ledger & audits)
    let anon_email = format!("deleted_user_{}@platform.local", target_user_id.0);
    if let Some(mut user) = user_repo.find_by_id(target_user_id).await? {
        user.email = domain::user::Email::parse(&anon_email).map_err(|e| AppError::Validation(e.to_string()))?;
        user.password_hash = "ANONYMIZED_DELETED_ACCOUNT".to_string();
        user.status = domain::user::UserStatus::Deactivated;
        // Keep User records in place without breaking foreign keys
    }

    let _ = audit_repo.record(
        Some(target_user_id),
        "ACCOUNT_ANONYMIZED_AND_DELETED",
        None,
        None,
        None,
    ).await;

    Ok(())
}

pub async fn get_security_incidents(
    sec_repo: Arc<dyn SecurityOperationsRepository>,
    limit: usize,
) -> Result<Vec<SecurityIncidentDto>, AppError> {
    sec_repo.list_recent_incidents(limit).await
}