use async_trait::async_trait;
use domain::admin::{AdminDashboardMetrics, AppealStatus, BusinessAppeal, FeatureFlagItem, SystemConfigItem};
use shared::{AppealId, BusinessId, UserId};
use utoipa::ToSchema;
use serde::Serialize;
use crate::errors::AppError;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct AdminUserDto {
    pub id: UserId,
    pub email: String,
    pub status: String,
    pub role: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_login_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct BusinessAppealDto {
    pub id: AppealId,
    pub business_id: BusinessId,
    pub reason: String,
    pub status: AppealStatus,
    pub submitted_by: UserId,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct AuditLogEntryDto {
    pub id: uuid::Uuid,
    pub actor_id: Option<UserId>,
    pub action: String,
    pub ip_address: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[async_trait]
pub trait AdminOperationsRepository: Send + Sync {
    // Dashboard & Metrics
    async fn get_dashboard_metrics(&self) -> Result<AdminDashboardMetrics, AppError>;

    // User Operations
    async fn list_users(&self, limit: usize, offset: usize) -> Result<Vec<AdminUserDto>, AppError>;
    async fn suspend_user(&self, user_id: UserId, reason: &str) -> Result<(), AppError>;
    async fn restore_user(&self, user_id: UserId) -> Result<(), AppError>;

    // Appeals
    async fn create_appeal(&self, appeal: &BusinessAppeal) -> Result<(), AppError>;
    async fn list_appeals(&self, limit: usize) -> Result<Vec<BusinessAppealDto>, AppError>;
    async fn resolve_appeal(&self, appeal_id: AppealId, status: AppealStatus, resolver_id: UserId) -> Result<(), AppError>;

    // Feature Flags & Configs
    async fn list_feature_flags(&self) -> Result<Vec<FeatureFlagItem>, AppError>;
    async fn set_feature_flag(&self, key: &str, is_enabled: bool, updated_by: UserId) -> Result<(), AppError>;
    async fn is_feature_enabled(&self, key: &str) -> Result<bool, AppError>;
    async fn list_runtime_configs(&self) -> Result<Vec<SystemConfigItem>, AppError>;
    async fn set_runtime_config(&self, key: &str, value: serde_json::Value, category: &str, updated_by: UserId) -> Result<(), AppError>;

    // Audit Trail
    async fn list_audit_logs(&self, limit: usize) -> Result<Vec<AuditLogEntryDto>, AppError>;
}