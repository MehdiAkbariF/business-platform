use async_trait::async_trait;
use domain::seo::{PageMetadata, SitemapEntry, StructuredDataJsonLd};
use shared::{BusinessId, CategoryId};
use utoipa::ToSchema;
use serde::Serialize;
use crate::errors::AppError;
use crate::use_cases::business::PublicBusinessProfileDto;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct BusinessSeoPageDto {
    pub is_redirect: bool,
    pub redirect_target: Option<String>,
    pub metadata: PageMetadata,
    pub structured_data: Option<StructuredDataJsonLd>,
    pub profile: Option<PublicBusinessProfileDto>,
    pub related_businesses_slugs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct CategoryLocationLandingDto {
    pub city: String,
    pub category_name: String,
    pub category_slug: String,
    pub business_count: usize,
    pub is_indexable: bool,
    pub metadata: PageMetadata,
    pub featured_businesses: Vec<PublicBusinessProfileDto>,
}

#[async_trait]
pub trait SeoRepository: Send + Sync {
    // Slug Redirects
    async fn find_redirect(&self, source_slug: &str) -> Result<Option<String>, AppError>;
    async fn register_slug_change(&self, entity_type: &str, old_slug: &str, new_slug: &str) -> Result<(), AppError>;

    // Inventory Checks for Thin Content Prevention
    async fn count_published_businesses_in_category_city(&self, category_id: CategoryId, city: &str) -> Result<usize, AppError>;
    
    // Sitemap Batch Feeds
    async fn get_indexable_businesses_sitemap(&self, limit: usize, offset: usize) -> Result<Vec<SitemapEntry>, AppError>;
    async fn get_active_categories_sitemap(&self) -> Result<Vec<SitemapEntry>, AppError>;
    async fn get_active_cities_sitemap(&self) -> Result<Vec<String>, AppError>;
}