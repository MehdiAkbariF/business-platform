use std::sync::Arc;
use application::ports::{
    monetization::{MonetizationRepository, PaymentProviderPort},
    ranking::RankingEnginePort,
    recommendation::{RecommendationEnginePort, RecommendationRepository},
    repositories::{
        AuditRepository, BusinessRepository, MembershipRepository, ModerationRepository, ProfileRepository,
        SessionRepository, TaxonomyRepository, UserRepository,
    },
    search::BusinessSearchPort,
    security::{PasswordHasherPort, RateLimiterPort, TokenServicePort},
    seo::SeoRepository,
    storage::ObjectStoragePort,
};
use infrastructure::{
    config::AppConfig,
    database::{
        repositories::{
            postgres_audit_repo::PostgresAuditRepository,
            postgres_business_repo::PostgresBusinessRepository,
            postgres_membership_repo::PostgresMembershipRepository,
            postgres_moderation_repo::PostgresModerationRepository,
            postgres_monetization_repo::PostgresMonetizationRepository,
            postgres_profile_repo::PostgresProfileRepository,
            postgres_recommendation_repo::PostgresRecommendationRepository,
            postgres_seo_repo::PostgresSeoRepository,
            postgres_session_repo::PostgresSessionRepository,
            postgres_taxonomy_repo::PostgresTaxonomyRepository,
            postgres_user_repo::PostgresUserRepository,
        },
        PostgresDatabase,
    },
    monetization::mock_provider::MockPaymentProvider,
    ranking::deterministic_engine::DeterministicRankingEngine,
    rate_limiter::redis_limiter::RedisRateLimiter,
    recommendation::modular_engine::ModularRecommendationEngine,
    redis::RedisClient,
    search::postgres_search::PostgresBusinessSearch,
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
    pub profile_repo: Arc<dyn ProfileRepository>,
    pub moderation_repo: Arc<dyn ModerationRepository>,
    pub rec_repo: Arc<dyn RecommendationRepository>,
    pub search_port: Arc<dyn BusinessSearchPort>,
    pub ranking_engine: Arc<dyn RankingEnginePort>,
    pub rec_engine: Arc<dyn RecommendationEnginePort>,
    pub monetization_repo: Arc<dyn MonetizationRepository>,
    pub payment_provider: Arc<dyn PaymentProviderPort>,
    pub seo_repo: Arc<dyn SeoRepository>,
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
        let profile_repo = Arc::new(PostgresProfileRepository::new(db.pool().clone()));
        let moderation_repo = Arc::new(PostgresModerationRepository::new(db.pool().clone()));
        let rec_repo = Arc::new(PostgresRecommendationRepository::new(db.pool().clone()));
        let search_port = Arc::new(PostgresBusinessSearch::new(
            db.pool().clone(),
            config.storage_endpoint.clone(),
            config.storage_bucket.clone(),
        ));
        let ranking_engine = Arc::new(DeterministicRankingEngine::new(config.max_search_radius_km));
        let rec_engine = Arc::new(ModularRecommendationEngine::new(rec_repo.clone()));
        let monetization_repo = Arc::new(PostgresMonetizationRepository::new(db.pool().clone()));
        let payment_provider = Arc::new(MockPaymentProvider);
        let seo_repo = Arc::new(PostgresSeoRepository::new(db.pool().clone()));
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
            profile_repo,
            moderation_repo,
            rec_repo,
            search_port,
            ranking_engine,
            rec_engine,
            monetization_repo,
            payment_provider,
            seo_repo,
            password_hasher,
            token_service,
            rate_limiter,
        })
    }
}