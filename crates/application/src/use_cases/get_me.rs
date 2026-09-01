use std::sync::Arc;
use chrono::{DateTime, Utc};
use serde::Serialize;
use shared::UserId;
use utoipa::ToSchema;
use crate::errors::AppError;
use crate::ports::repositories::UserRepository;

#[derive(Serialize, ToSchema)]
pub struct UserProfileDto {
    pub id: UserId,
    pub email: String,
    pub role: String,
    pub status: String,
    pub email_verified_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

pub async fn handle_get_me(
    user_repo: Arc<dyn UserRepository>,
    user_id: UserId,
) -> Result<UserProfileDto, AppError> {
    let user = user_repo
        .find_by_id(user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    Ok(UserProfileDto {
        id: user.id,
        email: user.email.as_str().to_string(),
        role: user.role.to_string(),
        status: user.status.to_string(),
        email_verified_at: user.email_verified_at,
        created_at: user.created_at,
    })
}