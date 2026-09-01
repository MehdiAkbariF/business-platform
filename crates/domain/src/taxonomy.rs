use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared::{AliasId, CategoryId, ServiceId};
use utoipa::ToSchema;
use uuid::Uuid;
use crate::validation::ValidationError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TaxonomyStatus {
    Active,
    Inactive,
}

impl std::fmt::Display for TaxonomyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "ACTIVE"),
            Self::Inactive => write!(f, "INACTIVE"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TaxonomyType {
    Category,
    Service,
}

impl std::fmt::Display for TaxonomyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Category => write!(f, "CATEGORY"),
            Self::Service => write!(f, "SERVICE"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaxonomySlug(String);

impl TaxonomySlug {
    pub fn parse(raw: &str) -> Result<Self, ValidationError> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(ValidationError::Required("slug".to_string()));
        }
        if trimmed.len() > 150 {
            return Err(ValidationError::TooLong("slug".to_string(), 150));
        }
        let is_valid = trimmed.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
        if !is_valid {
            return Err(ValidationError::InvalidFormat("slug".to_string(), "Slug must be alphanumeric with dashes".to_string()));
        }
        Ok(Self(trimmed.to_lowercase()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct Category {
    pub id: CategoryId,
    pub parent_id: Option<CategoryId>,
    pub name: String,
    pub normalized_name: String,
    pub slug: TaxonomySlug,
    pub description: Option<String>,
    pub status: TaxonomyStatus,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct Service {
    pub id: ServiceId,
    pub parent_id: Option<ServiceId>,
    pub name: String,
    pub normalized_name: String,
    pub slug: TaxonomySlug,
    pub description: Option<String>,
    pub status: TaxonomyStatus,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct TaxonomyAlias {
    pub id: AliasId,
    pub taxonomy_type: TaxonomyType,
    pub taxonomy_id: Uuid,
    pub term: String,
    pub normalized_term: String,
    pub language: String,
    pub created_at: DateTime<Utc>,
}

pub fn normalize_taxonomy_term(raw: &str) -> String {
    raw.trim()
        .to_lowercase()
        .replace('ي', "ی")
        .replace('ك', "ک")
}