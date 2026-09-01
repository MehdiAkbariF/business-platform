use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared::{BusinessId, MembershipId, UserId};
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MembershipRole {
    Editor,
    Admin,
    Owner,
}

impl std::fmt::Display for MembershipRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Editor => write!(f, "EDITOR"),
            Self::Admin => write!(f, "ADMIN"),
            Self::Owner => write!(f, "OWNER"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MembershipStatus {
    Active,
    Inactive,
}

impl std::fmt::Display for MembershipStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "ACTIVE"),
            Self::Inactive => write!(f, "INACTIVE"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BusinessMembership {
    pub id: MembershipId,
    pub business_id: BusinessId,
    pub user_id: UserId,
    pub role: MembershipRole,
    pub status: MembershipStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl BusinessMembership {
    pub fn new(id: MembershipId, business_id: BusinessId, user_id: UserId, role: MembershipRole) -> Self {
        let now = Utc::now();
        Self {
            id,
            business_id,
            user_id,
            role,
            status: MembershipStatus::Active,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn can_edit_profile(&self) -> bool {
        self.status == MembershipStatus::Active
    }

    pub fn can_manage_members(&self) -> bool {
        self.status == MembershipStatus::Active && (self.role == MembershipRole::Admin || self.role == MembershipRole::Owner)
    }

    pub fn can_submit(&self) -> bool {
        self.status == MembershipStatus::Active && (self.role == MembershipRole::Admin || self.role == MembershipRole::Owner)
    }

    pub fn can_archive(&self) -> bool {
        self.status == MembershipStatus::Active && self.role == MembershipRole::Owner
    }

    pub fn can_change_roles(&self) -> bool {
        self.status == MembershipStatus::Active && self.role == MembershipRole::Owner
    }
}