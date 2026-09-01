use async_trait::async_trait;
use aws_config::BehaviorVersion;
use aws_sdk_s3::config::{Credentials, Region};
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client;
use bytes::Bytes;

use application::errors::AppError;
use application::ports::storage::ObjectStoragePort;
use crate::config::AppConfig;

#[derive(Clone)]
pub struct S3ObjectStorage {
    client: Client,
    bucket: String,
}

impl S3ObjectStorage {
    pub async fn new(config: &AppConfig) -> Self {
        let credentials = Credentials::new(
            &config.storage_access_key,
            &config.storage_secret_key,
            None,
            None,
            "static",
        );

        let s3_config = aws_sdk_s3::config::Builder::new()
            .behavior_version(BehaviorVersion::latest())
            .region(Region::new(config.storage_region.clone()))
            .endpoint_url(&config.storage_endpoint)
            .credentials_provider(credentials)
            .force_path_style(true)
            .build();

        let client = Client::from_conf(s3_config);

        Self {
            client,
            bucket: config.storage_bucket.clone(),
        }
    }
}

#[async_trait]
impl ObjectStoragePort for S3ObjectStorage {
    async fn put_object(&self, key: &str, data: Bytes, content_type: &str) -> Result<String, AppError> {
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(ByteStream::from(data))
            .content_type(content_type)
            .send()
            .await
            .map_err(|e| AppError::internal(e))?;

        Ok(key.to_string())
    }

    async fn get_object(&self, key: &str) -> Result<Bytes, AppError> {
        let resp = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| AppError::NotFound(format!("Object '{}' not found: {}", key, e)))?;

        let data = resp
            .body
            .collect()
            .await
            .map_err(|e| AppError::internal(e))?
            .into_bytes();

        Ok(data)
    }

    async fn delete_object(&self, key: &str) -> Result<(), AppError> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| AppError::internal(e))?;

        Ok(())
    }

    async fn check_health(&self) -> Result<(), AppError> {
        self.client
            .head_bucket()
            .bucket(&self.bucket)
            .send()
            .await
            .map_err(|e| AppError::internal(e))?;

        Ok(())
    }
}