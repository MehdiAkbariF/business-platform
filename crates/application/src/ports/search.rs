use async_trait::async_trait;
use shared::{BusinessId, CategoryId, ServiceId};
use utoipa::ToSchema;
use serde::{Deserialize, Serialize};
use crate::errors::AppError;

#[derive(Debug, Clone, Deserialize)]
pub struct SearchQueryParams {
    pub q: Option<String>,
    pub category_id: Option<CategoryId>,
    pub service_id: Option<ServiceId>,
    pub city: Option<String>,
    pub district: Option<String>,
    pub is_verified: Option<bool>,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    pub radius_km: Option<f64>,
    pub min_lat: Option<f64>,
    pub max_lat: Option<f64>,
    pub min_lon: Option<f64>,
    pub max_lon: Option<f64>,
    pub cursor: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct SearchResultItemDto {
    pub id: BusinessId,
    pub slug: String,
    pub name: String,
    pub short_description: Option<String>,
    pub primary_category_name: Option<String>,
    pub city: Option<String>,
    pub district: Option<String>,
    pub formatted_address: Option<String>,
    pub distance_meters: Option<f64>,
    pub is_verified: bool,
    pub logo_url: Option<String>,
    pub score: f32,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct SearchResponseDto {
    pub items: Vec<SearchResultItemDto>,
    pub next_cursor: Option<String>,
    pub total_count: usize,
    pub has_more: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SuggestionType {
    Business,
    Category,
    Service,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct SuggestionItemDto {
    pub title: String,
    pub slug: String,
    pub suggestion_type: SuggestionType,
    pub subtitle: Option<String>,
}

#[async_trait]
pub trait BusinessSearchPort: Send + Sync {
    async fn search(&self, params: SearchQueryParams, normalized_text: &str, max_radius_km: f64) -> Result<SearchResponseDto, AppError>;
    async fn autocomplete(&self, normalized_prefix: &str, limit: usize) -> Result<Vec<SuggestionItemDto>, AppError>;
    async fn reindex_business(&self, business_id: BusinessId) -> Result<(), AppError>;
    async fn reindex_all(&self) -> Result<usize, AppError>;
}