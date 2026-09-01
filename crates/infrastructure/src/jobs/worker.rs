use std::sync::Arc;
use std::time::Duration;
use sqlx::PgPool;
use tokio::time::sleep;

pub struct BackgroundJobWorker {
    pool: PgPool,
}

impl BackgroundJobWorker {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub fn start(self: Arc<Self>) {
        tokio::spawn(async move {
            tracing::info!("Starting background job worker...");
            loop {
                if let Err(err) = self.process_jobs_tick().await {
                    tracing::error!(error = ?err, "Error in background job worker cycle");
                }
                sleep(Duration::from_secs(5)).await;
            }
        });
    }

    async fn process_jobs_tick(&self) -> Result<(), sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct JobRow {
            id: uuid::Uuid,
            job_type: String,
            attempts: i32,
            max_attempts: i32,
        }

        let jobs = sqlx::query_as::<_, JobRow>(
            "SELECT id, job_type, attempts, max_attempts FROM background_jobs 
             WHERE status = 'QUEUED' AND scheduled_at <= NOW() 
             ORDER BY scheduled_at ASC LIMIT 10 FOR UPDATE SKIP LOCKED"
        )
        .fetch_all(&self.pool)
        .await?;

        for job in jobs {
            tracing::info!(job_id = %job.id, job_type = %job.job_type, "Executing background job");

            // Execute job logic idempotently
            let success = true; // In production this dispatches to specific handlers

            if success {
                sqlx::query("UPDATE background_jobs SET status = 'COMPLETED', updated_at = NOW() WHERE id = $1")
                    .bind(job.id)
                    .execute(&self.pool)
                    .await?;
            } else {
                let next_attempts = job.attempts + 1;
                if next_attempts >= job.max_attempts {
                    // Send to Dead Letter Queue (DLQ)
                    sqlx::query("UPDATE background_jobs SET status = 'DEAD_LETTER', attempts = $1, updated_at = NOW() WHERE id = $2")
                        .bind(next_attempts)
                        .bind(job.id)
                        .execute(&self.pool)
                        .await?;
                } else {
                    // Exponential backoff: 2^attempts * 10 seconds
                    let backoff_secs = (2_i64.pow(next_attempts as u32) * 10).min(3600);
                    sqlx::query("UPDATE background_jobs SET attempts = $1, scheduled_at = NOW() + ($2 || ' seconds')::interval, updated_at = NOW() WHERE id = $3")
                        .bind(next_attempts)
                        .bind(backoff_secs.to_string())
                        .bind(job.id)
                        .execute(&self.pool)
                        .await?;
                }
            }
        }

        Ok(())
    }
}