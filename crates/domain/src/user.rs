use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared::UserId;
use utoipa::ToSchema;
use crate::validation::ValidationError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UserStatus {
    Active,
    Suspended,
    Deactivated,
}

impl std::fmt::Display for UserStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "ACTIVE"),
            Self::Suspended => write!(f, "SUSPENDED"),
            Self::Deactivated => write!(f, "DEACTIVATED"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GlobalRole {
    User,
    Moderator,
    Admin,
    SuperAdmin,
}

impl std::fmt::Display for GlobalRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::User => write!(f, "USER"),
            Self::Moderator => write!(f, "MODERATOR"),
            Self::Admin => write!(f, "ADMIN"),
            Self::SuperAdmin => write!(f, "SUPER_ADMIN"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Email(String);

impl Email {
    pub fn parse(raw: &str) -> Result<Self, ValidationError> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(ValidationError::Required("email".to_string()));
        }
        if trimmed.len() > 254 || !trimmed.contains('@') || trimmed.starts_with('@') || trimmed.ends_with('@') {
            return Err(ValidationError::InvalidFormat("email".to_string(), "Invalid email format".to_string()));
        }
        Ok(Self(trimmed.to_lowercase()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct User {
    pub id: UserId,
    pub email: Email,
    pub password_hash: String,
    pub status: UserStatus,
    pub role: GlobalRole,
    pub email_verified_at: Option<DateTime<Utc>>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    pub fn new(id: UserId, email: Email, password_hash: String) -> Self {
        let now = Utc::now();
        Self {
            id,
            email,
            password_hash,
            status: UserStatus::Active,
            role: GlobalRole::User,
            email_verified_at: None,
            last_login_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn can_authenticate(&self) -> bool {
        self.status == UserStatus::Active
    }
}