use std::sync::Arc;
use domain::admin::{AdminDashboardMetrics, AppealStatus, BusinessAppeal, FeatureFlagItem, SystemConfigItem};
use shared::{AppealId, BusinessId, ClientMetadata, UserId};
use utoipa::ToSchema;
use serde::Deserialize;
use crate::errors::AppError;
use crate::ports::admin::{AdminOperationsRepository, AdminUserDto, AuditLogEntryDto, BusinessAppealDto};
use crate::ports::repositories::{AuditRepository, SessionRepository};

#[derive(Deserialize, ToSchema)]
pub struct SubmitAppealCommand {
    pub reason: String,
}

#[derive(Deserialize, ToSchema)]
pub struct ResolveAppealCommand {
    pub status: AppealStatus,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateFeatureFlagCommand {
    pub is_enabled: bool,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateConfigCommand {
    pub value_json: serde_json::Value,
    pub category: String,
}

pub async fn get_dashboard(admin_repo: Arc<dyn AdminOperationsRepository>) -> Result<AdminDashboardMetrics, AppError> {
    admin_repo.get_dashboard_metrics().await
}

pub async fn list_platform_users(
    admin_repo: Arc<dyn AdminOperationsRepository>,
    limit: usize,
    offset: usize,
) -> Result<Vec<AdminUserDto>, AppError> {
    admin_repo.list_users(limit, offset).await
}

pub async fn suspend_platform_user(
    admin_repo: Arc<dyn AdminOperationsRepository>,
    session_repo: Arc<dyn SessionRepository>,
    audit_repo: Arc<dyn AuditRepository>,
    target_user_id: UserId,
    moderator_id: UserId,
    reason: &str,
    metadata: ClientMetadata,
) -> Result<(), AppError> {
    admin_repo.suspend_user(target_user_id, reason).await?;
    // Securely revoke all active sessions of suspended user
    session_repo.revoke_all_user_sessions(target_user_id).await?;

    let _ = audit_repo.record(
        Some(moderator_id),
        "USER_SUSPENDED_BY_ADMIN",
        metadata.ip_address,
        metadata.user_agent,
        Some(serde_json::json!({ "target_user_id": target_user_id.to_string(), "reason": reason })),
    ).await;

    Ok(())
}

pub async fn restore_platform_user(
    admin_repo: Arc<dyn AdminOperationsRepository>,
    audit_repo: Arc<dyn AuditRepository>,
    target_user_id: UserId,
    moderator_id: UserId,
    metadata: ClientMetadata,
) -> Result<(), AppError> {
    admin_repo.restore_user(target_user_id).await?;

    let _ = audit_repo.record(
        Some(moderator_id),
        "USER_RESTORED_BY_ADMIN",
        metadata.ip_address,
        metadata.user_agent,
        Some(serde_json::json!({ "target_user_id": target_user_id.to_string() })),
    ).await;

    Ok(())
}

pub async fn revoke_user_sessions_admin(
    session_repo: Arc<dyn SessionRepository>,
    audit_repo: Arc<dyn AuditRepository>,
    target_user_id: UserId,
    moderator_id: UserId,
    metadata: ClientMetadata,
) -> Result<(), AppError> {
    session_repo.revoke_all_user_sessions(target_user_id).await?;

    let _ = audit_repo.record(
        Some(moderator_id),
        "USER_SESSIONS_REVOKED_BY_ADMIN",
        metadata.ip_address,
        metadata.user_agent,
        Some(serde_json::json!({ "target_user_id": target_user_id.to_string() })),
    ).await;

    Ok(())
}

pub async fn submit_business_appeal(
    admin_repo: Arc<dyn AdminOperationsRepository>,
    audit_repo: Arc<dyn AuditRepository>,
    business_id: BusinessId,
    user_id: UserId,
    cmd: SubmitAppealCommand,
    metadata: ClientMetadata,
) -> Result<AppealId, AppError> {
    let appeal_id = AppealId::new();
    let appeal = BusinessAppeal {
        id: appeal_id,
        business_id,
        case_id: None,
        reason: cmd.reason,
        status: AppealStatus::Pending,
        submitted_by: user_id,
        resolved_by: None,
        created_at: chrono::Utc::now(),
        resolved_at: None,
    };

    admin_repo.create_appeal(&appeal).await?;

    let _ = audit_repo.record(
        Some(user_id),
        "BUSINESS_APPEAL_SUBMITTED",
        metadata.ip_address,
        metadata.user_agent,
        Some(serde_json::json!({ "business_id": business_id.to_string(), "appeal_id": appeal_id.to_string() })),
    ).await;

    Ok(appeal_id)
}

pub async fn list_appeals(admin_repo: Arc<dyn AdminOperationsRepository>, limit: usize) -> Result<Vec<BusinessAppealDto>, AppError> {
    admin_repo.list_appeals(limit).await
}

pub async fn resolve_appeal_cmd(
    admin_repo: Arc<dyn AdminOperationsRepository>,
    audit_repo: Arc<dyn AuditRepository>,
    appeal_id: AppealId,
    moderator_id: UserId,
    status: AppealStatus,
    metadata: ClientMetadata,
) -> Result<(), AppError> {
    admin_repo.resolve_appeal(appeal_id, status, moderator_id).await?;

    let _ = audit_repo.record(
        Some(moderator_id),
        "BUSINESS_APPEAL_RESOLVED",
        metadata.ip_address,
        metadata.user_agent,
        Some(serde_json::json!({ "appeal_id": appeal_id.to_string(), "status": status.to_string() })),
    ).await;

    Ok(())
}

pub async fn get_feature_flags(admin_repo: Arc<dyn AdminOperationsRepository>) -> Result<Vec<FeatureFlagItem>, AppError> {
    admin_repo.list_feature_flags().await
}

pub async fn set_feature_flag_cmd(
    admin_repo: Arc<dyn AdminOperationsRepository>,
    audit_repo: Arc<dyn AuditRepository>,
    key: &str,
    is_enabled: bool,
    moderator_id: UserId,
    metadata: ClientMetadata,
) -> Result<(), AppError> {
    admin_repo.set_feature_flag(key, is_enabled, moderator_id).await?;

    let _ = audit_repo.record(
        Some(moderator_id),
        "FEATURE_FLAG_UPDATED",
        metadata.ip_address,
        metadata.user_agent,
        Some(serde_json::json!({ "flag_key": key, "is_enabled": is_enabled })),
    ).await;

    Ok(())
}

pub async fn get_system_configs(admin_repo: Arc<dyn AdminOperationsRepository>) -> Result<Vec<SystemConfigItem>, AppError> {
    admin_repo.list_runtime_configs().await
}

pub async fn set_system_config_cmd(
    admin_repo: Arc<dyn AdminOperationsRepository>,
    audit_repo: Arc<dyn AuditRepository>,
    key: &str,
    cmd: UpdateConfigCommand,
    moderator_id: UserId,
    metadata: ClientMetadata,
) -> Result<(), AppError> {
    admin_repo.set_runtime_config(key, cmd.value_json, &cmd.category, moderator_id).await?;

    let _ = audit_repo.record(
        Some(moderator_id),
        "SYSTEM_CONFIG_UPDATED",
        metadata.ip_address,
        metadata.user_agent,
        Some(serde_json::json!({ "config_key": key })),
    ).await;

    Ok(())
}

pub async fn get_audit_trail(admin_repo: Arc<dyn AdminOperationsRepository>, limit: usize) -> Result<Vec<AuditLogEntryDto>, AppError> {
    admin_repo.list_audit_logs(limit).await
}