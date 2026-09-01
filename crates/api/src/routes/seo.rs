use axum::{
    extract::{Path, State},
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use application::ports::seo::{BusinessSeoPageDto, CategoryLocationLandingDto};
use application::use_cases::seo::{
    generate_businesses_sitemap_xml, generate_sitemap_index_xml, get_business_seo_page,
    get_category_location_landing_page,
};
use crate::errors::ApiError;
use crate::state::AppState;

#[utoipa::path(
    get,
    path = "/api/v1/seo/business/{slug}",
    responses(
        (status = 200, description = "Business SEO projection with JSON-LD and meta tags", body = BusinessSeoPageDto),
        (status = 301, description = "Permanent redirect to new canonical slug"),
        (status = 404, description = "Business not found or unpublished", body = crate::errors::ApiErrorResponse)
    )
)]
pub async fn get_business_seo_endpoint(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let res = get_business_seo_page(
        state.seo_repo.clone(),
        state.business_repo.clone(),
        state.taxonomy_repo.clone(),
        &state.config.public_origin,
        &slug,
    ).await?;

    if res.is_redirect {
        if let Some(ref target) = res.redirect_target {
            let mut headers = HeaderMap::new();
            if let Ok(header_val) = target.parse() {
                headers.insert(header::LOCATION, header_val);
            }
            return Ok((StatusCode::MOVED_PERMANENTLY, headers, Json(res)).into_response());
        }
    }

    Ok((StatusCode::OK, HeaderMap::new(), Json(res)).into_response())
}

#[utoipa::path(
    get,
    path = "/api/v1/seo/landing/{city}/{category_slug}",
    responses(
        (status = 200, description = "SEO Landing Page for City + Category", body = CategoryLocationLandingDto),
        (status = 404, description = "Category not found", body = crate::errors::ApiErrorResponse)
    )
)]
pub async fn get_landing_page_endpoint(
    State(state): State<AppState>,
    Path((city, category_slug)): Path<(String, String)>,
) -> Result<Json<CategoryLocationLandingDto>, ApiError> {
    let landing = get_category_location_landing_page(
        state.seo_repo.clone(),
        state.taxonomy_repo.clone(),
        state.business_repo.clone(),
        &state.config.public_origin,
        state.config.min_seo_landing_inventory,
        &city,
        &category_slug,
    ).await?;

    Ok(Json(landing))
}

pub async fn get_robots_txt(State(state): State<AppState>) -> impl IntoResponse {
    let body = format!(
        "User-agent: *\nAllow: /\nDisallow: /api/\nDisallow: /admin/\nSitemap: {}/sitemap.xml\n",
        state.config.public_origin
    );
    ([(header::CONTENT_TYPE, "text/plain; charset=utf-8")], body)
}

pub async fn get_sitemap_index(State(state): State<AppState>) -> impl IntoResponse {
    let xml = generate_sitemap_index_xml(&state.config.public_origin).await;
    ([(header::CONTENT_TYPE, "application/xml; charset=utf-8")], xml)
}

pub async fn get_businesses_sitemap(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    let xml = generate_businesses_sitemap_xml(state.seo_repo.clone(), &state.config.public_origin).await?;
    Ok(([(header::CONTENT_TYPE, "application/xml; charset=utf-8")], xml))
}