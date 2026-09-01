use std::sync::Arc;
use application::ports::{
    repositories::{AuditRepository, BusinessRepository, MembershipRepository, SessionRepository, TaxonomyRepository, UserRepository},
    security::{PasswordHasherPort, RateLimiterPort, TokenServicePort},
    storage::ObjectStoragePort,
};
use infrastructure::{
    config::AppConfig,
    database::{
        repositories::{
            postgres_audit_repo::PostgresAuditRepository,
            postgres_business_repo::PostgresBusinessRepository,
            postgres_membership_repo::PostgresMembershipRepository,
            postgres_session_repo::PostgresSessionRepository,
            postgres_taxonomy_repo::PostgresTaxonomyRepository,
            postgres_user_repo::PostgresUserRepository,
        },
        PostgresDatabase,
    },
    rate_limiter::redis_limiter::RedisRateLimiter,
    redis::RedisClient,
    security::{argon2_hasher::Argon2PasswordHasher, jwt_token_service::JwtTokenService},
    storage::S3ObjectStorage,
};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub db: PostgresDatabase,
    pub redis: RedisClient,
    pub storage: Arc<dyn ObjectStoragePort>,
    pub user_repo: Arc<dyn UserRepository>,
    pub session_repo: Arc<dyn SessionRepository>,
    pub audit_repo: Arc<dyn AuditRepository>,
    pub business_repo: Arc<dyn BusinessRepository>,
    pub membership_repo: Arc<dyn MembershipRepository>,
    pub taxonomy_repo: Arc<dyn TaxonomyRepository>,
    pub password_hasher: Arc<dyn PasswordHasherPort>,
    pub token_service: Arc<dyn TokenServicePort>,
    pub rate_limiter: Arc<dyn RateLimiterPort>,
}

impl AppState {
    pub async fn init(config: AppConfig) -> Result<Self, anyhow::Error> {
        let config = Arc::new(config);
        let db = PostgresDatabase::new(&config).await?;
        let redis = RedisClient::new(&config.redis_url).await
            .map_err(|e| anyhow::anyhow!("Failed to connect to Redis: {:?}", e))?;
        let storage = Arc::new(S3ObjectStorage::new(&config).await);

        let user_repo = Arc::new(PostgresUserRepository::new(db.pool().clone()));
        let session_repo = Arc::new(PostgresSessionRepository::new(db.pool().clone()));
        let audit_repo = Arc::new(PostgresAuditRepository::new(db.pool().clone()));
        let business_repo = Arc::new(PostgresBusinessRepository::new(db.pool().clone()));
        let membership_repo = Arc::new(PostgresMembershipRepository::new(db.pool().clone()));
        let taxonomy_repo = Arc::new(PostgresTaxonomyRepository::new(db.pool().clone()));
        let password_hasher = Arc::new(Argon2PasswordHasher);
        let token_service = Arc::new(JwtTokenService::new(
            config.jwt_secret.clone(),
            config.access_token_ttl_seconds,
        ));
        let rate_limiter = Arc::new(RedisRateLimiter::new(redis.clone()));

        Ok(Self {
            config,
            db,
            redis,
            storage,
            user_repo,
            session_repo,
            audit_repo,
            business_repo,
            membership_repo,
            taxonomy_repo,
            password_hasher,
            token_service,
            rate_limiter,
        })
    }
}