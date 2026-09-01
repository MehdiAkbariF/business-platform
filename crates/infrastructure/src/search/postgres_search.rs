use async_trait::async_trait;
use sqlx::PgPool;
use application::errors::AppError;
use application::ports::search::{
    BusinessSearchPort, SearchQueryParams, SearchResponseDto, SearchResultItemDto, SuggestionItemDto, SuggestionType,
};
use shared::BusinessId;
use uuid::Uuid;

pub struct PostgresBusinessSearch {
    pool: PgPool,
    storage_endpoint: String,
    storage_bucket: String,
}

impl PostgresBusinessSearch {
    pub fn new(pool: PgPool, storage_endpoint: String, storage_bucket: String) -> Self {
        Self {
            pool,
            storage_endpoint,
            storage_bucket,
        }
    }
}

#[derive(sqlx::FromRow)]
struct SearchRow {
    id: Uuid,
    slug: String,
    name: String,
    short_description: Option<String>,
    primary_category_name: Option<String>,
    city: Option<String>,
    district: Option<String>,
    formatted_address: Option<String>,
    logo_key: Option<String>,
    verification_status: String,
    distance_meters: Option<f64>,
    score: f32,
}

#[async_trait]
impl BusinessSearchPort for PostgresBusinessSearch {
    async fn search(&self, params: SearchQueryParams, normalized_text: &str, max_radius_km: f64) -> Result<SearchResponseDto, AppError> {
        let limit = params.limit.unwrap_or(20).min(50);
        let radius = params.radius_km.unwrap_or(10.0).min(max_radius_km) * 1000.0; // convert km to meters

        let search_text = if normalized_text.is_empty() { "%".to_string() } else { format!("%{}%", normalized_text) };
        let has_geo = params.lat.is_some() && params.lon.is_some();
        let lat = params.lat.unwrap_or(0.0);
        let lon = params.lon.unwrap_or(0.0);

        let rows = sqlx::query_as::<_, SearchRow>(
            "SELECT 
                b.id,
                b.slug,
                b.name,
                b.short_description,
                c.name AS primary_category_name,
                loc.city,
                loc.district,
                loc.formatted_address,
                bm.storage_key AS logo_key,
                b.verification_status::text,
                CASE WHEN $2 THEN ST_Distance(loc.geom::geography, ST_SetSRID(ST_MakePoint($4, $3), 4326)::geography) ELSE NULL END AS distance_meters,
                similarity(b.name, $5) AS score
             FROM businesses b
             LEFT JOIN business_categories bc ON b.id = bc.business_id AND bc.is_primary = TRUE
             LEFT JOIN categories c ON bc.category_id = c.id
             LEFT JOIN business_locations loc ON b.id = loc.business_id AND loc.is_primary = TRUE
             LEFT JOIN business_media bm ON b.id = bm.business_id AND bm.media_type = 'LOGO' AND bm.status = 'ACTIVE'
             WHERE b.status = 'PUBLISHED'
               AND ($5 = '%' OR (b.name ILIKE $5 OR b.short_description ILIKE $5 OR c.name ILIKE $5))
               AND ($6::uuid IS NULL OR bc.category_id = $6)
               AND ($7::text IS NULL OR loc.city = $7)
               AND ($8::boolean IS NULL OR (b.verification_status = 'VERIFIED') = $8)
               AND (NOT $2 OR (loc.geom IS NOT NULL AND ST_DWithin(loc.geom::geography, ST_SetSRID(ST_MakePoint($4, $3), 4326)::geography, $9)))
             ORDER BY 
                score DESC,
                distance_meters ASC NULLS LAST,
                b.id ASC
             LIMIT $1"
        )
        .bind(limit as i64 + 1)
        .bind(has_geo)
        .bind(lat)
        .bind(lon)
        .bind(&search_text)
        .bind(params.category_id.map(|c| c.0))
        .bind(params.city)
        .bind(params.is_verified)
        .bind(radius)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        let has_more = rows.len() > limit;
        let result_rows = if has_more { &rows[..limit] } else { &rows[..] };

        let items: Vec<SearchResultItemDto> = result_rows.iter().map(|r| {
            let logo_url = r.logo_key.as_ref().map(|k| format!("{}/{}/{}", self.storage_endpoint, self.storage_bucket, k));
            SearchResultItemDto {
                id: BusinessId::from_uuid(r.id),
                slug: r.slug.clone(),
                name: r.name.clone(),
                short_description: r.short_description.clone(),
                primary_category_name: r.primary_category_name.clone(),
                city: r.city.clone(),
                district: r.district.clone(),
                formatted_address: r.formatted_address.clone(),
                distance_meters: r.distance_meters,
                is_verified: r.verification_status == "VERIFIED",
                logo_url,
                score: r.score,
            }
        }).collect();

        let total_count = items.len();
        let next_cursor = if has_more { items.last().map(|i| i.id.to_string()) } else { None };

        Ok(SearchResponseDto {
            items,
            next_cursor,
            total_count,
            has_more,
        })
    }

    async fn autocomplete(&self, normalized_prefix: &str, limit: usize) -> Result<Vec<SuggestionItemDto>, AppError> {
        let pattern = format!("%{}%", normalized_prefix);
        let safe_limit = limit.min(10) as i64;

        #[derive(sqlx::FromRow)]
        struct SugRow {
            title: String,
            slug: String,
            stype: String,
            subtitle: Option<String>,
        }

        let rows = sqlx::query_as::<_, SugRow>(
            "(SELECT name AS title, slug, 'BUSINESS' AS stype, short_description AS subtitle FROM businesses WHERE status = 'PUBLISHED' AND name ILIKE $1 LIMIT $2)
             UNION ALL
             (SELECT name AS title, slug, 'CATEGORY' AS stype, NULL AS subtitle FROM categories WHERE status = 'ACTIVE' AND name ILIKE $1 LIMIT $2)
             UNION ALL
             (SELECT name AS title, slug, 'SERVICE' AS stype, NULL AS subtitle FROM services WHERE status = 'ACTIVE' AND name ILIKE $1 LIMIT $2)
             LIMIT $2"
        )
        .bind(&pattern)
        .bind(safe_limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        Ok(rows.into_iter().map(|r| {
            let suggestion_type = match r.stype.as_str() {
                "CATEGORY" => SuggestionType::Category,
                "SERVICE" => SuggestionType::Service,
                _ => SuggestionType::Business,
            };
            SuggestionItemDto {
                title: r.title,
                slug: r.slug,
                suggestion_type,
                subtitle: r.subtitle,
            }
        }).collect())
    }

    async fn reindex_business(&self, _business_id: BusinessId) -> Result<(), AppError> {
        // Postgres implementation is synchronous/immediate via DB triggers and indexes
        Ok(())
    }

    async fn reindex_all(&self) -> Result<usize, AppError> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM businesses WHERE status = 'PUBLISHED'")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::internal(e))?;
        Ok(count.0 as usize)
    }
}