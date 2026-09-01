use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    // Server & Environment
    pub app_env: String,
    pub server_host: String,
    pub server_port: u16,

    // Database (PostgreSQL / PostGIS)
    pub database_url: String,
    pub database_max_connections: u32,
    pub database_min_connections: u32,

    // Cache & Rate Limiting (Redis)
    pub redis_url: String,

    // Object Storage (MinIO / S3)
    pub storage_endpoint: String,
    pub storage_bucket: String,
    pub storage_access_key: String,
    pub storage_secret_key: String,
    pub storage_region: String,

    // Security & CORS
    pub cors_allowed_origins: Vec<String>,

    // Authentication & Tokens (Phase 12)
    pub jwt_secret: String,
    pub access_token_ttl_seconds: i64,
    pub refresh_token_ttl_seconds: i64,
    pub auth_rate_limit_per_minute: u32,

    // Taxonomy Limits (Phase 14)
    pub max_business_categories: usize,
    pub max_business_services: usize,
    pub max_taxonomy_depth: usize,

    // Search Engine & Geo Limits (Phase 17)
    pub max_search_radius_km: f64,
    pub max_search_limit: usize,
}

impl AppConfig {
    pub fn load() -> Result<Self, anyhow::Error> {
        let _ = dotenvy::dotenv();

        // 1. Environment & Server
        let app_env = env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());
        let server_host = env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let server_port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()?;

        // 2. Database
        let database_url = env::var("DATABASE_URL")
            .map_err(|_| anyhow::anyhow!("DATABASE_URL is a required environment variable"))?;
        let database_max_connections = env::var("DATABASE_MAX_CONNECTIONS")
            .unwrap_or_else(|_| "20".to_string())
            .parse::<u32>()?;
        let database_min_connections = env::var("DATABASE_MIN_CONNECTIONS")
            .unwrap_or_else(|_| "5".to_string())
            .parse::<u32>()?;

        // 3. Redis
        let redis_url = env::var("REDIS_URL")
            .map_err(|_| anyhow::anyhow!("REDIS_URL is a required environment variable"))?;

        // 4. Object Storage
        let storage_endpoint = env::var("STORAGE_ENDPOINT")
            .map_err(|_| anyhow::anyhow!("STORAGE_ENDPOINT is a required environment variable"))?;
        let storage_bucket = env::var("STORAGE_BUCKET")
            .map_err(|_| anyhow::anyhow!("STORAGE_BUCKET is a required environment variable"))?;
        let storage_access_key = env::var("STORAGE_ACCESS_KEY")
            .map_err(|_| anyhow::anyhow!("STORAGE_ACCESS_KEY is a required environment variable"))?;
        let storage_secret_key = env::var("STORAGE_SECRET_KEY")
            .map_err(|_| anyhow::anyhow!("STORAGE_SECRET_KEY is a required environment variable"))?;
        let storage_region = env::var("STORAGE_REGION").unwrap_or_else(|_| "us-east-1".to_string());

        // 5. CORS
        let origins = env::var("CORS_ALLOWED_ORIGINS").unwrap_or_else(|_| "".to_string());
        let cors_allowed_origins = if origins.trim().is_empty() {
            vec!["*".to_string()]
        } else {
            origins.split(',').map(|s| s.trim().to_string()).collect()
        };

        // 6. Auth Configuration
        let jwt_secret = env::var("JWT_SECRET")
            .unwrap_or_else(|_| "default_insecure_jwt_secret_must_be_changed_in_prod_at_least_32_bytes".to_string());
        let access_token_ttl_seconds = env::var("ACCESS_TOKEN_TTL_SECONDS")
            .unwrap_or_else(|_| "900".to_string())
            .parse::<i64>()?;
        let refresh_token_ttl_seconds = env::var("REFRESH_TOKEN_TTL_SECONDS")
            .unwrap_or_else(|_| "2592000".to_string())
            .parse::<i64>()?;
        let auth_rate_limit_per_minute = env::var("AUTH_RATE_LIMIT_PER_MINUTE")
            .unwrap_or_else(|_| "20".to_string())
            .parse::<u32>()?;

        // 7. Taxonomy Limits
        let max_business_categories = env::var("MAX_BUSINESS_CATEGORIES")
            .unwrap_or_else(|_| "5".to_string())
            .parse::<usize>()?;
        let max_business_services = env::var("MAX_BUSINESS_SERVICES")
            .unwrap_or_else(|_| "20".to_string())
            .parse::<usize>()?;
        let max_taxonomy_depth = env::var("MAX_TAXONOMY_DEPTH")
            .unwrap_or_else(|_| "4".to_string())
            .parse::<usize>()?;

        // 8. Search & Geo Limits
        let max_search_radius_km = env::var("MAX_SEARCH_RADIUS_KM")
            .unwrap_or_else(|_| "50.0".to_string())
            .parse::<f64>()?;
        let max_search_limit = env::var("MAX_SEARCH_LIMIT")
            .unwrap_or_else(|_| "50".to_string())
            .parse::<usize>()?;

        Ok(Self {
            app_env,
            server_host,
            server_port,
            database_url,
            database_max_connections,
            database_min_connections,
            redis_url,
            storage_endpoint,
            storage_bucket,
            storage_access_key,
            storage_secret_key,
            storage_region,
            cors_allowed_origins,
            jwt_secret,
            access_token_ttl_seconds,
            refresh_token_ttl_seconds,
            auth_rate_limit_per_minute,
            max_business_categories,
            max_business_services,
            max_taxonomy_depth,
            max_search_radius_km,
            max_search_limit,
        })
    }
}