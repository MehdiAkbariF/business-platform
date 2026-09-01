use chrono::{DateTime, Utc};
use shared::{SessionId, TokenFamilyId, UserId};

#[derive(Debug, Clone)]
pub struct Session {
    pub id: SessionId,
    pub user_id: UserId,
    pub token_family_id: TokenFamilyId,
    pub refresh_token_hash: String,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub last_used_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl Session {
    pub fn new(
        id: SessionId,
        user_id: UserId,
        token_family_id: TokenFamilyId,
        refresh_token_hash: String,
        expires_at: DateTime<Utc>,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            user_id,
            token_family_id,
            refresh_token_hash,
            user_agent,
            ip_address,
            expires_at,
            revoked_at: None,
            last_used_at: now,
            created_at: now,
        }
    }

    pub fn is_active(&self) -> bool {
        self.revoked_at.is_none() && self.expires_at > Utc::now()
    }
}