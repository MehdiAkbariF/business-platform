use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use utoipa::ToSchema;
use uuid::Uuid;

use application::use_cases::taxonomy::{
    assign_business_category, assign_business_service, get_category_by_slug, get_service_by_slug,
    list_categories, list_services, remove_business_category, remove_business_service,
    set_business_primary_category, AssignCategoryCommand, AssignServiceCommand, BusinessCategoryDto,
    BusinessServiceDto, CategoryDto, ServiceDto,
};
use shared::{BusinessId, CategoryId, ClientMetadata, ServiceId};
use crate::errors::ApiError;
use crate::middleware::auth::AuthPrincipal;
use crate::state::AppState;

fn extract_client_metadata(headers: &HeaderMap) -> ClientMetadata {
    let ip_address = headers
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or("").trim().to_string());

    let user_agent = headers
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    ClientMetadata {
        ip_address,
        user_agent,
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/categories",
    responses(
        (status = 200, description = "List of active categories", body = Vec<CategoryDto>)
    )
)]
pub async fn get_categories(State(state): State<AppState>) -> Result<Json<Vec<CategoryDto>>, ApiError> {
    let list = list_categories(state.taxonomy_repo.clone()).await?;
    Ok(Json(list))
}

#[utoipa::path(
    get,
    path = "/api/v1/categories/{slug}",
    responses(
        (status = 200, description = "Category detail", body = CategoryDto),
        (status = 404, description = "Category not found", body = crate::errors::ApiErrorResponse)
    )
)]
pub async fn get_category(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<CategoryDto>, ApiError> {
    let cat = get_category_by_slug(state.taxonomy_repo.clone(), &slug).await?;
    Ok(Json(cat))
}

#[utoipa::path(
    get,
    path = "/api/v1/services",
    responses(
        (status = 200, description = "List of active services", body = Vec<ServiceDto>)
    )
)]
pub async fn get_services(State(state): State<AppState>) -> Result<Json<Vec<ServiceDto>>, ApiError> {
    let list = list_services(state.taxonomy_repo.clone()).await?;
    Ok(Json(list))
}

#[utoipa::path(
    get,
    path = "/api/v1/services/{slug}",
    responses(
        (status = 200, description = "Service detail", body = ServiceDto),
        (status = 404, description = "Service not found", body = crate::errors::ApiErrorResponse)
    )
)]
pub async fn get_service(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<ServiceDto>, ApiError> {
    let s = get_service_by_slug(state.taxonomy_repo.clone(), &slug).await?;
    Ok(Json(s))
}

#[utoipa::path(
    get,
    path = "/api/v1/businesses/{id}/categories",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Business categories", body = Vec<BusinessCategoryDto>)
    )
)]
pub async fn get_business_categories(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<BusinessCategoryDto>>, ApiError> {
    let business_id = BusinessId::from_uuid(id);
    let rows = state.taxonomy_repo.get_business_categories(business_id).await?;
    Ok(Json(rows.into_iter().map(|r| BusinessCategoryDto {
        category_id: r.category_id,
        name: r.name,
        slug: r.slug,
        is_primary: r.is_primary,
    }).collect()))
}

#[utoipa::path(
    post,
    path = "/api/v1/businesses/{id}/categories",
    request_body = AssignCategoryCommand,
    security(("bearer_auth" = [])),
    responses(
        (status = 201, description = "Category assigned"),
        (status = 400, description = "Validation error", body = crate::errors::ApiErrorResponse),
        (status = 403, description = "Forbidden", body = crate::errors::ApiErrorResponse)
    )
)]
pub async fn add_business_category(
    State(state): State<AppState>,
    AuthPrincipal(principal): AuthPrincipal,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    Json(payload): Json<AssignCategoryCommand>,
) -> Result<StatusCode, ApiError> {
    let metadata = extract_client_metadata(&headers);
    let business_id = BusinessId::from_uuid(id);
    assign_business_category(
        state.taxonomy_repo.clone(),
        state.membership_repo.clone(),
        state.audit_repo.clone(),
        business_id,
        principal.user_id,
        state.config.max_business_categories,
        payload,
        metadata,
    ).await?;

    Ok(StatusCode::CREATED)
}

