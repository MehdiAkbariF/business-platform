use axum::{extract::State, Json};
use application::use_cases::get_me::{handle_get_me, UserProfileDto};
use crate::errors::ApiError;
use crate::middleware::auth::AuthPrincipal;
use crate::state::AppState;

#[utoipa::path(
    get,
    path = "/api/v1/me",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Current authenticated user profile", body = UserProfileDto),
        (status = 401, description = "Unauthorized", body = crate::errors::ApiErrorResponse)
    )
)]
pub async fn get_me(
    State(state): State<AppState>,
    AuthPrincipal(principal): AuthPrincipal,
) -> Result<Json<UserProfileDto>, ApiError> {
    let profile = handle_get_me(state.user_repo.clone(), principal.user_id).await?;
    Ok(Json(profile))
}