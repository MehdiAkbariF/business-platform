use async_trait::async_trait;
use domain::monetization::{CampaignStatus, Currency, LedgerEntry, Money, PaymentStatus, SubscriptionStatus};
use shared::{BusinessId, CampaignId, CreativeId, PaymentId, PlanId, SubscriptionId, TransactionId, UserId};
use utoipa::ToSchema;
use serde::{Deserialize, Serialize};
use crate::errors::AppError;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct PlanDto {
    pub id: PlanId,
    pub identifier: String,
    pub version: i32,
    pub name: String,
    pub price: Money,
    pub billing_period: String,
    pub entitlements: Vec<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct SubscriptionDto {
    pub id: SubscriptionId,
    pub business_id: BusinessId,
    pub plan_id: PlanId,
    pub status: SubscriptionStatus,
    pub current_period_end: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct PaymentDto {
    pub id: PaymentId,
    pub business_id: BusinessId,
    pub amount: Money,
    pub status: PaymentStatus,
    pub provider: String,
    pub payment_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct SponsoredAdDto {
    pub campaign_id: CampaignId,
    pub creative_id: CreativeId,
    pub title: String,
    pub description: String,
    pub image_url: Option<String>,
    pub destination_url: String,
    pub is_sponsored: bool,
}

#[async_trait]
pub trait PaymentProviderPort: Send + Sync {
    async fn initiate_payment(&self, payment_id: PaymentId, amount: Money, callback_url: &str) -> Result<String, AppError>;
    async fn verify_payment(&self, provider_ref: &str, amount: Money) -> Result<bool, AppError>;
    async fn refund_payment(&self, provider_ref: &str, amount: Money) -> Result<bool, AppError>;
}

#[async_trait]
pub trait MonetizationRepository: Send + Sync {
    // Plans & Subscriptions
    async fn list_active_plans(&self) -> Result<Vec<PlanDto>, AppError>;
    async fn find_plan_by_id(&self, plan_id: PlanId) -> Result<Option<PlanDto>, AppError>;
    async fn create_subscription(&self, business_id: BusinessId, plan_id: PlanId, duration_days: i64) -> Result<SubscriptionId, AppError>;
    async fn get_business_subscription(&self, business_id: BusinessId) -> Result<Option<SubscriptionDto>, AppError>;

    // Payments & Ledger
    async fn create_payment(&self, payment_id: PaymentId, business_id: BusinessId, user_id: UserId, idempotency_key: &str, amount: Money, provider: &str) -> Result<(), AppError>;
    async fn find_payment_by_id(&self, payment_id: PaymentId) -> Result<Option<PaymentDto>, AppError>;
    async fn record_successful_payment(&self, payment_id: PaymentId, provider_ref: &str, ledger_entries: &[LedgerEntry]) -> Result<(), AppError>;

    // Advertising Campaigns & Creatives
    async fn create_campaign(&self, campaign_id: CampaignId, business_id: BusinessId, name: &str, budget: Money, start_date: chrono::DateTime<chrono::Utc>, end_date: chrono::DateTime<chrono::Utc>) -> Result<(), AppError>;
    async fn add_creative(&self, creative_id: CreativeId, campaign_id: CampaignId, title: &str, description: &str, image_url: Option<String>, destination_url: &str) -> Result<(), AppError>;
    async fn get_active_sponsored_ads(&self, city: Option<&str>, limit: usize) -> Result<Vec<SponsoredAdDto>, AppError>;
    async fn atomically_reserve_and_spend_budget(&self, campaign_id: CampaignId, cost_per_click: i64) -> Result<bool, AppError>;
}