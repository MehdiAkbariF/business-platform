use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared::{BusinessId, CampaignId, CreativeId, PaymentId, PlanId, SubscriptionId, TransactionId, UserId};
use utoipa::ToSchema;
use crate::validation::ValidationError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct Money {
    pub amount: i64, // Stored in minor units (e.g. Rials or Cents)
    pub currency: Currency,
}

impl Money {
    pub fn new(amount: i64, currency: Currency) -> Self {
        Self { amount, currency }
    }

    pub fn add(&self, other: &Self) -> Result<Self, ValidationError> {
        if self.currency != other.currency {
            return Err(ValidationError::InvalidFormat("currency".to_string(), "Currency mismatch in arithmetic".to_string()));
        }
        Ok(Self {
            amount: self.amount + other.amount,
            currency: self.currency,
        })
    }

    pub fn subtract(&self, other: &Self) -> Result<Self, ValidationError> {
        if self.currency != other.currency {
            return Err(ValidationError::InvalidFormat("currency".to_string(), "Currency mismatch in arithmetic".to_string()));
        }
        if self.amount < other.amount {
            return Err(ValidationError::InvalidFormat("amount".to_string(), "Insufficient funds for subtraction".to_string()));
        }
        Ok(Self {
            amount: self.amount - other.amount,
            currency: self.currency,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Currency {
    Irr,
    Toman,
    Usd,
    Eur,
}

impl std::fmt::Display for Currency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Irr => write!(f, "IRR"),
            Self::Toman => write!(f, "TOMAN"),
            Self::Usd => write!(f, "USD"),
            Self::Eur => write!(f, "EUR"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SubscriptionStatus {
    Trial,
    Active,
    PastDue,
    Paused,
    Canceled,
    Expired,
}

impl std::fmt::Display for SubscriptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Trial => write!(f, "TRIAL"),
            Self::Active => write!(f, "ACTIVE"),
            Self::PastDue => write!(f, "PAST_DUE"),
            Self::Paused => write!(f, "PAUSED"),
            Self::Canceled => write!(f, "CANCELED"),
            Self::Expired => write!(f, "EXPIRED"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PaymentStatus {
    Initiated,
    Pending,
    Processing,
    Succeeded,
    Failed,
    Refunded,
}

impl std::fmt::Display for PaymentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Initiated => write!(f, "INITIATED"),
            Self::Pending => write!(f, "PENDING"),
            Self::Processing => write!(f, "PROCESSING"),
            Self::Succeeded => write!(f, "SUCCEEDED"),
            Self::Failed => write!(f, "FAILED"),
            Self::Refunded => write!(f, "REFUNDED"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CampaignStatus {
    Draft,
    PendingReview,
    Active,
    Paused,
    Exhausted,
    Completed,
    Rejected,
}

impl std::fmt::Display for CampaignStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Draft => write!(f, "DRAFT"),
            Self::PendingReview => write!(f, "PENDING_REVIEW"),
            Self::Active => write!(f, "ACTIVE"),
            Self::Paused => write!(f, "PAUSED"),
            Self::Exhausted => write!(f, "EXHAUSTED"),
            Self::Completed => write!(f, "COMPLETED"),
            Self::Rejected => write!(f, "REJECTED"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CreativeStatus {
    Pending,
    Approved,
    Rejected,
}

impl std::fmt::Display for CreativeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "PENDING"),
            Self::Approved => write!(f, "APPROVED"),
            Self::Rejected => write!(f, "REJECTED"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LedgerEntryType {
    Debit,
    Credit,
}

impl std::fmt::Display for LedgerEntryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Debit => write!(f, "DEBIT"),
            Self::Credit => write!(f, "CREDIT"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LedgerEntry {
    pub account: String,
    pub entry_type: LedgerEntryType,
    pub amount: i64,
}

pub fn validate_double_entry_balance(entries: &[LedgerEntry]) -> Result<(), ValidationError> {
    let debits: i64 = entries.iter().filter(|e| e.entry_type == LedgerEntryType::Debit).map(|e| e.amount).sum();
    let credits: i64 = entries.iter().filter(|e| e.entry_type == LedgerEntryType::Credit).map(|e| e.amount).sum();

    if debits != credits {
        return Err(ValidationError::InvalidFormat("ledger".to_string(), format!("Double-entry balance mismatch: Debits={} != Credits={}", debits, credits)));
    }
    Ok(())
}