use async_trait::async_trait;
use domain::user::GlobalRole;
use shared::{SessionId, UserId};
use crate::errors::AppError;

pub struct AccessTokenClaims {
    pub user_id: UserId,
    pub session_id: SessionId,
    pub role: GlobalRole,
}

#[async_trait]
pub trait PasswordHasherPort: Send + Sync {
    fn hash_password(&self, password: &str) -> Result<String, AppError>;
    fn verify_password(&self, password: &str, hash: &str) -> Result<bool, AppError>;
}

#[async_trait]
pub trait TokenServicePort: Send + Sync {
    fn generate_access_token(&self, claims: AccessTokenClaims) -> Result<String, AppError>;
    fn verify_access_token(&self, token: &str) -> Result<AccessTokenClaims, AppError>;
    fn generate_refresh_token(&self) -> String;
    fn hash_refresh_token(&self, raw_token: &str) -> String;
}

#[async_trait]
pub trait RateLimiterPort: Send + Sync {
    async fn check_rate_limit(&self, key: &str, max_requests: u32, window_seconds: u64) -> Result<bool, AppError>;
}