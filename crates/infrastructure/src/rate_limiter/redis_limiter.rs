use async_trait::async_trait;
use application::errors::AppError;
use application::ports::security::RateLimiterPort;
use crate::redis::RedisClient;

pub struct RedisRateLimiter {
    redis: RedisClient,
}

impl RedisRateLimiter {
    pub fn new(redis: RedisClient) -> Self {
        Self { redis }
    }
}

#[async_trait]
impl RateLimiterPort for RedisRateLimiter {
    async fn check_rate_limit(&self, key: &str, max_requests: u32, window_seconds: u64) -> Result<bool, AppError> {
        let mut conn = self.redis.manager().clone();
        let redis_key = format!("rl:{}", key);

        let count: u32 = redis::cmd("INCR")
            .arg(&redis_key)
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError::internal(e))?;

        if count == 1 {
            let _: () = redis::cmd("EXPIRE")
                .arg(&redis_key)
                .arg(window_seconds)
                .query_async(&mut conn)
                .await
                .map_err(|e| AppError::internal(e))?;
        }

        Ok(count <= max_requests)
    }
}