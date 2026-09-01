use async_trait::async_trait;
use sqlx::PgPool;
use application::errors::AppError;
use application::ports::outbox::OutboxRepository;
use shared::{JobId, OutboxId};
use uuid::Uuid;

pub struct PostgresOutboxRepository {
    pool: PgPool,
}

impl PostgresOutboxRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct OutboxRow {
    id: Uuid,
    event_type: String,
    payload: serde_json::Value,
}

#[async_trait]
impl OutboxRepository for PostgresOutboxRepository {
    async fn publish_event(&self, event_type: &str, payload: serde_json::Value) -> Result<OutboxId, AppError> {
        let id = OutboxId::new();
        sqlx::query(
            "INSERT INTO outbox_events (id, event_type, payload, status, created_at) VALUES ($1, $2, $3, 'PENDING', NOW())"
        )
        .bind(id.0)
        .bind(event_type)
        .bind(payload)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        Ok(id)
    }

    async fn fetch_pending_events(&self, batch_size: usize) -> Result<Vec<(OutboxId, String, serde_json::Value)>, AppError> {
        let rows = sqlx::query_as::<_, OutboxRow>(
            "SELECT id, event_type, payload FROM outbox_events WHERE status = 'PENDING' ORDER BY created_at ASC LIMIT $1 FOR UPDATE SKIP LOCKED"
        )
        .bind(batch_size as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        Ok(rows.into_iter().map(|r| (OutboxId::from_uuid(r.id), r.event_type, r.payload)).collect())
    }

    async fn mark_processed(&self, id: OutboxId) -> Result<(), AppError> {
        sqlx::query("UPDATE outbox_events SET status = 'PROCESSED', processed_at = NOW() WHERE id = $1")
            .bind(id.0)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::internal(e))?;

        Ok(())
    }

    async fn mark_failed(&self, id: OutboxId, _error: &str) -> Result<(), AppError> {
        sqlx::query("UPDATE outbox_events SET status = 'FAILED', retry_count = retry_count + 1 WHERE id = $1")
            .bind(id.0)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::internal(e))?;

        Ok(())
    }

    async fn enqueue_job(&self, job_type: &str, payload: serde_json::Value) -> Result<JobId, AppError> {
        let id = JobId::new();
        sqlx::query(
            "INSERT INTO background_jobs (id, job_type, payload, status, attempts, max_attempts, scheduled_at, created_at, updated_at) 
             VALUES ($1, $2, $3, 'QUEUED', 0, 5, NOW(), NOW(), NOW())"
        )
        .bind(id.0)
        .bind(job_type)
        .bind(payload)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        Ok(id)
    }
}