use async_trait::async_trait;
use chrono::{DateTime, Utc};
use domain::monetization::{Currency, LedgerEntry, Money, PaymentStatus, SubscriptionStatus};
use shared::{BusinessId, CampaignId, CreativeId, PaymentId, PlanId, SubscriptionId, TransactionId, UserId};
use sqlx::PgPool;
use application::errors::AppError;
use application::ports::monetization::{MonetizationRepository, PaymentDto, PlanDto, SponsoredAdDto, SubscriptionDto};
use uuid::Uuid;

pub struct PostgresMonetizationRepository {
    pool: PgPool,
}

impl PostgresMonetizationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct PlanRow {
    id: Uuid,
    identifier: String,
    version: i32,
    name: String,
    price_amount: i64,
    currency: String,
    billing_period: String,
}

#[derive(sqlx::FromRow)]
struct PaymentRow {
    id: Uuid,
    business_id: Uuid,
    amount: i64,
    currency: String,
    status: String,
    provider: String,
}

#[derive(sqlx::FromRow)]
struct SponsoredAdRow {
    campaign_id: Uuid,
    creative_id: Uuid,
    title: String,
    description: String,
    image_url: Option<String>,
    destination_url: String,
}

#[async_trait]
impl MonetizationRepository for PostgresMonetizationRepository {
    async fn list_active_plans(&self) -> Result<Vec<PlanDto>, AppError> {
        let rows = sqlx::query_as::<_, PlanRow>(
            "SELECT id, identifier, version, name, price_amount, currency, billing_period FROM subscription_plans WHERE is_active = TRUE"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        Ok(rows.into_iter().map(|r| PlanDto {
            id: PlanId::from_uuid(r.id),
            identifier: r.identifier,
            version: r.version,
            name: r.name,
            price: Money::new(r.price_amount, Currency::Irr),
            billing_period: r.billing_period,
            entitlements: vec!["FEATURED_LISTING".to_string(), "ADVANCED_ANALYTICS".to_string()],
        }).collect())
    }

    async fn find_plan_by_id(&self, plan_id: PlanId) -> Result<Option<PlanDto>, AppError> {
        let row = sqlx::query_as::<_, PlanRow>(
            "SELECT id, identifier, version, name, price_amount, currency, billing_period FROM subscription_plans WHERE id = $1"
        )
        .bind(plan_id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        Ok(row.map(|r| PlanDto {
            id: PlanId::from_uuid(r.id),
            identifier: r.identifier,
            version: r.version,
            name: r.name,
            price: Money::new(r.price_amount, Currency::Irr),
            billing_period: r.billing_period,
            entitlements: vec!["FEATURED_LISTING".to_string()],
        }))
    }

    async fn create_subscription(&self, business_id: BusinessId, plan_id: PlanId, duration_days: i64) -> Result<SubscriptionId, AppError> {
        let sub_id = SubscriptionId::new();
        let start = Utc::now();
        let end = start + chrono::Duration::days(duration_days);

        sqlx::query(
            "INSERT INTO business_subscriptions (id, business_id, plan_id, status, current_period_start, current_period_end, started_at, updated_at) 
             VALUES ($1, $2, $3, 'ACTIVE'::subscription_status, $4, $5, $4, $4)"
        )
        .bind(sub_id.0)
        .bind(business_id.0)
        .bind(plan_id.0)
        .bind(start)
        .bind(end)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        Ok(sub_id)
    }

    async fn get_business_subscription(&self, business_id: BusinessId) -> Result<Option<SubscriptionDto>, AppError> {
        #[derive(sqlx::FromRow)]
        struct Row { id: Uuid, plan_id: Uuid, status: String, current_period_end: DateTime<Utc> }

        let row = sqlx::query_as::<_, Row>(
            "SELECT id, plan_id, status::text, current_period_end FROM business_subscriptions WHERE business_id = $1 AND status = 'ACTIVE' LIMIT 1"
        )
        .bind(business_id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        Ok(row.map(|r| SubscriptionDto {
            id: SubscriptionId::from_uuid(r.id),
            business_id,
            plan_id: PlanId::from_uuid(r.plan_id),
            status: SubscriptionStatus::Active,
            current_period_end: r.current_period_end,
        }))
    }

    async fn create_payment(&self, payment_id: PaymentId, business_id: BusinessId, user_id: UserId, idempotency_key: &str, amount: Money, provider: &str) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO payments (id, business_id, user_id, idempotency_key, amount, currency, provider, status, created_at, updated_at) 
             VALUES ($1, $2, $3, $4, $5, $6, $7, 'INITIATED'::payment_status, NOW(), NOW())"
        )
        .bind(payment_id.0)
        .bind(business_id.0)
        .bind(user_id.0)
        .bind(idempotency_key)
        .bind(amount.amount)
        .bind(amount.currency.to_string())
        .bind(provider)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            if let Some(db_err) = e.as_database_error() {
                if db_err.is_unique_violation() {
                    return AppError::Conflict("Duplicate payment request (Idempotency Key already used)".to_string());
                }
            }
            AppError::internal(e)
        })?;

