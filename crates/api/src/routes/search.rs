use axum::{
    extract::{Query, State},
    Json,
};
use serde::Deserialize;
use utoipa::IntoParams;

use application::ports::search::{SearchResponseDto, SuggestionItemDto};
use application::use_cases::search::{execute_autocomplete, execute_search, SearchRequestQuery};
use crate::errors::ApiError;
use crate::state::AppState;

#[derive(Deserialize, IntoParams)]
pub struct AutocompleteQuery {
    pub q: String,
    pub limit: Option<usize>,
}

#[utoipa::path(
    get,
    path = "/api/v1/search",
    params(
        SearchRequestQuery
    ),
    responses(
        (status = 200, description = "Search results matching query and filters", body = SearchResponseDto),
        (status = 400, description = "Invalid search parameters", body = crate::errors::ApiErrorResponse)
    )
)]
pub async fn search_businesses(
    State(state): State<AppState>,
    Query(query): Query<SearchRequestQuery>,
) -> Result<Json<SearchResponseDto>, ApiError> {
    let resp = execute_search(state.search_port.clone(), query, state.config.max_search_radius_km).await?;
    Ok(Json(resp))
}

#[utoipa::path(
    get,
    path = "/api/v1/search/suggestions",
    params(
        AutocompleteQuery
    ),
    responses(
        (status = 200, description = "Autocomplete suggestions", body = Vec<SuggestionItemDto>)
    )
)]
pub async fn autocomplete_suggestions(
    State(state): State<AppState>,
    Query(query): Query<AutocompleteQuery>,
) -> Result<Json<Vec<SuggestionItemDto>>, ApiError> {
    let limit = query.limit.unwrap_or(8);
    let list = execute_autocomplete(state.search_port.clone(), &query.q, limit).await?;
    Ok(Json(list))
}