use async_trait::async_trait;
use chrono::{DateTime, Utc};
use domain::profile::{BusinessHoursInterval, BusinessMedia, MediaStatus, MediaType, SocialPlatform};
use shared::{AttributeId, BusinessId, MediaId, SocialLinkId};
use sqlx::PgPool;
use application::errors::AppError;
use application::ports::repositories::{BusinessAttributeRow, BusinessSocialLinkRow, ProfileRepository};
use uuid::Uuid;

pub struct PostgresProfileRepository {
    pool: PgPool,
}

impl PostgresProfileRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct MediaRow {
    id: Uuid,
    business_id: Uuid,
    media_type: String,
    storage_key: String,
    mime_type: String,
    size_bytes: i64,
    width: i32,
    height: i32,
    alt_text: Option<String>,
    sort_order: i32,
    status: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<MediaRow> for BusinessMedia {
    fn from(r: MediaRow) -> Self {
        let media_type = match r.media_type.as_str() {
            "LOGO" => MediaType::Logo,
            "COVER" => MediaType::Cover,
            _ => MediaType::Gallery,
        };
        let status = match r.status.as_str() {
            "ACTIVE" => MediaStatus::Active,
            "PROCESSING" => MediaStatus::Processing,
            _ => MediaStatus::Rejected,
        };

        BusinessMedia {
            id: MediaId::from_uuid(r.id),
            business_id: BusinessId::from_uuid(r.business_id),
            media_type,
            storage_key: r.storage_key,
            mime_type: r.mime_type,
            size_bytes: r.size_bytes,
            width: r.width,
            height: r.height,
            alt_text: r.alt_text,
            sort_order: r.sort_order,
            status,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

#[async_trait]
impl ProfileRepository for PostgresProfileRepository {
    async fn save_media(&self, m: &BusinessMedia) -> Result<(), AppError> {
        let mut tx = self.pool.begin().await.map_err(|e| AppError::internal(e))?;

        // Enforce single active logo/cover by deactivating previous ones
        if m.media_type == MediaType::Logo || m.media_type == MediaType::Cover {
            sqlx::query(
                "UPDATE business_media SET status = 'REJECTED'::media_status WHERE business_id = $1 AND media_type = $2::media_type AND status = 'ACTIVE'::media_status"
            )
            .bind(m.business_id.0)
            .bind(m.media_type.to_string())
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::internal(e))?;
        }

        sqlx::query(
            "INSERT INTO business_media (id, business_id, media_type, storage_key, mime_type, size_bytes, width, height, alt_text, sort_order, status, created_at, updated_at) 
             VALUES ($1, $2, $3::media_type, $4, $5, $6, $7, $8, $9, $10, $11::media_status, $12, $13)"
        )
        .bind(m.id.0)
        .bind(m.business_id.0)
        .bind(m.media_type.to_string())
        .bind(&m.storage_key)
        .bind(&m.mime_type)
        .bind(m.size_bytes)
        .bind(m.width)
        .bind(m.height)
        .bind(&m.alt_text)
        .bind(m.sort_order)
        .bind(m.status.to_string())
        .bind(m.created_at)
        .bind(m.updated_at)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::internal(e))?;

        tx.commit().await.map_err(|e| AppError::internal(e))?;
        Ok(())
    }

    async fn get_media(&self, business_id: BusinessId) -> Result<Vec<BusinessMedia>, AppError> {
        let rows = sqlx::query_as::<_, MediaRow>(
            "SELECT id, business_id, media_type::text, storage_key, mime_type, size_bytes, width, height, alt_text, sort_order, status::text, created_at, updated_at 
             FROM business_media WHERE business_id = $1 AND status = 'ACTIVE' ORDER BY sort_order ASC, created_at ASC"
        )
        .bind(business_id.0)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        Ok(rows.into_iter().map(BusinessMedia::from).collect())
    }

    async fn find_media_by_id(&self, media_id: MediaId) -> Result<Option<BusinessMedia>, AppError> {
        let row = sqlx::query_as::<_, MediaRow>(
            "SELECT id, business_id, media_type::text, storage_key, mime_type, size_bytes, width, height, alt_text, sort_order, status::text, created_at, updated_at 
             FROM business_media WHERE id = $1"
        )
        .bind(media_id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        Ok(row.map(BusinessMedia::from))
    }

    async fn delete_media(&self, media_id: MediaId) -> Result<(), AppError> {
        sqlx::query("UPDATE business_media SET status = 'REJECTED'::media_status WHERE id = $1")
            .bind(media_id.0)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::internal(e))?;

        Ok(())
    }

    async fn save_hours(&self, business_id: BusinessId, hours: &[BusinessHoursInterval]) -> Result<(), AppError> {
        let mut tx = self.pool.begin().await.map_err(|e| AppError::internal(e))?;

        sqlx::query("DELETE FROM business_hours WHERE business_id = $1")
            .bind(business_id.0)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::internal(e))?;

        for h in hours {
            let id = Uuid::now_v7();
            sqlx::query(
                "INSERT INTO business_hours (id, business_id, day_of_week, opens_at, closes_at, is_24_hours, sort_order) 
                 VALUES ($1, $2, $3, $4, $5, $6, $7)"
            )
            .bind(id)
            .bind(business_id.0)
            .bind(h.day_of_week as i16)
            .bind(&h.opens_at)
            .bind(&h.closes_at)
            .bind(h.is_24_hours)
            .bind(h.sort_order)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::internal(e))?;
        }

        tx.commit().await.map_err(|e| AppError::internal(e))?;
        Ok(())
    }

