use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared::{BusinessId, MediaId};
use utoipa::ToSchema;
use crate::validation::ValidationError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MediaType {
    Logo,
    Cover,
    Gallery,
}

impl std::fmt::Display for MediaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Logo => write!(f, "LOGO"),
            Self::Cover => write!(f, "COVER"),
            Self::Gallery => write!(f, "GALLERY"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MediaStatus {
    Processing,
    Active,
    Rejected,
}

impl std::fmt::Display for MediaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Processing => write!(f, "PROCESSING"),
            Self::Active => write!(f, "ACTIVE"),
            Self::Rejected => write!(f, "REJECTED"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BusinessMedia {
    pub id: MediaId,
    pub business_id: BusinessId,
    pub media_type: MediaType,
    pub storage_key: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub width: i32,
    pub height: i32,
    pub alt_text: Option<String>,
    pub sort_order: i32,
    pub status: MediaStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BusinessHoursInterval {
    pub day_of_week: u8,
    pub opens_at: Option<String>,
    pub closes_at: Option<String>,
    pub is_24_hours: bool,
    pub sort_order: i32,
}

impl BusinessHoursInterval {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.day_of_week > 6 {
            return Err(ValidationError::InvalidFormat("day_of_week".to_string(), "Must be between 0 and 6".to_string()));
        }
        if self.is_24_hours {
            return Ok(());
        }
        match (&self.opens_at, &self.closes_at) {
            (Some(op), Some(cl)) => {
                if op.len() != 5 || cl.len() != 5 {
                    return Err(ValidationError::InvalidFormat("hours".to_string(), "Format must be HH:MM".to_string()));
                }
                Ok(())
            }
            _ => Err(ValidationError::Required("opens_at and closes_at required when not 24 hours".to_string())),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AttributeType {
    Boolean,
    Enum,
}

impl std::fmt::Display for AttributeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Boolean => write!(f, "BOOLEAN"),
            Self::Enum => write!(f, "ENUM"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SocialPlatform {
    Instagram,
    Telegram,
    Whatsapp,
    Linkedin,
    Facebook,
    Youtube,
}

impl std::fmt::Display for SocialPlatform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Instagram => write!(f, "INSTAGRAM"),
            Self::Telegram => write!(f, "TELEGRAM"),
            Self::Whatsapp => write!(f, "WHATSAPP"),
            Self::Linkedin => write!(f, "LINKEDIN"),
            Self::Facebook => write!(f, "FACEBOOK"),
            Self::Youtube => write!(f, "YOUTUBE"),
        }
    }
}

pub fn validate_safe_url(raw_url: &str) -> Result<String, ValidationError> {
    let parsed = url::Url::parse(raw_url.trim())
        .map_err(|_| ValidationError::InvalidFormat("url".to_string(), "Malformed URL".to_string()))?;
    
    if parsed.scheme() != "http" && parsed.scheme() != "https" {
        return Err(ValidationError::InvalidFormat("url".to_string(), "Only HTTP and HTTPS URLs are permitted".to_string()));
    }

    Ok(parsed.to_string())
}