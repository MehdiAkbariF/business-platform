use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared::BusinessId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessContact {
    pub business_id: BusinessId,
    pub phone: Option<String>,
    pub mobile: Option<String>,
    pub email: Option<String>,
    pub website: Option<String>,
    pub preferred_contact_method: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}