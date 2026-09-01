use async_trait::async_trait;
use shared::{AuditLogId, UserId};
use sqlx::PgPool;
use application::errors::AppError;
use application::ports::repositories::AuditRepository;

pub struct PostgresAuditRepository {
    pool: PgPool,
}

impl PostgresAuditRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AuditRepository for PostgresAuditRepository {
    async fn record(
        &self,
        actor: Option<UserId>,
        action: &str,
        ip: Option<String>,
        user_agent: Option<String>,
        metadata: Option<serde_json::Value>,
    ) -> Result<(), AppError> {
        let id = AuditLogId::new();
        sqlx::query(
            "INSERT INTO audit_logs (id, actor_user_id, action, ip_address, user_agent, metadata, created_at) VALUES ($1, $2, $3, $4, $5, $6, NOW())"
        )
        .bind(id.0)
        .bind(actor.map(|a| a.0))
        .bind(action)
        .bind(ip)
        .bind(user_agent)
        .bind(metadata)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        Ok(())
    }
}