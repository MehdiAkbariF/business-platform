use std::sync::Arc;
use domain::ranking::{RankingProfileType, RawRankingFeatures};
use domain::search::{parse_search_query, validate_coordinates};
use utoipa::{IntoParams, ToSchema};
use serde::Deserialize;
use uuid::Uuid;
use crate::errors::AppError;
use crate::ports::ranking::{RankingCandidate, RankingContext, RankingEnginePort};
use crate::ports::search::{
    BusinessSearchPort, SearchQueryParams, SearchResponseDto, SearchResultItemDto, SuggestionItemDto,
};

#[derive(Deserialize, ToSchema, IntoParams)]
pub struct SearchRequestQuery {
    pub q: Option<String>,
    pub category_id: Option<Uuid>,
    pub service_id: Option<Uuid>,
    pub city: Option<String>,
    pub district: Option<String>,
    pub is_verified: Option<bool>,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    pub radius_km: Option<f64>,
    pub cursor: Option<String>,
    pub limit: Option<usize>,
}

pub async fn execute_search(
    search_port: Arc<dyn BusinessSearchPort>,
    ranking_engine: Arc<dyn RankingEnginePort>,
    req: SearchRequestQuery,
    max_radius_km: f64,
) -> Result<SearchResponseDto, AppError> {
    if let (Some(lat), Some(lon)) = (req.lat, req.lon) {
        validate_coordinates(lat, lon).map_err(|e| AppError::Validation(e.to_string()))?;
    }

    let normalized_text = match &req.q {
        Some(raw) if !raw.trim().is_empty() => {
            let parsed = parse_search_query(raw).map_err(|e| AppError::Validation(e.to_string()))?;
            parsed.normalized_query
        }
        _ => String::new(),
    };

    let params = SearchQueryParams {
        q: req.q,
        category_id: req.category_id.map(shared::CategoryId::from_uuid),
        service_id: req.service_id.map(shared::ServiceId::from_uuid),
        city: req.city,
        district: req.district,
        is_verified: req.is_verified,
        lat: req.lat,
        lon: req.lon,
        radius_km: req.radius_km,
        min_lat: None,
        max_lat: None,
        min_lon: None,
        max_lon: None,
        cursor: req.cursor,
        limit: req.limit,
    };

    // 1. Candidate Retrieval from Search Store
    let initial_results = search_port.search(params.clone(), &normalized_text, max_radius_km).await?;

    if initial_results.items.is_empty() {
        return Ok(initial_results);
    }

    // 2. Feature Extraction for Candidates
    let candidates: Vec<RankingCandidate> = initial_results.items.iter().map(|item| {
        RankingCandidate {
            raw_features: RawRankingFeatures {
                business_id: item.id,
                text_similarity: item.score,
                distance_meters: item.distance_meters,
                completeness_score: 80,
                is_verified: item.is_verified,
                raw_average_rating: None,
                review_count: 0,
                trusted_views: 10,
                trusted_clicks: 2,
                created_at: chrono::Utc::now(),
            },
        }
    }).collect();

    // 3. Execution of Ranking & Score Calculation
    let ranking_context = RankingContext {
        profile: if params.lat.is_some() { RankingProfileType::Local } else { RankingProfileType::Default },
        has_geo_query: params.lat.is_some(),
        max_radius_km,
    };

    let ranked_items = ranking_engine.rank(candidates, &ranking_context)?;

    // 4. Reorder and compose final result DTO
    let mut final_items: Vec<SearchResultItemDto> = Vec::with_capacity(ranked_items.len());
    for ranked in ranked_items {
        if let Some(mut original) = initial_results.items.iter().find(|i| i.id == ranked.business_id).cloned() {
            original.score = ranked.final_score;
            final_items.push(original);
        }
    }

    let total_count = final_items.len();
    let next_cursor = if initial_results.has_more { final_items.last().map(|i| i.id.to_string()) } else { None };

    Ok(SearchResponseDto {
        items: final_items,
        next_cursor,
        total_count,
        has_more: initial_results.has_more,
    })
}

pub async fn execute_autocomplete(
    search_port: Arc<dyn BusinessSearchPort>,
    query: &str,
    limit: usize,
) -> Result<Vec<SuggestionItemDto>, AppError> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }

    let parsed = parse_search_query(trimmed).map_err(|e| AppError::Validation(e.to_string()))?;
    search_port.autocomplete(&parsed.normalized_query, limit).await
}