        Ok(())
    }

    async fn find_payment_by_id(&self, payment_id: PaymentId) -> Result<Option<PaymentDto>, AppError> {
        let row = sqlx::query_as::<_, PaymentRow>(
            "SELECT id, business_id, amount, currency, status::text, provider FROM payments WHERE id = $1"
        )
        .bind(payment_id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        Ok(row.map(|r| PaymentDto {
            id: PaymentId::from_uuid(r.id),
            business_id: BusinessId::from_uuid(r.business_id),
            amount: Money::new(r.amount, Currency::Irr),
            status: PaymentStatus::Initiated,
            provider: r.provider,
            payment_url: None,
        }))
    }

    async fn record_successful_payment(&self, payment_id: PaymentId, provider_ref: &str, ledger_entries: &[LedgerEntry]) -> Result<(), AppError> {
        let mut tx = self.pool.begin().await.map_err(|e| AppError::internal(e))?;

        // 1. Update Payment Status
        sqlx::query("UPDATE payments SET status = 'SUCCEEDED'::payment_status, provider_ref = $1, updated_at = NOW() WHERE id = $2")
            .bind(provider_ref)
            .bind(payment_id.0)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::internal(e))?;

        // 2. Create Financial Transaction
        let tx_id = TransactionId::new();
        sqlx::query(
            "INSERT INTO financial_transactions (id, payment_id, description, currency, created_at) VALUES ($1, $2, 'Subscription Payment', 'IRR', NOW())"
        )
        .bind(tx_id.0)
        .bind(payment_id.0)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::internal(e))?;

        // 3. Write Immutable Double-Entry Ledger
        for entry in ledger_entries {
            let entry_id = Uuid::now_v7();
            sqlx::query(
                "INSERT INTO ledger_entries (id, transaction_id, account, entry_type, amount, created_at) 
                 VALUES ($1, $2, $3, $4::ledger_entry_type, $5, NOW())"
            )
            .bind(entry_id)
            .bind(tx_id.0)
            .bind(&entry.account)
            .bind(entry.entry_type.to_string())
            .bind(entry.amount)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::internal(e))?;
        }

        tx.commit().await.map_err(|e| AppError::internal(e))?;
        Ok(())
    }

    async fn create_campaign(&self, campaign_id: CampaignId, business_id: BusinessId, name: &str, budget: Money, start_date: DateTime<Utc>, end_date: DateTime<Utc>) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO ad_campaigns (id, business_id, name, status, total_budget, spent_amount, reserved_amount, currency, start_date, end_date, created_at, updated_at) 
             VALUES ($1, $2, $3, 'ACTIVE'::campaign_status, $4, 0, 0, $5, $6, $7, NOW(), NOW())"
        )
        .bind(campaign_id.0)
        .bind(business_id.0)
        .bind(name)
        .bind(budget.amount)
        .bind(budget.currency.to_string())
        .bind(start_date)
        .bind(end_date)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        Ok(())
    }

    async fn add_creative(&self, creative_id: CreativeId, campaign_id: CampaignId, title: &str, description: &str, image_url: Option<String>, destination_url: &str) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO ad_creatives (id, campaign_id, title, description, image_url, destination_url, status, created_at) 
             VALUES ($1, $2, $3, $4, $5, $6, 'APPROVED'::creative_status, NOW())"
        )
        .bind(creative_id.0)
        .bind(campaign_id.0)
        .bind(title)
        .bind(description)
        .bind(image_url)
        .bind(destination_url)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        Ok(())
    }

    async fn get_active_sponsored_ads(&self, _city: Option<&str>, limit: usize) -> Result<Vec<SponsoredAdDto>, AppError> {
        let rows = sqlx::query_as::<_, SponsoredAdRow>(
            "SELECT c.id AS campaign_id, cr.id AS creative_id, cr.title, cr.description, cr.image_url, cr.destination_url 
             FROM ad_campaigns c
             INNER JOIN ad_creatives cr ON c.id = cr.campaign_id
             WHERE c.status = 'ACTIVE' AND cr.status = 'APPROVED' AND c.spent_amount < c.total_budget
             ORDER BY c.created_at DESC
             LIMIT $1"
        )
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        Ok(rows.into_iter().map(|r| SponsoredAdDto {
            campaign_id: CampaignId::from_uuid(r.campaign_id),
            creative_id: CreativeId::from_uuid(r.creative_id),
            title: r.title,
            description: r.description,
            image_url: r.image_url,
            destination_url: r.destination_url,
            is_sponsored: true,
        }).collect())
    }

    async fn atomically_reserve_and_spend_budget(&self, campaign_id: CampaignId, cost_per_click: i64) -> Result<bool, AppError> {
        // Atomic budget reservation preventing overspending under concurrent requests
        let res = sqlx::query(
            "UPDATE ad_campaigns 
             SET spent_amount = spent_amount + $1, updated_at = NOW() 
             WHERE id = $2 AND status = 'ACTIVE' AND (spent_amount + reserved_amount + $1) <= total_budget"
        )
        .bind(cost_per_click)
        .bind(campaign_id.0)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        Ok(res.rows_affected() > 0)
    }
}