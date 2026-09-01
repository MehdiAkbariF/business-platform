use async_trait::async_trait;
use chrono::{DateTime, Utc};
use domain::session::Session;
use domain::user::User;
use shared::{SessionId, TokenFamilyId, UserId};
use crate::errors::AppError;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, id: UserId) -> Result<Option<User>, AppError>;
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError>;
    async fn create(&self, user: &User) -> Result<(), AppError>;
    async fn update_last_login(&self, id: UserId, login_time: DateTime<Utc>) -> Result<(), AppError>;
}

#[async_trait]
pub trait SessionRepository: Send + Sync {
    async fn create(&self, session: &Session) -> Result<(), AppError>;
    async fn find_by_id(&self, id: SessionId) -> Result<Option<Session>, AppError>;
    async fn find_by_refresh_token_hash_for_update(&self, hash: &str) -> Result<Option<Session>, AppError>;
    async fn rotate_token(&self, old_session_id: SessionId, new_session: &Session) -> Result<(), AppError>;
    async fn revoke_session(&self, id: SessionId) -> Result<(), AppError>;
    async fn revoke_token_family(&self, family_id: TokenFamilyId) -> Result<(), AppError>;
    async fn revoke_all_user_sessions(&self, user_id: UserId) -> Result<(), AppError>;
}

#[async_trait]
pub trait AuditRepository: Send + Sync {
    async fn record(
        &self,
        actor: Option<UserId>,
        action: &str,
        ip: Option<String>,
        user_agent: Option<String>,
        metadata: Option<serde_json::Value>,
    ) -> Result<(), AppError>;
}