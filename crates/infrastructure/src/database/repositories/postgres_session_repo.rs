use async_trait::async_trait;
use chrono::{DateTime, Utc};
use domain::session::Session;
use shared::{SessionId, TokenFamilyId, UserId};
use sqlx::PgPool;
use application::errors::AppError;
use application::ports::repositories::SessionRepository;
use uuid::Uuid;

pub struct PostgresSessionRepository {
    pool: PgPool,
}

impl PostgresSessionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct SessionRow {
    id: Uuid,
    user_id: Uuid,
    token_family_id: Uuid,
    refresh_token_hash: String,
    user_agent: Option<String>,
    ip_address: Option<String>,
    expires_at: DateTime<Utc>,
    revoked_at: Option<DateTime<Utc>>,
    last_used_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
}

impl From<SessionRow> for Session {
    fn from(row: SessionRow) -> Self {
        Session {
            id: SessionId::from_uuid(row.id),
            user_id: UserId::from_uuid(row.user_id),
            token_family_id: TokenFamilyId::from_uuid(row.token_family_id),
            refresh_token_hash: row.refresh_token_hash,
            user_agent: row.user_agent,
            ip_address: row.ip_address,
            expires_at: row.expires_at,
            revoked_at: row.revoked_at,
            last_used_at: row.last_used_at,
            created_at: row.created_at,
        }
    }
}

#[async_trait]
impl SessionRepository for PostgresSessionRepository {
    async fn create(&self, session: &Session) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO sessions (id, user_id, token_family_id, refresh_token_hash, user_agent, ip_address, expires_at, revoked_at, last_used_at, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"
        )
        .bind(session.id.0)
        .bind(session.user_id.0)
        .bind(session.token_family_id.0)
        .bind(&session.refresh_token_hash)
        .bind(&session.user_agent)
        .bind(&session.ip_address)
        .bind(session.expires_at)
        .bind(session.revoked_at)
        .bind(session.last_used_at)
        .bind(session.created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        Ok(())
    }

    async fn find_by_id(&self, id: SessionId) -> Result<Option<Session>, AppError> {
        let row = sqlx::query_as::<_, SessionRow>("SELECT * FROM sessions WHERE id = $1")
            .bind(id.0)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AppError::internal(e))?;

        Ok(row.map(Session::from))
    }

    async fn find_by_refresh_token_hash_for_update(&self, hash: &str) -> Result<Option<Session>, AppError> {
        let row = sqlx::query_as::<_, SessionRow>(
            "SELECT * FROM sessions WHERE refresh_token_hash = $1 FOR UPDATE"
        )
        .bind(hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        Ok(row.map(Session::from))
    }

    async fn rotate_token(&self, old_session_id: SessionId, new_session: &Session) -> Result<(), AppError> {
        let mut tx = self.pool.begin().await.map_err(|e| AppError::internal(e))?;

        // 1. Revoke the old session
        sqlx::query("UPDATE sessions SET revoked_at = NOW(), last_used_at = NOW() WHERE id = $1")
            .bind(old_session_id.0)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::internal(e))?;

        // 2. Insert new session
        sqlx::query(
            "INSERT INTO sessions (id, user_id, token_family_id, refresh_token_hash, user_agent, ip_address, expires_at, revoked_at, last_used_at, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"
        )
        .bind(new_session.id.0)
        .bind(new_session.user_id.0)
        .bind(new_session.token_family_id.0)
        .bind(&new_session.refresh_token_hash)
        .bind(&new_session.user_agent)
        .bind(&new_session.ip_address)
        .bind(new_session.expires_at)
        .bind(new_session.revoked_at)
        .bind(new_session.last_used_at)
        .bind(new_session.created_at)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::internal(e))?;

        tx.commit().await.map_err(|e| AppError::internal(e))?;
        Ok(())
    }

    async fn revoke_session(&self, id: SessionId) -> Result<(), AppError> {
        sqlx::query("UPDATE sessions SET revoked_at = NOW() WHERE id = $1")
            .bind(id.0)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::internal(e))?;

        Ok(())
    }

    async fn revoke_token_family(&self, family_id: TokenFamilyId) -> Result<(), AppError> {
        sqlx::query("UPDATE sessions SET revoked_at = NOW() WHERE token_family_id = $1 AND revoked_at IS NULL")
            .bind(family_id.0)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::internal(e))?;

        Ok(())
    }

    async fn revoke_all_user_sessions(&self, user_id: UserId) -> Result<(), AppError> {
        sqlx::query("UPDATE sessions SET revoked_at = NOW() WHERE user_id = $1 AND revoked_at IS NULL")
            .bind(user_id.0)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::internal(e))?;

        Ok(())
    }
}