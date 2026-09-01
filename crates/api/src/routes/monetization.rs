use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use utoipa::IntoParams;
use uuid::Uuid;

use application::ports::monetization::{PaymentDto, PlanDto, SponsoredAdDto};
use application::use_cases::monetization::{
    create_sponsored_campaign, get_sponsored_placements, list_plans, start_subscription_checkout,
    verify_and_activate_payment, CreateCampaignCommand, SubscribeCommand,
};
use shared::{BusinessId, PaymentId};
use crate::errors::ApiError;
use crate::middleware::auth::AuthPrincipal;
use crate::state::AppState;

#[derive(Deserialize, IntoParams)]
pub struct VerifyCallbackQuery {
    pub provider_ref: String,
}

#[utoipa::path(
    get,
    path = "/api/v1/plans",
    responses(
        (status = 200, description = "List of active subscription plans", body = Vec<PlanDto>)
    )
)]
pub async fn get_plans(State(state): State<AppState>) -> Result<Json<Vec<PlanDto>>, ApiError> {
    let plans = list_plans(state.monetization_repo.clone()).await?;
    Ok(Json(plans))
}

#[utoipa::path(
    post,
    path = "/api/v1/businesses/{id}/subscribe",
    request_body = SubscribeCommand,
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Checkout initiated", body = PaymentDto),
        (status = 403, description = "Forbidden (Owner only)", body = crate::errors::ApiErrorResponse)
    )
)]
pub async fn subscribe_business_endpoint(
    State(state): State<AppState>,
    AuthPrincipal(principal): AuthPrincipal,
    Path(id): Path<Uuid>,
    Json(payload): Json<SubscribeCommand>,
) -> Result<Json<PaymentDto>, ApiError> {
    let business_id = BusinessId::from_uuid(id);
    let payment = start_subscription_checkout(
        state.monetization_repo.clone(),
        state.payment_provider.clone(),
        state.membership_repo.clone(),
        state.audit_repo.clone(),
        business_id,
        principal.user_id,
        payload,
    ).await?;

    Ok(Json(payment))
}

#[utoipa::path(
    post,
    path = "/api/v1/billing/payments/{id}/verify",
    params(
        VerifyCallbackQuery
    ),
    responses(
        (status = 204, description = "Payment verified and activated"),
        (status = 400, description = "Verification failed", body = crate::errors::ApiErrorResponse)
    )
)]
pub async fn verify_payment_endpoint(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(query): Query<VerifyCallbackQuery>,
) -> Result<StatusCode, ApiError> {
    let payment_id = PaymentId::from_uuid(id);
    verify_and_activate_payment(
        state.monetization_repo.clone(),
        state.payment_provider.clone(),
        state.audit_repo.clone(),
        payment_id,
        &query.provider_ref,
    ).await?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/api/v1/businesses/{id}/campaigns",
    request_body = CreateCampaignCommand,
    security(("bearer_auth" = [])),
    responses(
        (status = 201, description = "Ad campaign created"),
        (status = 403, description = "Forbidden", body = crate::errors::ApiErrorResponse)
    )
)]
pub async fn create_campaign_endpoint(
    State(state): State<AppState>,
    AuthPrincipal(principal): AuthPrincipal,
    Path(id): Path<Uuid>,
    Json(payload): Json<CreateCampaignCommand>,
) -> Result<StatusCode, ApiError> {
    let business_id = BusinessId::from_uuid(id);
    create_sponsored_campaign(
        state.monetization_repo.clone(),
        state.membership_repo.clone(),
        state.audit_repo.clone(),
        business_id,
        principal.user_id,
        payload,
    ).await?;

    Ok(StatusCode::CREATED)
}

#[utoipa::path(
    get,
    path = "/api/v1/ads/sponsored",
    responses(
        (status = 200, description = "Active sponsored ad placements", body = Vec<SponsoredAdDto>)
    )
)]
pub async fn get_sponsored_ads(State(state): State<AppState>) -> Result<Json<Vec<SponsoredAdDto>>, ApiError> {
    let ads = get_sponsored_placements(state.monetization_repo.clone(), None, 4).await?;
    Ok(Json(ads))
}