use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Validation failed: {0}")]
    Validation(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Conflict occurred: {0}")]
    Conflict(String),

    #[error("Too many requests: {0}")]
    RateLimited(String),

    #[error("Internal system error")]
    Internal(#[source] anyhow::Error),
}

impl AppError {
    pub fn internal<E: Into<anyhow::Error>>(error: E) -> Self {
        Self::Internal(error.into())
    }
}