#[utoipa::path(
    delete,
    path = "/api/v1/businesses/{id}/categories/{category_id}",
    security(("bearer_auth" = [])),
    responses(
        (status = 204, description = "Category removed"),
        (status = 403, description = "Forbidden", body = crate::errors::ApiErrorResponse)
    )
)]
pub async fn remove_business_category_endpoint(
    State(state): State<AppState>,
    AuthPrincipal(principal): AuthPrincipal,
    Path((id, category_id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> Result<StatusCode, ApiError> {
    let metadata = extract_client_metadata(&headers);
    let business_id = BusinessId::from_uuid(id);
    let cat_id = CategoryId::from_uuid(category_id);
    remove_business_category(
        state.taxonomy_repo.clone(),
        state.membership_repo.clone(),
        state.audit_repo.clone(),
        business_id,
        principal.user_id,
        cat_id,
        metadata,
    ).await?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    put,
    path = "/api/v1/businesses/{id}/categories/{category_id}/primary",
    security(("bearer_auth" = [])),
    responses(
        (status = 204, description = "Primary category updated"),
        (status = 403, description = "Forbidden", body = crate::errors::ApiErrorResponse)
    )
)]
pub async fn set_primary_category_endpoint(
    State(state): State<AppState>,
    AuthPrincipal(principal): AuthPrincipal,
    Path((id, category_id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> Result<StatusCode, ApiError> {
    let metadata = extract_client_metadata(&headers);
    let business_id = BusinessId::from_uuid(id);
    let cat_id = CategoryId::from_uuid(category_id);
    set_business_primary_category(
        state.taxonomy_repo.clone(),
        state.membership_repo.clone(),
        state.audit_repo.clone(),
        business_id,
        principal.user_id,
        cat_id,
        metadata,
    ).await?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/api/v1/businesses/{id}/services",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Business services", body = Vec<BusinessServiceDto>)
    )
)]
pub async fn get_business_services(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<BusinessServiceDto>>, ApiError> {
    let business_id = BusinessId::from_uuid(id);
    let rows = state.taxonomy_repo.get_business_services(business_id).await?;
    Ok(Json(rows.into_iter().map(|r| BusinessServiceDto {
        service_id: r.service_id,
        name: r.name,
        slug: r.slug,
        is_active: r.is_active,
        sort_order: r.sort_order,
    }).collect()))
}

#[utoipa::path(
    post,
    path = "/api/v1/businesses/{id}/services",
    request_body = AssignServiceCommand,
    security(("bearer_auth" = [])),
    responses(
        (status = 201, description = "Service assigned"),
        (status = 400, description = "Incompatible service or validation error", body = crate::errors::ApiErrorResponse),
        (status = 403, description = "Forbidden", body = crate::errors::ApiErrorResponse)
    )
)]
pub async fn add_business_service_endpoint(
    State(state): State<AppState>,
    AuthPrincipal(principal): AuthPrincipal,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    Json(payload): Json<AssignServiceCommand>,
) -> Result<StatusCode, ApiError> {
    let metadata = extract_client_metadata(&headers);
    let business_id = BusinessId::from_uuid(id);
    assign_business_service(
        state.taxonomy_repo.clone(),
        state.membership_repo.clone(),
        state.audit_repo.clone(),
        business_id,
        principal.user_id,
        state.config.max_business_services,
        payload,
        metadata,
    ).await?;

    Ok(StatusCode::CREATED)
}

#[utoipa::path(
    delete,
    path = "/api/v1/businesses/{id}/services/{service_id}",
    security(("bearer_auth" = [])),
    responses(
        (status = 204, description = "Service removed"),
        (status = 403, description = "Forbidden", body = crate::errors::ApiErrorResponse)
    )
)]
pub async fn remove_business_service_endpoint(
    State(state): State<AppState>,
    AuthPrincipal(principal): AuthPrincipal,
    Path((id, service_id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> Result<StatusCode, ApiError> {
    let metadata = extract_client_metadata(&headers);
    let business_id = BusinessId::from_uuid(id);
    let s_id = ServiceId::from_uuid(service_id);
    remove_business_service(
        state.taxonomy_repo.clone(),
        state.membership_repo.clone(),
        state.audit_repo.clone(),
        business_id,
        principal.user_id,
        s_id,
        metadata,
    ).await?;

    Ok(StatusCode::NO_CONTENT)
}