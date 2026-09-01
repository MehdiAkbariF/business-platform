use async_trait::async_trait;
use domain::recommendation::{CandidateSource, RecommendationCandidate, RecommendationContext};
use shared::BusinessId;
use utoipa::ToSchema;
use serde::Serialize;
use crate::errors::AppError;
use crate::ports::search::SearchResultItemDto;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct RecommendationItemDto {
    pub business: SearchResultItemDto,
    pub sources: Vec<CandidateSource>,
    pub final_score: f32,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct RecommendationResponseDto {
    pub items: Vec<RecommendationItemDto>,
    pub strategy_version: &'static str,
    pub total_candidates_evaluated: usize,
}

#[async_trait]
pub trait RecommendationRepository: Send + Sync {
    async fn get_nearby_candidates(&self, lat: f64, lon: f64, radius_km: f64, limit: usize) -> Result<Vec<BusinessId>, AppError>;
    async fn get_popular_candidates(&self, city: Option<&str>, limit: usize) -> Result<Vec<BusinessId>, AppError>;
    async fn get_trending_candidates(&self, limit: usize) -> Result<Vec<BusinessId>, AppError>;
    async fn get_fresh_candidates(&self, limit: usize) -> Result<Vec<BusinessId>, AppError>;
    async fn get_similar_candidates(&self, target_business_id: BusinessId, limit: usize) -> Result<Vec<BusinessId>, AppError>;
    async fn get_category_candidates(&self, category_id: shared::CategoryId, limit: usize) -> Result<Vec<BusinessId>, AppError>;
}

#[async_trait]
pub trait RecommendationEnginePort: Send + Sync {
    async fn generate_recommendations(&self, context: &RecommendationContext) -> Result<Vec<RecommendationCandidate>, AppError>;
}