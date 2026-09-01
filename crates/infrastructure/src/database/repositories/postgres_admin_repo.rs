use async_trait::async_trait;
use chrono::{DateTime, Utc};
use domain::admin::{AdminDashboardMetrics, AppealStatus, BusinessAppeal, FeatureFlagItem, SystemConfigItem};
use shared::{AppealId, BusinessId, CaseId, UserId};
use sqlx::PgPool;
use application::errors::AppError;
use application::ports::admin::{AdminOperationsRepository, AdminUserDto, AuditLogEntryDto, BusinessAppealDto};
use uuid::Uuid;

pub struct PostgresAdminOperationsRepository {
    pool: PgPool,
}

impl PostgresAdminOperationsRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AdminOperationsRepository for PostgresAdminOperationsRepository {
    async fn get_dashboard_metrics(&self) -> Result<AdminDashboardMetrics, AppError> {
        let (pending_biz,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM businesses WHERE status = 'PENDING_REVIEW'").fetch_one(&self.pool).await.map_err(|e| AppError::internal(e))?;
        let (open_rep,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM business_reports WHERE status = 'PENDING'").fetch_one(&self.pool).await.map_err(|e| AppError::internal(e))?;
        let (pending_app,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM business_appeals WHERE status = 'PENDING'").fetch_one(&self.pool).await.map_err(|e| AppError::internal(e))?;
        let (active_camp,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM ad_campaigns WHERE status = 'ACTIVE'").fetch_one(&self.pool).await.map_err(|e| AppError::internal(e))?;
        let (published_biz,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM businesses WHERE status = 'PUBLISHED'").fetch_one(&self.pool).await.map_err(|e| AppError::internal(e))?;
        let (active_usr,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users WHERE status = 'ACTIVE'").fetch_one(&self.pool).await.map_err(|e| AppError::internal(e))?;

        Ok(AdminDashboardMetrics {
            pending_businesses: pending_biz,
            open_reports: open_rep,
            pending_appeals: pending_app,
            active_campaigns: active_camp,
            total_published_businesses: published_biz,
            total_active_users: active_usr,
        })
    }

    async fn list_users(&self, limit: usize, offset: usize) -> Result<Vec<AdminUserDto>, AppError> {
        #[derive(sqlx::FromRow)]
        struct Row { id: Uuid, email: String, status: String, role: String, created_at: DateTime<Utc>, last_login_at: Option<DateTime<Utc>> }

        let rows = sqlx::query_as::<_, Row>(
            "SELECT id, email, status::text, role::text, created_at, last_login_at FROM users ORDER BY created_at DESC LIMIT $1 OFFSET $2"
        )
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        Ok(rows.into_iter().map(|r| AdminUserDto {
            id: UserId::from_uuid(r.id),
            email: r.email,
            status: r.status,
            role: r.role,
            created_at: r.created_at,
            last_login_at: r.last_login_at,
        }).collect())
    }

    async fn suspend_user(&self, user_id: UserId, _reason: &str) -> Result<(), AppError> {
        sqlx::query("UPDATE users SET status = 'SUSPENDED'::user_status, updated_at = NOW() WHERE id = $1")
            .bind(user_id.0)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::internal(e))?;

        Ok(())
    }

    async fn restore_user(&self, user_id: UserId) -> Result<(), AppError> {
        sqlx::query("UPDATE users SET status = 'ACTIVE'::user_status, updated_at = NOW() WHERE id = $1")
            .bind(user_id.0)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::internal(e))?;

        Ok(())
    }

    async fn create_appeal(&self, a: &BusinessAppeal) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO business_appeals (id, business_id, case_id, reason, status, submitted_by, created_at) 
             VALUES ($1, $2, $3, $4, 'PENDING', $5, NOW())"
        )
        .bind(a.id.0)
        .bind(a.business_id.0)
        .bind(a.case_id.map(|c| c.0))
        .bind(&a.reason)
        .bind(a.submitted_by.0)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        Ok(())
    }

    async fn list_appeals(&self, limit: usize) -> Result<Vec<BusinessAppealDto>, AppError> {
        #[derive(sqlx::FromRow)]
        struct Row { id: Uuid, business_id: Uuid, reason: String, status: String, submitted_by: Uuid, created_at: DateTime<Utc> }

        let rows = sqlx::query_as::<_, Row>(
            "SELECT id, business_id, reason, status, submitted_by, created_at FROM business_appeals ORDER BY created_at DESC LIMIT $1"
        )
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        Ok(rows.into_iter().map(|r| BusinessAppealDto {
            id: AppealId::from_uuid(r.id),
            business_id: BusinessId::from_uuid(r.business_id),
            reason: r.reason,
            status: match r.status.as_str() {
                "ACCEPTED" => AppealStatus::Accepted,
                "REJECTED" => AppealStatus::Rejected,
                "UNDER_REVIEW" => AppealStatus::UnderReview,
                _ => AppealStatus::Pending,
            },
            submitted_by: UserId::from_uuid(r.submitted_by),
            created_at: r.created_at,
        }).collect())
    }

    async fn resolve_appeal(&self, appeal_id: AppealId, status: AppealStatus, resolver_id: UserId) -> Result<(), AppError> {
        sqlx::query("UPDATE business_appeals SET status = $1, resolved_by = $2, resolved_at = NOW() WHERE id = $3")
            .bind(status.to_string())
            .bind(resolver_id.0)
            .bind(appeal_id.0)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::internal(e))?;

        Ok(())
    }

    async fn list_feature_flags(&self) -> Result<Vec<FeatureFlagItem>, AppError> {
        #[derive(sqlx::FromRow)]
        struct Row { key: String, description: String, is_enabled: bool, target_type: String, updated_at: DateTime<Utc> }

        let rows = sqlx::query_as::<_, Row>(
            "SELECT key, description, is_enabled, target_type, updated_at FROM feature_flags ORDER BY key ASC"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        Ok(rows.into_iter().map(|r| FeatureFlagItem {
            key: r.key,
            description: r.description,
            is_enabled: r.is_enabled,
            target_type: r.target_type,
            updated_at: r.updated_at,
        }).collect())
    }

    async fn set_feature_flag(&self, key: &str, is_enabled: bool, updated_by: UserId) -> Result<(), AppError> {
        sqlx::query("UPDATE feature_flags SET is_enabled = $1, updated_by = $2, updated_at = NOW() WHERE key = $3")
            .bind(is_enabled)
            .bind(updated_by.0)
            .bind(key)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::internal(e))?;

        Ok(())
    }

    async fn is_feature_enabled(&self, key: &str) -> Result<bool, AppError> {
        let row: Option<(bool,)> = sqlx::query_as("SELECT is_enabled FROM feature_flags WHERE key = $1")
            .bind(key)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AppError::internal(e))?;

        Ok(row.map(|r| r.0).unwrap_or(true)) // default true if flag not present
    }

