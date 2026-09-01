use async_trait::async_trait;
use chrono::{DateTime, Utc};
use domain::user::{Email, GlobalRole, User, UserStatus};
use shared::UserId;
use sqlx::PgPool;
use application::errors::AppError;
use application::ports::repositories::UserRepository;
use uuid::Uuid;

pub struct PostgresUserRepository {
    pool: PgPool,
}

impl PostgresUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct UserRow {
    id: Uuid,
    email: String,
    password_hash: String,
    status: String,
    role: String,
    email_verified_at: Option<DateTime<Utc>>,
    last_login_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<UserRow> for User {
    type Error = AppError;
    fn try_from(row: UserRow) -> Result<Self, Self::Error> {
        let email = Email::parse(&row.email).map_err(|e| AppError::internal(anyhow::anyhow!(e)))?;
        let status = match row.status.as_str() {
            "ACTIVE" => UserStatus::Active,
            "SUSPENDED" => UserStatus::Suspended,
            "DEACTIVATED" => UserStatus::Deactivated,
            _ => UserStatus::Active,
        };
        let role = match row.role.as_str() {
            "USER" => GlobalRole::User,
            "MODERATOR" => GlobalRole::Moderator,
            "ADMIN" => GlobalRole::Admin,
            "SUPER_ADMIN" => GlobalRole::SuperAdmin,
            _ => GlobalRole::User,
        };

        Ok(User {
            id: UserId::from_uuid(row.id),
            email,
            password_hash: row.password_hash,
            status,
            role,
            email_verified_at: row.email_verified_at,
            last_login_at: row.last_login_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn find_by_id(&self, id: UserId) -> Result<Option<User>, AppError> {
        let row = sqlx::query_as::<_, UserRow>(
            "SELECT id, email, password_hash, status::text, role::text, email_verified_at, last_login_at, created_at, updated_at FROM users WHERE id = $1"
        )
        .bind(id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        row.map(User::try_from).transpose()
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        let row = sqlx::query_as::<_, UserRow>(
            "SELECT id, email, password_hash, status::text, role::text, email_verified_at, last_login_at, created_at, updated_at FROM users WHERE LOWER(email) = LOWER($1)"
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        row.map(User::try_from).transpose()
    }

    async fn create(&self, user: &User) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO users (id, email, password_hash, status, role, email_verified_at, last_login_at, created_at, updated_at) VALUES ($1, $2, $3, $4::user_status, $5::global_role, $6, $7, $8, $9)"
        )
        .bind(user.id.0)
        .bind(user.email.as_str())
        .bind(&user.password_hash)
        .bind(user.status.to_string())
        .bind(user.role.to_string())
        .bind(user.email_verified_at)
        .bind(user.last_login_at)
        .bind(user.created_at)
        .bind(user.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            if let Some(db_err) = e.as_database_error() {
                if db_err.is_unique_violation() {
                    return AppError::Conflict("An account with this email address already exists".to_string());
                }
            }
            AppError::internal(e)
        })?;

        Ok(())
    }

    async fn update_last_login(&self, id: UserId, login_time: DateTime<Utc>) -> Result<(), AppError> {
        sqlx::query("UPDATE users SET last_login_at = $1, updated_at = $1 WHERE id = $2")
            .bind(login_time)
            .bind(id.0)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::internal(e))?;

        Ok(())
    }
}