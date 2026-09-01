use std::sync::Arc;
use application::ports::storage::ObjectStoragePort;
use infrastructure::{
    config::AppConfig,
    database::PostgresDatabase,
    redis::RedisClient,
    storage::S3ObjectStorage,
};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub db: PostgresDatabase,
    pub redis: RedisClient,
    pub storage: Arc<dyn ObjectStoragePort>,
}

impl AppState {
    pub async fn init(config: AppConfig) -> Result<Self, anyhow::Error> {
        let config = Arc::new(config);
        let db = PostgresDatabase::new(&config).await?;
        let redis = RedisClient::new(&config.redis_url).await
            .map_err(|e| anyhow::anyhow!("Failed to connect to Redis: {:?}", e))?;
        let storage = Arc::new(S3ObjectStorage::new(&config).await);

        Ok(Self {
            config,
            db,
            redis,
            storage,
        })
    }
}