    async fn list_runtime_configs(&self) -> Result<Vec<SystemConfigItem>, AppError> {
        #[derive(sqlx::FromRow)]
        struct Row { key: String, value_json: serde_json::Value, category: String, updated_at: DateTime<Utc> }

        let rows = sqlx::query_as::<_, Row>(
            "SELECT key, value_json, category, updated_at FROM system_runtime_configs ORDER BY category ASC, key ASC"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        Ok(rows.into_iter().map(|r| SystemConfigItem {
            key: r.key,
            value_json: r.value_json,
            category: r.category,
            updated_at: r.updated_at,
        }).collect())
    }

    async fn set_runtime_config(&self, key: &str, value: serde_json::Value, category: &str, updated_by: UserId) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO system_runtime_configs (key, value_json, category, updated_by, updated_at) 
             VALUES ($1, $2, $3, $4, NOW())
             ON CONFLICT (key) DO UPDATE SET value_json = EXCLUDED.value_json, category = EXCLUDED.category, updated_by = EXCLUDED.updated_by, updated_at = NOW()"
        )
        .bind(key)
        .bind(value)
        .bind(category)
        .bind(updated_by.0)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        Ok(())
    }

    async fn list_audit_logs(&self, limit: usize) -> Result<Vec<AuditLogEntryDto>, AppError> {
        #[derive(sqlx::FromRow)]
        struct Row { id: Uuid, actor_user_id: Option<Uuid>, action: String, ip_address: Option<String>, metadata: Option<serde_json::Value>, created_at: DateTime<Utc> }

        let rows = sqlx::query_as::<_, Row>(
            "SELECT id, actor_user_id, action, ip_address, metadata, created_at FROM audit_logs ORDER BY created_at DESC LIMIT $1"
        )
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        Ok(rows.into_iter().map(|r| AuditLogEntryDto {
            id: r.id,
            actor_id: r.actor_user_id.map(UserId::from_uuid),
            action: r.action,
            ip_address: r.ip_address,
            metadata: r.metadata,
            created_at: r.created_at,
        }).collect())
    }
}