use std::sync::Arc;
use domain::monetization::{validate_double_entry_balance, Currency, LedgerEntry, LedgerEntryType, Money};
use shared::{BusinessId, CampaignId, CreativeId, PaymentId, PlanId, UserId};
use utoipa::ToSchema;
use serde::{Deserialize, Serialize};
use crate::errors::AppError;
use crate::ports::monetization::{MonetizationRepository, PaymentDto, PaymentProviderPort, PlanDto, SponsoredAdDto, SubscriptionDto};
use crate::ports::repositories::{AuditRepository, MembershipRepository};

#[derive(Deserialize, ToSchema)]
pub struct SubscribeCommand {
    pub plan_id: PlanId,
    pub idempotency_key: String,
}

#[derive(Deserialize, ToSchema)]
pub struct CreateCampaignCommand {
    pub name: String,
    pub budget_amount: i64,
    pub currency: Currency,
    pub title: String,
    pub description: String,
    pub destination_url: String,
    pub image_url: Option<String>,
}

pub async fn list_plans(repo: Arc<dyn MonetizationRepository>) -> Result<Vec<PlanDto>, AppError> {
    repo.list_active_plans().await
}

pub async fn start_subscription_checkout(
    monetization_repo: Arc<dyn MonetizationRepository>,
    payment_provider: Arc<dyn PaymentProviderPort>,
    membership_repo: Arc<dyn MembershipRepository>,
    audit_repo: Arc<dyn AuditRepository>,
    business_id: BusinessId,
    user_id: UserId,
    cmd: SubscribeCommand,
) -> Result<PaymentDto, AppError> {
    let membership = membership_repo
        .find_membership(business_id, user_id)
        .await?
        .ok_or_else(|| AppError::Forbidden("You are not an authorized member of this business".to_string()))?;

    if !membership.can_archive() { // Only Owner can purchase subscriptions
        return Err(AppError::Forbidden("Only the business OWNER can manage billing and subscriptions".to_string()));
    }

    let plan = monetization_repo
        .find_plan_by_id(cmd.plan_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Subscription plan not found".to_string()))?;

    let payment_id = PaymentId::new();
    monetization_repo.create_payment(
        payment_id,
        business_id,
        user_id,
        &cmd.idempotency_key,
        plan.price,
        "ZARINPAL",
    ).await?;

    let redirect_url = payment_provider.initiate_payment(
        payment_id,
        plan.price,
        "https://platform.com/api/v1/billing/callback",
    ).await?;

    let _ = audit_repo.record(
        Some(user_id),
        "SUBSCRIPTION_PAYMENT_INITIATED",
        None,
        None,
        Some(serde_json::json!({ "business_id": business_id.to_string(), "payment_id": payment_id.to_string(), "amount": plan.price.amount })),
    ).await;

    Ok(PaymentDto {
        id: payment_id,
        business_id,
        amount: plan.price,
        status: domain::monetization::PaymentStatus::Initiated,
        provider: "ZARINPAL".to_string(),
        payment_url: Some(redirect_url),
    })
}

pub async fn verify_and_activate_payment(
    monetization_repo: Arc<dyn MonetizationRepository>,
    payment_provider: Arc<dyn PaymentProviderPort>,
    audit_repo: Arc<dyn AuditRepository>,
    payment_id: PaymentId,
    provider_ref: &str,
) -> Result<(), AppError> {
    let payment = monetization_repo
        .find_payment_by_id(payment_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Payment record not found".to_string()))?;

    let is_verified = payment_provider.verify_payment(provider_ref, payment.amount).await?;
    if !is_verified {
        return Err(AppError::Validation("Payment verification failed at provider".to_string()));
    }

    // Double-Entry Balanced Ledger Entries
    let ledger = vec![
        LedgerEntry {
            account: "ASSETS:BANK_ACCOUNT".to_string(),
            entry_type: LedgerEntryType::Debit,
            amount: payment.amount.amount,
        },
        LedgerEntry {
            account: "REVENUE:BUSINESS_SUBSCRIPTIONS".to_string(),
            entry_type: LedgerEntryType::Credit,
            amount: payment.amount.amount,
        },
    ];

    validate_double_entry_balance(&ledger).map_err(|e| AppError::Validation(e.to_string()))?;
    monetization_repo.record_successful_payment(payment_id, provider_ref, &ledger).await?;

    let _ = audit_repo.record(
        None,
        "PAYMENT_VERIFIED_AND_ACTIVATED",
        None,
        None,
        Some(serde_json::json!({ "payment_id": payment_id.to_string(), "provider_ref": provider_ref })),
    ).await;

    Ok(())
}

pub async fn create_sponsored_campaign(
    monetization_repo: Arc<dyn MonetizationRepository>,
    membership_repo: Arc<dyn MembershipRepository>,
    audit_repo: Arc<dyn AuditRepository>,
    business_id: BusinessId,
    user_id: UserId,
    cmd: CreateCampaignCommand,
) -> Result<CampaignId, AppError> {
    let membership = membership_repo
        .find_membership(business_id, user_id)
        .await?
        .ok_or_else(|| AppError::Forbidden("You are not an authorized member".to_string()))?;

    if !membership.can_archive() {
        return Err(AppError::Forbidden("Only the business OWNER can create advertising campaigns".to_string()));
    }

    let campaign_id = CampaignId::new();
    let creative_id = CreativeId::new();
    let budget = Money::new(cmd.budget_amount, cmd.currency);

    let start_date = chrono::Utc::now();
    let end_date = start_date + chrono::Duration::days(30);

    monetization_repo.create_campaign(campaign_id, business_id, &cmd.name, budget, start_date, end_date).await?;
    monetization_repo.add_creative(creative_id, campaign_id, &cmd.title, &cmd.description, cmd.image_url, &cmd.destination_url).await?;

    let _ = audit_repo.record(
        Some(user_id),
        "AD_CAMPAIGN_CREATED",
        None,
        None,
        Some(serde_json::json!({ "business_id": business_id.to_string(), "campaign_id": campaign_id.to_string(), "budget": budget.amount })),
    ).await;

    Ok(campaign_id)
}

pub async fn get_sponsored_placements(
    monetization_repo: Arc<dyn MonetizationRepository>,
    city: Option<&str>,
    limit: usize,
) -> Result<Vec<SponsoredAdDto>, AppError> {
    monetization_repo.get_active_sponsored_ads(city, limit).await
}