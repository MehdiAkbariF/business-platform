use async_trait::async_trait;
use bytes::Bytes;
use crate::errors::AppError;

#[async_trait]
pub trait ObjectStoragePort: Send + Sync {
    async fn put_object(&self, key: &str, data: Bytes, content_type: &str) -> Result<String, AppError>;
    async fn get_object(&self, key: &str) -> Result<Bytes, AppError>;
    async fn delete_object(&self, key: &str) -> Result<(), AppError>;
    async fn check_health(&self) -> Result<(), AppError>;
}