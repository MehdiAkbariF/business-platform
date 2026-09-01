use serde::{Deserialize, Serialize};
use shared::{BusinessId, CategoryId, UserId};
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RecommendationSurface {
    Home,
    BusinessDetail,
    Category,
    City,
    SearchEmpty,
    PostAction,
}

impl std::fmt::Display for RecommendationSurface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Home => write!(f, "HOME"),
            Self::BusinessDetail => write!(f, "BUSINESS_DETAIL"),
            Self::Category => write!(f, "CATEGORY"),
            Self::City => write!(f, "CITY"),
            Self::SearchEmpty => write!(f, "SEARCH_EMPTY"),
            Self::PostAction => write!(f, "POST_ACTION"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CandidateSource {
    Nearby,
    Popular,
    Trending,
    Fresh,
    Similar,
    Category,
    Personalized,
}

impl std::fmt::Display for CandidateSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Nearby => write!(f, "NEARBY"),
            Self::Popular => write!(f, "POPULAR"),
            Self::Trending => write!(f, "TRENDING"),
            Self::Fresh => write!(f, "FRESH"),
            Self::Similar => write!(f, "SIMILAR"),
            Self::Category => write!(f, "CATEGORY"),
            Self::Personalized => write!(f, "PERSONALIZED"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RecommendationCandidate {
    pub business_id: BusinessId,
    pub sources: Vec<CandidateSource>,
}

#[derive(Debug, Clone)]
pub struct RecommendationContext {
    pub user_id: Option<UserId>,
    pub surface: RecommendationSurface,
    pub business_id: Option<BusinessId>,
    pub category_id: Option<CategoryId>,
    pub city: Option<String>,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    pub radius_km: Option<f64>,
    pub limit: usize,
    pub strategy_version: &'static str,
}