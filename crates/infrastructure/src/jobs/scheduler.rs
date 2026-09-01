use std::sync::Arc;
use std::time::Duration;
use sqlx::PgPool;
use tokio::time::sleep;

pub struct PeriodicScheduler {
    pool: PgPool,
}

impl PeriodicScheduler {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub fn start(self: Arc<Self>) {
        tokio::spawn(async move {
            tracing::info!("Starting periodic system scheduler...");
            loop {
                // Run maintenance tasks every 60 seconds
                sleep(Duration::from_secs(60)).await;

                // 1. Auto-expire finished ad campaigns
                let _ = sqlx::query(
                    "UPDATE ad_campaigns SET status = 'COMPLETED', updated_at = NOW() WHERE status = 'ACTIVE' AND end_date <= NOW()"
                )
                .execute(&self.pool)
                .await;

                // 2. Auto-expire exhausted ad campaigns
                let _ = sqlx::query(
                    "UPDATE ad_campaigns SET status = 'EXHAUSTED', updated_at = NOW() WHERE status = 'ACTIVE' AND spent_amount >= total_budget"
                )
                .execute(&self.pool)
                .await;

                // 3. Cleanup unblocked IP records past deadline
                let _ = sqlx::query(
                    "DELETE FROM blocked_ip_records WHERE blocked_until <= NOW()"
                )
                .execute(&self.pool)
                .await;
            }
        });
    }
}