use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared::{AppealId, BusinessId, CaseId, UserId};
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AppealStatus {
    Pending,
    UnderReview,
    Accepted,
    Rejected,
}

impl std::fmt::Display for AppealStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "PENDING"),
            Self::UnderReview => write!(f, "UNDER_REVIEW"),
            Self::Accepted => write!(f, "ACCEPTED"),
            Self::Rejected => write!(f, "REJECTED"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BusinessAppeal {
    pub id: AppealId,
    pub business_id: BusinessId,
    pub case_id: Option<CaseId>,
    pub reason: String,
    pub status: AppealStatus,
    pub submitted_by: UserId,
    pub resolved_by: Option<UserId>,
    pub created_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FeatureFlagItem {
    pub key: String,
    pub description: String,
    pub is_enabled: bool,
    pub target_type: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SystemConfigItem {
    pub key: String,
    pub value_json: serde_json::Value,
    pub category: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct AdminDashboardMetrics {
    pub pending_businesses: i64,
    pub open_reports: i64,
    pub pending_appeals: i64,
    pub active_campaigns: i64,
    pub total_published_businesses: i64,
    pub total_active_users: i64,
}