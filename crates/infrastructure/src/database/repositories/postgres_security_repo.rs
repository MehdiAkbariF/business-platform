use async_trait::async_trait;
use chrono::{DateTime, Utc};
use domain::security::SecuritySeverity;
use shared::{IncidentId, UserId};
use sqlx::PgPool;
use application::errors::AppError;
use application::ports::security_ops::{SecurityIncidentDto, SecurityOperationsRepository};
use uuid::Uuid;

pub struct PostgresSecurityRepository {
    pool: PgPool,
}

impl PostgresSecurityRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct IncidentRow {
    id: Uuid,
    severity: String,
    event_type: String,
    actor_id: Option<Uuid>,
    ip_address: String,
    details: serde_json::Value,
    created_at: DateTime<Utc>,
}

#[async_trait]
impl SecurityOperationsRepository for PostgresSecurityRepository {
    async fn record_incident(
        &self,
        incident_id: IncidentId,
        severity: SecuritySeverity,
        event_type: &str,
        actor: Option<UserId>,
        ip: &str,
        details: serde_json::Value,
    ) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO security_incidents (id, severity, event_type, actor_id, ip_address, details, created_at) 
             VALUES ($1, $2, $3, $4, $5, $6, NOW())"
        )
        .bind(incident_id.0)
        .bind(severity.to_string())
        .bind(event_type)
        .bind(actor.map(|a| a.0))
        .bind(ip)
        .bind(details)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        Ok(())
    }

    async fn is_ip_blocked(&self, ip: &str) -> Result<bool, AppError> {
        let row: Option<(bool,)> = sqlx::query_as(
            "SELECT TRUE FROM blocked_ip_records WHERE ip_address = $1 AND blocked_until > NOW() LIMIT 1"
        )
        .bind(ip)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        Ok(row.is_some())
    }

    async fn block_ip(&self, ip: &str, reason: &str, duration_minutes: i64) -> Result<(), AppError> {
        let blocked_until = Utc::now() + chrono::Duration::minutes(duration_minutes);
        sqlx::query(
            "INSERT INTO blocked_ip_records (ip_address, reason, blocked_until, created_at) 
             VALUES ($1, $2, $3, NOW())
             ON CONFLICT (ip_address) DO UPDATE SET reason = EXCLUDED.reason, blocked_until = EXCLUDED.blocked_until"
        )
        .bind(ip)
        .bind(reason)
        .bind(blocked_until)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        Ok(())
    }

    async fn list_recent_incidents(&self, limit: usize) -> Result<Vec<SecurityIncidentDto>, AppError> {
        let rows = sqlx::query_as::<_, IncidentRow>(
            "SELECT id, severity, event_type, actor_id, ip_address, details, created_at FROM security_incidents ORDER BY created_at DESC LIMIT $1"
        )
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        Ok(rows.into_iter().map(|r| SecurityIncidentDto {
            id: IncidentId::from_uuid(r.id),
            severity: r.severity,
            event_type: r.event_type,
            actor_id: r.actor_id.map(UserId::from_uuid),
            ip_address: r.ip_address,
            details: r.details,
            created_at: r.created_at,
        }).collect())
    }
}