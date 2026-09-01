use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared::{BusinessId, UserId};
use utoipa::ToSchema;
use crate::moderation::{ClaimStatus, VerificationStatus};
use crate::validation::ValidationError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BusinessStatus {
    Draft,
    PendingReview,
    Published,
    Rejected,
    Suspended,
    Archived,
}

impl std::fmt::Display for BusinessStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Draft => write!(f, "DRAFT"),
            Self::PendingReview => write!(f, "PENDING_REVIEW"),
            Self::Published => write!(f, "PUBLISHED"),
            Self::Rejected => write!(f, "REJECTED"),
            Self::Suspended => write!(f, "SUSPENDED"),
            Self::Archived => write!(f, "ARCHIVED"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BusinessSlug(String);

impl BusinessSlug {
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
            return Err(ValidationError::InvalidFormat("slug".to_string(), "Slug must be URL-safe alphanumeric with dashes/underscores".to_string()));
        }
        Ok(Self(trimmed.to_lowercase()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct Business {
    pub id: BusinessId,
    pub slug: BusinessSlug,
    pub name: String,
    pub short_description: Option<String>,
    pub description: Option<String>,
    pub timezone: String,
    pub status: BusinessStatus,
    pub verification_status: VerificationStatus,
    pub claim_status: ClaimStatus,
    pub version: i32,
    pub created_by: UserId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub published_at: Option<DateTime<Utc>>,
}

impl Business {
    pub fn new(id: BusinessId, slug: BusinessSlug, name: String, short_description: Option<String>, description: Option<String>, created_by: UserId) -> Self {
        let now = Utc::now();
        Self {
            id,
            slug,
            name,
            short_description,
            description,
            timezone: "Asia/Tehran".to_string(),
            status: BusinessStatus::Draft,
            verification_status: VerificationStatus::Unverified,
            claim_status: ClaimStatus::NotClaimed,
            version: 1,
            created_by,
            created_at: now,
            updated_at: now,
            published_at: None,
        }
    }

    pub fn can_transition_to(&self, next: BusinessStatus) -> bool {
        match (self.status, next) {
            (BusinessStatus::Draft, BusinessStatus::PendingReview) => true,
            (BusinessStatus::PendingReview, BusinessStatus::Published) => true,
            (BusinessStatus::PendingReview, BusinessStatus::Rejected) => true,
            (BusinessStatus::Rejected, BusinessStatus::Draft) => true,
            (BusinessStatus::Published, BusinessStatus::Suspended) => true,
            (BusinessStatus::Suspended, BusinessStatus::Published) => true,
            (BusinessStatus::Published, BusinessStatus::Archived) => true,
            (BusinessStatus::Suspended, BusinessStatus::Archived) => true,
            (BusinessStatus::PendingReview, BusinessStatus::Archived) => true,
            _ => false,
        }
    }

    pub fn submit_for_review(&mut self, has_primary_location: bool, has_primary_category: bool) -> Result<(), ValidationError> {
        if !has_primary_location {
            return Err(ValidationError::Required("A primary location is mandatory before submission".to_string()));
        }
        if !has_primary_category {
            return Err(ValidationError::Required("A primary category is mandatory before submission".to_string()));
        }
        if !self.can_transition_to(BusinessStatus::PendingReview) {
            return Err(ValidationError::InvalidFormat("status".to_string(), format!("Cannot transition from {} to PENDING_REVIEW", self.status)));
        }
        self.status = BusinessStatus::PendingReview;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn archive(&mut self) -> Result<(), ValidationError> {
        if !self.can_transition_to(BusinessStatus::Archived) {
            return Err(ValidationError::InvalidFormat("status".to_string(), format!("Cannot transition from {} to ARCHIVED", self.status)));
        }
        self.status = BusinessStatus::Archived;
        self.updated_at = Utc::now();
        Ok(())
    }
}