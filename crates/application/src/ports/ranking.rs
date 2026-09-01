use async_trait::async_trait;
use domain::ranking::{RankingExplanation, RankingProfileType, RawRankingFeatures};
use shared::BusinessId;
use crate::errors::AppError;

pub struct RankingCandidate {
    pub raw_features: RawRankingFeatures,
}

pub struct RankedItem {
    pub business_id: BusinessId,
    pub final_score: f32,
    pub explanation: RankingExplanation,
}

#[derive(Debug, Clone)]
pub struct RankingContext {
    pub profile: RankingProfileType,
    pub has_geo_query: bool,
    pub max_radius_km: f64,
}

#[async_trait]
pub trait RankingEnginePort: Send + Sync {
    fn rank(&self, candidates: Vec<RankingCandidate>, context: &RankingContext) -> Result<Vec<RankedItem>, AppError>;
}