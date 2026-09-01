use std::sync::Arc;
use domain::search::{parse_search_query, validate_coordinates};
use utoipa::{IntoParams, ToSchema};
use serde::Deserialize;
use uuid::Uuid;
use crate::errors::AppError;
use crate::ports::search::{BusinessSearchPort, SearchQueryParams, SearchResponseDto, SuggestionItemDto};

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

    search_port.search(params, &normalized_text, max_radius_km).await
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