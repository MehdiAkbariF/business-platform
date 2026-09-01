use async_trait::async_trait;
use chrono::{DateTime, Utc};
use domain::membership::{BusinessMembership, MembershipRole, MembershipStatus};
use shared::{BusinessId, MembershipId, UserId};
use sqlx::PgPool;
use application::errors::AppError;
use application::ports::repositories::MembershipRepository;
use uuid::Uuid;

pub struct PostgresMembershipRepository {
    pool: PgPool,
}

impl PostgresMembershipRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct MembershipRow {
    id: Uuid,
    business_id: Uuid,
    user_id: Uuid,
    role: String,
    status: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<MembershipRow> for BusinessMembership {
    fn from(r: MembershipRow) -> Self {
        let role = match r.role.as_str() {
            "OWNER" => MembershipRole::Owner,
            "ADMIN" => MembershipRole::Admin,
            _ => MembershipRole::Editor,
        };
        let status = match r.status.as_str() {
            "ACTIVE" => MembershipStatus::Active,
            _ => MembershipStatus::Inactive,
        };

        BusinessMembership {
            id: MembershipId::from_uuid(r.id),
            business_id: BusinessId::from_uuid(r.business_id),
            user_id: UserId::from_uuid(r.user_id),
            role,
            status,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

#[async_trait]
impl MembershipRepository for PostgresMembershipRepository {
    async fn find_membership(&self, business_id: BusinessId, user_id: UserId) -> Result<Option<BusinessMembership>, AppError> {
        let row = sqlx::query_as::<_, MembershipRow>(
            "SELECT id, business_id, user_id, role::text, status::text, created_at, updated_at FROM business_memberships WHERE business_id = $1 AND user_id = $2"
        )
        .bind(business_id.0)
        .bind(user_id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        Ok(row.map(BusinessMembership::from))
    }

    async fn list_members(&self, business_id: BusinessId) -> Result<Vec<BusinessMembership>, AppError> {
        let rows = sqlx::query_as::<_, MembershipRow>(
            "SELECT id, business_id, user_id, role::text, status::text, created_at, updated_at FROM business_memberships WHERE business_id = $1 ORDER BY created_at ASC"
        )
        .bind(business_id.0)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        Ok(rows.into_iter().map(BusinessMembership::from).collect())
    }

    async fn add_member(&self, m: &BusinessMembership) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO business_memberships (id, business_id, user_id, role, status, created_at, updated_at) VALUES ($1, $2, $3, $4::membership_role, $5::membership_status, $6, $7)"
        )
        .bind(m.id.0)
        .bind(m.business_id.0)
        .bind(m.user_id.0)
        .bind(m.role.to_string())
        .bind(m.status.to_string())
        .bind(m.created_at)
        .bind(m.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            if let Some(db_err) = e.as_database_error() {
                if db_err.is_unique_violation() {
                    return AppError::Conflict("User is already a member of this business".to_string());
                }
            }
            AppError::internal(e)
        })?;

        Ok(())
    }

    async fn remove_member(&self, business_id: BusinessId, user_id: UserId) -> Result<(), AppError> {
        sqlx::query("DELETE FROM business_memberships WHERE business_id = $1 AND user_id = $2")
            .bind(business_id.0)
            .bind(user_id.0)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::internal(e))?;

        Ok(())
    }

    async fn change_role(&self, business_id: BusinessId, user_id: UserId, role: MembershipRole) -> Result<(), AppError> {
        sqlx::query("UPDATE business_memberships SET role = $1::membership_role, updated_at = NOW() WHERE business_id = $2 AND user_id = $3")
            .bind(role.to_string())
            .bind(business_id.0)
            .bind(user_id.0)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::internal(e))?;

        Ok(())
    }

    async fn count_active_owners(&self, business_id: BusinessId) -> Result<i64, AppError> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM business_memberships WHERE business_id = $1 AND role = 'OWNER' AND status = 'ACTIVE'")
            .bind(business_id.0)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::internal(e))?;

        Ok(count.0)
    }
}