use async_trait::async_trait;
use sqlx::PgPool;
use application::errors::AppError;
use application::ports::recommendation::RecommendationRepository;
use shared::{BusinessId, CategoryId};
use uuid::Uuid;

pub struct PostgresRecommendationRepository {
    pool: PgPool,
}

impl PostgresRecommendationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RecommendationRepository for PostgresRecommendationRepository {
    async fn get_nearby_candidates(&self, lat: f64, lon: f64, radius_km: f64, limit: usize) -> Result<Vec<BusinessId>, AppError> {
        let radius_m = radius_km * 1000.0;
        #[derive(sqlx::FromRow)]
        struct Row { id: Uuid }

        let rows = sqlx::query_as::<_, Row>(
            "SELECT b.id 
             FROM businesses b
             INNER JOIN business_locations loc ON b.id = loc.business_id AND loc.is_primary = TRUE
             WHERE b.status = 'PUBLISHED'
               AND ST_DWithin(loc.geom::geography, ST_SetSRID(ST_MakePoint($2, $1), 4326)::geography, $3)
             ORDER BY ST_Distance(loc.geom::geography, ST_SetSRID(ST_MakePoint($2, $1), 4326)::geography) ASC
             LIMIT $4"
        )
        .bind(lat)
        .bind(lon)
        .bind(radius_m)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        Ok(rows.into_iter().map(|r| BusinessId::from_uuid(r.id)).collect())
    }

    async fn get_popular_candidates(&self, city: Option<&str>, limit: usize) -> Result<Vec<BusinessId>, AppError> {
        #[derive(sqlx::FromRow)]
        struct Row { id: Uuid }

        let rows = sqlx::query_as::<_, Row>(
            "SELECT b.id 
             FROM businesses b
             LEFT JOIN business_analytics ba ON b.id = ba.business_id
             LEFT JOIN business_locations loc ON b.id = loc.business_id AND loc.is_primary = TRUE
             WHERE b.status = 'PUBLISHED'
               AND ($1::text IS NULL OR loc.city = $1)
             ORDER BY COALESCE(ba.views_count, 0) + COALESCE(ba.clicks_count * 5, 0) DESC, b.created_at DESC
             LIMIT $2"
        )
        .bind(city)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        Ok(rows.into_iter().map(|r| BusinessId::from_uuid(r.id)).collect())
    }

    async fn get_trending_candidates(&self, limit: usize) -> Result<Vec<BusinessId>, AppError> {
        #[derive(sqlx::FromRow)]
        struct Row { id: Uuid }

        let rows = sqlx::query_as::<_, Row>(
            "SELECT b.id 
             FROM businesses b
             LEFT JOIN business_analytics ba ON b.id = ba.business_id
             WHERE b.status = 'PUBLISHED'
             ORDER BY COALESCE(ba.recent_views, 0) + COALESCE(ba.recent_clicks * 10, 0) DESC, b.published_at DESC NULLS LAST
             LIMIT $1"
        )
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        Ok(rows.into_iter().map(|r| BusinessId::from_uuid(r.id)).collect())
    }

    async fn get_fresh_candidates(&self, limit: usize) -> Result<Vec<BusinessId>, AppError> {
        #[derive(sqlx::FromRow)]
        struct Row { id: Uuid }

        let rows = sqlx::query_as::<_, Row>(
            "SELECT id FROM businesses WHERE status = 'PUBLISHED' ORDER BY published_at DESC NULLS LAST, created_at DESC LIMIT $1"
        )
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        Ok(rows.into_iter().map(|r| BusinessId::from_uuid(r.id)).collect())
    }

    async fn get_similar_candidates(&self, target_business_id: BusinessId, limit: usize) -> Result<Vec<BusinessId>, AppError> {
        #[derive(sqlx::FromRow)]
        struct Row { id: Uuid }

        let rows = sqlx::query_as::<_, Row>(
            "WITH target_cats AS (
                SELECT category_id FROM business_categories WHERE business_id = $1
            )
            SELECT DISTINCT b.id 
            FROM businesses b
            INNER JOIN business_categories bc ON b.id = bc.business_id
            INNER JOIN target_cats tc ON bc.category_id = tc.category_id
            WHERE b.id != $1 AND b.status = 'PUBLISHED'
            LIMIT $2"
        )
        .bind(target_business_id.0)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        Ok(rows.into_iter().map(|r| BusinessId::from_uuid(r.id)).collect())
    }

    async fn get_category_candidates(&self, category_id: CategoryId, limit: usize) -> Result<Vec<BusinessId>, AppError> {
        #[derive(sqlx::FromRow)]
        struct Row { id: Uuid }

        let rows = sqlx::query_as::<_, Row>(
            "SELECT b.id 
             FROM businesses b
             INNER JOIN business_categories bc ON b.id = bc.business_id
             WHERE bc.category_id = $1 AND b.status = 'PUBLISHED'
             ORDER BY bc.is_primary DESC, b.created_at DESC
             LIMIT $2"
        )
        .bind(category_id.0)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        Ok(rows.into_iter().map(|r| BusinessId::from_uuid(r.id)).collect())
    }
}