use thiserror::Error;

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Value is required: {0}")]
    Required(String),
    #[error("Value exceeds length: field={0}, max={1}")]
    TooLong(String, usize),
    #[error("Invalid format: field={0}, reason={1}")]
    InvalidFormat(String, String),
}