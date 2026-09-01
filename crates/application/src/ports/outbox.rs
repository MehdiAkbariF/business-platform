use async_trait::async_trait;
use shared::{JobId, OutboxId};
use crate::errors::AppError;

#[async_trait]
pub trait OutboxRepository: Send + Sync {
    async fn publish_event(&self, event_type: &str, payload: serde_json::Value) -> Result<OutboxId, AppError>;
    async fn fetch_pending_events(&self, batch_size: usize) -> Result<Vec<(OutboxId, String, serde_json::Value)>, AppError>;
    async fn mark_processed(&self, id: OutboxId) -> Result<(), AppError>;
    async fn mark_failed(&self, id: OutboxId, error: &str) -> Result<(), AppError>;
    async fn enqueue_job(&self, job_type: &str, payload: serde_json::Value) -> Result<JobId, AppError>;
}