    async fn get_hours(&self, business_id: BusinessId) -> Result<Vec<BusinessHoursInterval>, AppError> {
        #[derive(sqlx::FromRow)]
        struct Row {
            day_of_week: i16,
            opens_at: Option<String>,
            closes_at: Option<String>,
            is_24_hours: bool,
            sort_order: i32,
        }

        let rows = sqlx::query_as::<_, Row>(
            "SELECT day_of_week, opens_at, closes_at, is_24_hours, sort_order FROM business_hours WHERE business_id = $1 ORDER BY day_of_week ASC, sort_order ASC"
        )
        .bind(business_id.0)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        Ok(rows.into_iter().map(|r| BusinessHoursInterval {
            day_of_week: r.day_of_week as u8,
            opens_at: r.opens_at,
            closes_at: r.closes_at,
            is_24_hours: r.is_24_hours,
            sort_order: r.sort_order,
        }).collect())
    }

    async fn save_attributes(&self, business_id: BusinessId, attributes: &[(AttributeId, serde_json::Value)]) -> Result<(), AppError> {
        let mut tx = self.pool.begin().await.map_err(|e| AppError::internal(e))?;

        sqlx::query("DELETE FROM business_attributes WHERE business_id = $1")
            .bind(business_id.0)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::internal(e))?;

        for (attr_id, val) in attributes {
            sqlx::query(
                "INSERT INTO business_attributes (business_id, attribute_id, value_json) VALUES ($1, $2, $3)"
            )
            .bind(business_id.0)
            .bind(attr_id.0)
            .bind(val)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::internal(e))?;
        }

        tx.commit().await.map_err(|e| AppError::internal(e))?;
        Ok(())
    }

    async fn get_attributes(&self, business_id: BusinessId) -> Result<Vec<BusinessAttributeRow>, AppError> {
        #[derive(sqlx::FromRow)]
        struct Row {
            attribute_id: Uuid,
            key: String,
            name: String,
            value_json: serde_json::Value,
        }

        let rows = sqlx::query_as::<_, Row>(
            "SELECT ba.attribute_id, ad.key, ad.name, ba.value_json 
             FROM business_attributes ba
             INNER JOIN attribute_definitions ad ON ba.attribute_id = ad.id
             WHERE ba.business_id = $1"
        )
        .bind(business_id.0)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        Ok(rows.into_iter().map(|r| BusinessAttributeRow {
            attribute_id: AttributeId::from_uuid(r.attribute_id),
            key: r.key,
            name: r.name,
            value_json: r.value_json,
        }).collect())
    }

    async fn save_social_links(&self, business_id: BusinessId, links: &[(SocialPlatform, String)]) -> Result<(), AppError> {
        let mut tx = self.pool.begin().await.map_err(|e| AppError::internal(e))?;

        sqlx::query("DELETE FROM business_social_links WHERE business_id = $1")
            .bind(business_id.0)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::internal(e))?;

        for (platform, raw_url) in links {
            let id = Uuid::now_v7();
            sqlx::query(
                "INSERT INTO business_social_links (id, business_id, platform, url, is_public, created_at, updated_at) 
                 VALUES ($1, $2, $3::social_platform, $4, TRUE, NOW(), NOW())"
            )
            .bind(id)
            .bind(business_id.0)
            .bind(platform.to_string())
            .bind(raw_url)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::internal(e))?;
        }

        tx.commit().await.map_err(|e| AppError::internal(e))?;
        Ok(())
    }

    async fn get_social_links(&self, business_id: BusinessId) -> Result<Vec<BusinessSocialLinkRow>, AppError> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            platform: String,
            url: String,
        }

        let rows = sqlx::query_as::<_, Row>(
            "SELECT id, platform::text, url FROM business_social_links WHERE business_id = $1 AND is_public = TRUE"
        )
        .bind(business_id.0)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        Ok(rows.into_iter().map(|r| {
            let platform = match r.platform.as_str() {
                "INSTAGRAM" => SocialPlatform::Instagram,
                "TELEGRAM" => SocialPlatform::Telegram,
                "WHATSAPP" => SocialPlatform::Whatsapp,
                "LINKEDIN" => SocialPlatform::Linkedin,
                "FACEBOOK" => SocialPlatform::Facebook,
                _ => SocialPlatform::Youtube,
            };
            BusinessSocialLinkRow {
                id: SocialLinkId::from_uuid(r.id),
                platform,
                url: r.url,
            }
        }).collect())
    }
}