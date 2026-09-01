use async_trait::async_trait;
use chrono::{DateTime, Utc};
use domain::seo::SitemapEntry;
use shared::CategoryId;
use sqlx::PgPool;
use application::errors::AppError;
use application::ports::seo::SeoRepository;
use uuid::Uuid;

pub struct PostgresSeoRepository {
    pool: PgPool,
}

impl PostgresSeoRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SeoRepository for PostgresSeoRepository {
    async fn find_redirect(&self, source_slug: &str) -> Result<Option<String>, AppError> {
        #[derive(sqlx::FromRow)]
        struct Row { target_slug: String }

        let row = sqlx::query_as::<_, Row>(
            "SELECT target_slug FROM slug_redirects WHERE source_slug = $1 LIMIT 1"
        )
        .bind(source_slug)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        Ok(row.map(|r| r.target_slug))
    }

    async fn register_slug_change(&self, entity_type: &str, old_slug: &str, new_slug: &str) -> Result<(), AppError> {
        let mut tx = self.pool.begin().await.map_err(|e| AppError::internal(e))?;

        // Prevent redirect chains: If old_slug was already pointed to by others, update them directly to new_slug
        sqlx::query("UPDATE slug_redirects SET target_slug = $1 WHERE target_slug = $2")
            .bind(new_slug)
            .bind(old_slug)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::internal(e))?;

        // Insert direct mapping
        sqlx::query(
            "INSERT INTO slug_redirects (id, entity_type, source_slug, target_slug, http_status, created_at) 
             VALUES ($1, $2, $3, $4, 301, NOW())
             ON CONFLICT (source_slug) DO UPDATE SET target_slug = EXCLUDED.target_slug"
        )
        .bind(Uuid::now_v7())
        .bind(entity_type)
        .bind(old_slug)
        .bind(new_slug)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::internal(e))?;

        tx.commit().await.map_err(|e| AppError::internal(e))?;
        Ok(())
    }

    async fn count_published_businesses_in_category_city(&self, category_id: CategoryId, city: &str) -> Result<usize, AppError> {
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(DISTINCT b.id) 
             FROM businesses b
             INNER JOIN business_categories bc ON b.id = bc.business_id
             INNER JOIN business_locations loc ON b.id = loc.business_id AND loc.is_primary = TRUE
             WHERE b.status = 'PUBLISHED' AND bc.category_id = $1 AND loc.city = $2"
        )
        .bind(category_id.0)
        .bind(city)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        Ok(count.0 as usize)
    }

    async fn get_indexable_businesses_sitemap(&self, limit: usize, offset: usize) -> Result<Vec<SitemapEntry>, AppError> {
        #[derive(sqlx::FromRow)]
        struct Row { slug: String, updated_at: DateTime<Utc> }

        let rows = sqlx::query_as::<_, Row>(
            "SELECT slug, updated_at FROM businesses WHERE status = 'PUBLISHED' ORDER BY updated_at DESC LIMIT $1 OFFSET $2"
        )
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        Ok(rows.into_iter().map(|r| SitemapEntry {
            loc: r.slug,
            lastmod: r.updated_at,
            changefreq: Some("weekly".to_string()),
            priority: Some(0.8),
        }).collect())
    }

    async fn get_active_categories_sitemap(&self) -> Result<Vec<SitemapEntry>, AppError> {
        #[derive(sqlx::FromRow)]
        struct Row { slug: String, updated_at: DateTime<Utc> }

        let rows = sqlx::query_as::<_, Row>(
            "SELECT slug, updated_at FROM categories WHERE status = 'ACTIVE' ORDER BY sort_order ASC"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        Ok(rows.into_iter().map(|r| SitemapEntry {
            loc: r.slug,
            lastmod: r.updated_at,
            changefreq: Some("monthly".to_string()),
            priority: Some(0.6),
        }).collect())
    }

    async fn get_active_cities_sitemap(&self) -> Result<Vec<String>, AppError> {
        #[derive(sqlx::FromRow)]
        struct Row { city: String }

        let rows = sqlx::query_as::<_, Row>(
            "SELECT DISTINCT city FROM business_locations loc INNER JOIN businesses b ON loc.business_id = b.id WHERE b.status = 'PUBLISHED'"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        Ok(rows.into_iter().map(|r| r.city).collect())
    }
}