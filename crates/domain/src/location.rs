use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared::{BusinessId, LocationId};
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LocationAccuracy {
    Exact,
    Approximate,
}

impl std::fmt::Display for LocationAccuracy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Exact => write!(f, "EXACT"),
            Self::Approximate => write!(f, "APPROXIMATE"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessLocation {
    pub id: LocationId,
    pub business_id: BusinessId,
    pub label: String,
    pub latitude: f64,
    pub longitude: f64,
    pub country: String,
    pub province: String,
    pub city: String,
    pub district: Option<String>,
    pub street: String,
    pub postal_code: Option<String>,
    pub formatted_address: String,
    pub accuracy: LocationAccuracy,
    pub is_primary: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}