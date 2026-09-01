use application::errors::AppError;
use redis::aio::ConnectionManager;

#[derive(Clone)]
pub struct RedisClient {
    manager: ConnectionManager,
}

impl RedisClient {
    pub async fn new(redis_url: &str) -> Result<Self, AppError> {
        let client = redis::Client::open(redis_url)
            .map_err(|e| AppError::internal(e))?;
        let manager = ConnectionManager::new(client)
            .await
            .map_err(|e| AppError::internal(e))?;

        Ok(Self { manager })
    }

    pub fn manager(&self) -> &ConnectionManager {
        &self.manager
    }

    pub async fn check_health(&self) -> Result<(), AppError> {
        let mut conn = self.manager.clone();
        let pong: String = redis::cmd("PING")
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError::internal(e))?;

        if pong == "PONG" {
            Ok(())
        } else {
            Err(AppError::internal(anyhow::anyhow!("Invalid PING response from Redis")))
        }
    }
}