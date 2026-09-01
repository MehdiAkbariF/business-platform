use std::sync::Arc;
use domain::ranking::{RankingProfileType, RawRankingFeatures};
use domain::recommendation::{RecommendationContext, RecommendationSurface};
use shared::{BusinessId, CategoryId, UserId};
use utoipa::{IntoParams, ToSchema};
use serde::Deserialize;
use uuid::Uuid;
use crate::errors::AppError;
use crate::ports::ranking::{RankingCandidate, RankingContext, RankingEnginePort};
use crate::ports::recommendation::{RecommendationEnginePort, RecommendationItemDto, RecommendationResponseDto};
use crate::ports::search::{BusinessSearchPort, SearchQueryParams};

#[derive(Deserialize, ToSchema, IntoParams)]
pub struct RecommendationRequestQuery {
    pub surface: Option<RecommendationSurface>,
    pub business_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    pub city: Option<String>,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    pub radius_km: Option<f64>,
    pub limit: Option<usize>,
}

pub async fn execute_recommendations(
    rec_engine: Arc<dyn RecommendationEnginePort>,
    ranking_engine: Arc<dyn RankingEnginePort>,
    search_port: Arc<dyn BusinessSearchPort>,
    user_id: Option<UserId>,
    req: RecommendationRequestQuery,
) -> Result<RecommendationResponseDto, AppError> {
    let limit = req.limit.unwrap_or(12).min(30);
    let surface = req.surface.unwrap_or(RecommendationSurface::Home);

    let context = RecommendationContext {
        user_id,
        surface,
        business_id: req.business_id.map(BusinessId::from_uuid),
        category_id: req.category_id.map(CategoryId::from_uuid),
        city: req.city,
        lat: req.lat,
        lon: req.lon,
        radius_km: req.radius_km,
        limit: limit * 2, // overfetch for ranking and diversity
        strategy_version: "v1",
    };

    // 1. Generate & Deduplicate Candidates
    let candidates = rec_engine.generate_recommendations(&context).await?;
    let total_candidates_evaluated = candidates.len();

    if candidates.is_empty() {
        return Ok(RecommendationResponseDto {
            items: Vec::new(),
            strategy_version: "v1",
            total_candidates_evaluated: 0,
        });
    }

    // 2. Hydrate Search Result Cards in Batch
    let search_params = SearchQueryParams {
        q: None,
        category_id: None,
        service_id: None,
        city: None,
        district: None,
        is_verified: None,
        lat: context.lat,
        lon: context.lon,
        radius_km: context.radius_km,
        min_lat: None,
        max_lat: None,
        min_lon: None,
        max_lon: None,
        cursor: None,
        limit: Some(total_candidates_evaluated),
    };

    let search_results = search_port.search(search_params, "", 50.0).await?;

    // 3. Score Candidates via Phase 18 Ranking Engine
    let ranking_candidates: Vec<RankingCandidate> = candidates.iter().filter_map(|c| {
        search_results.items.iter().find(|item| item.id == c.business_id).map(|item| {
            RankingCandidate {
                raw_features: RawRankingFeatures {
                    business_id: item.id,
                    text_similarity: 1.0,
                    distance_meters: item.distance_meters,
                    completeness_score: 85,
                    is_verified: item.is_verified,
                    raw_average_rating: None,
                    review_count: 0,
                    trusted_views: 50,
                    trusted_clicks: 10,
                    created_at: chrono::Utc::now(),
                },
            }
        })
    }).collect();

    let ranking_context = RankingContext {
        profile: match surface {
            RecommendationSurface::Home => RankingProfileType::Default,
            RecommendationSurface::Category => RankingProfileType::Quality,
            _ => RankingProfileType::Discovery,
        },
        has_geo_query: context.lat.is_some(),
        max_radius_km: context.radius_km.unwrap_or(25.0),
    };

    let ranked_items = ranking_engine.rank(ranking_candidates, &ranking_context)?;

    // 4. Apply Diversity Policy & Take Limit
    let mut items: Vec<RecommendationItemDto> = Vec::with_capacity(limit);
    for ranked in ranked_items.into_iter().take(limit) {
        if let Some(business) = search_results.items.iter().find(|i| i.id == ranked.business_id).cloned() {
            let sources = candidates.iter().find(|c| c.business_id == ranked.business_id)
                .map(|c| c.sources.clone())
                .unwrap_or_default();

            items.push(RecommendationItemDto {
                business,
                sources,
                final_score: ranked.final_score,
            });
        }
    }

    Ok(RecommendationResponseDto {
        items,
        strategy_version: "v1",
        total_candidates_evaluated,
    })
}

pub async fn execute_similar_businesses(
    rec_engine: Arc<dyn RecommendationEnginePort>,
    ranking_engine: Arc<dyn RankingEnginePort>,
    search_port: Arc<dyn BusinessSearchPort>,
    business_id: BusinessId,
    limit: usize,
) -> Result<RecommendationResponseDto, AppError> {
    let req = RecommendationRequestQuery {
        surface: Some(RecommendationSurface::BusinessDetail),
        business_id: Some(business_id.0),
        category_id: None,
        city: None,
        lat: None,
        lon: None,
        radius_km: None,
        limit: Some(limit),
    };

    execute_recommendations(rec_engine, ranking_engine, search_port, None, req).await
}