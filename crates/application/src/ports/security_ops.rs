use async_trait::async_trait;
use domain::security::SecuritySeverity;
use shared::{IncidentId, UserId};
use utoipa::ToSchema;
use serde::Serialize;
use crate::errors::AppError;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct SecurityIncidentDto {
    pub id: IncidentId,
    pub severity: String,
    pub event_type: String,
    pub actor_id: Option<UserId>,
    pub ip_address: String,
    pub details: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[async_trait]
pub trait SecurityOperationsRepository: Send + Sync {
    async fn record_incident(&self, incident_id: IncidentId, severity: SecuritySeverity, event_type: &str, actor: Option<UserId>, ip: &str, details: serde_json::Value) -> Result<(), AppError>;
    async fn is_ip_blocked(&self, ip: &str) -> Result<bool, AppError>;
    async fn block_ip(&self, ip: &str, reason: &str, duration_minutes: i64) -> Result<(), AppError>;
    async fn list_recent_incidents(&self, limit: usize) -> Result<Vec<SecurityIncidentDto>, AppError>;
}