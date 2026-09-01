use axum::{
    extract::{FromRef, FromRequestParts},
    http::{header, request::Parts},
};
use application::errors::AppError;
use application::security::principal::AuthenticatedPrincipal;
use crate::errors::ApiError;
use crate::state::AppState;

#[derive(Clone)]
pub struct AuthPrincipal(pub AuthenticatedPrincipal);

impl<S> FromRequestParts<S> for AuthPrincipal
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);
        let auth_header = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .ok_or_else(|| ApiError::from(AppError::Unauthorized("Missing Authorization header".to_string())))?;

        if !auth_header.starts_with("Bearer ") {
            return Err(ApiError::from(AppError::Unauthorized("Invalid authorization scheme".to_string())));
        }

        let token = &auth_header[7..];
        let claims = app_state
            .token_service
            .verify_access_token(token)
            .map_err(ApiError::from)?;

        Ok(AuthPrincipal(AuthenticatedPrincipal {
            user_id: claims.user_id,
            session_id: claims.session_id,
            role: claims.role,
        }))
    }
}