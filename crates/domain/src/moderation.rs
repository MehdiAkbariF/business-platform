use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared::{BusinessId, CaseId, ClaimId, DecisionId, ReportId, UserId};
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum VerificationStatus {
    Unverified,
    Pending,
    Verified,
    Expired,
    Revoked,
}

impl std::fmt::Display for VerificationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unverified => write!(f, "UNVERIFIED"),
            Self::Pending => write!(f, "PENDING"),
            Self::Verified => write!(f, "VERIFIED"),
            Self::Expired => write!(f, "EXPIRED"),
            Self::Revoked => write!(f, "REVOKED"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ClaimStatus {
    NotClaimed,
    Pending,
    Claimed,
    Rejected,
    Revoked,
}

impl std::fmt::Display for ClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotClaimed => write!(f, "NOT_CLAIMED"),
            Self::Pending => write!(f, "PENDING"),
            Self::Claimed => write!(f, "CLAIMED"),
            Self::Rejected => write!(f, "REJECTED"),
            Self::Revoked => write!(f, "REVOKED"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ClaimMethod {
    Email,
    Phone,
    Document,
    ManualReview,
}

impl std::fmt::Display for ClaimMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Email => write!(f, "EMAIL"),
            Self::Phone => write!(f, "PHONE"),
            Self::Document => write!(f, "DOCUMENT"),
            Self::ManualReview => write!(f, "MANUAL_REVIEW"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ModerationCaseType {
    BusinessSubmission,
    ProfileUpdate,
    ClaimReview,
    ReportReview,
}

impl std::fmt::Display for ModerationCaseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BusinessSubmission => write!(f, "BUSINESS_SUBMISSION"),
            Self::ProfileUpdate => write!(f, "PROFILE_UPDATE"),
            Self::ClaimReview => write!(f, "CLAIM_REVIEW"),
            Self::ReportReview => write!(f, "REPORT_REVIEW"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ModerationCaseStatus {
    Pending,
    InReview,
    Approved,
    Rejected,
    Escalated,
}

impl std::fmt::Display for ModerationCaseStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "PENDING"),
            Self::InReview => write!(f, "IN_REVIEW"),
            Self::Approved => write!(f, "APPROVED"),
            Self::Rejected => write!(f, "REJECTED"),
            Self::Escalated => write!(f, "ESCALATED"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ModerationReasonCode {
    Duplicate,
    Spam,
    InvalidInformation,
    ProhibitedContent,
    InsufficientInformation,
    UnauthorizedClaim,
    PolicyViolation,
}

impl std::fmt::Display for ModerationReasonCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Duplicate => write!(f, "DUPLICATE"),
            Self::Spam => write!(f, "SPAM"),
            Self::InvalidInformation => write!(f, "INVALID_INFORMATION"),
            Self::ProhibitedContent => write!(f, "PROHIBITED_CONTENT"),
            Self::InsufficientInformation => write!(f, "INSUFFICIENT_INFORMATION"),
            Self::UnauthorizedClaim => write!(f, "UNAUTHORIZED_CLAIM"),
            Self::PolicyViolation => write!(f, "POLICY_VIOLATION"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReportReason {
    FakeBusiness,
    WrongInformation,
    Duplicate,
    Scam,
    ProhibitedContent,
    ClosedBusiness,
}

impl std::fmt::Display for ReportReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FakeBusiness => write!(f, "FAKE_BUSINESS"),
            Self::WrongInformation => write!(f, "WRONG_INFORMATION"),
            Self::Duplicate => write!(f, "DUPLICATE"),
            Self::Scam => write!(f, "SCAM"),
            Self::ProhibitedContent => write!(f, "PROHIBITED_CONTENT"),
            Self::ClosedBusiness => write!(f, "CLOSED_BUSINESS"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReportStatus {
    Pending,
    InReview,
    Valid,
    Invalid,
    Dismissed,
}

impl std::fmt::Display for ReportStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "PENDING"),
            Self::InReview => write!(f, "IN_REVIEW"),
            Self::Valid => write!(f, "VALID"),
            Self::Invalid => write!(f, "INVALID"),
            Self::Dismissed => write!(f, "DISMISSED"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ModerationCase {
    pub id: CaseId,
    pub business_id: BusinessId,
    pub case_type: ModerationCaseType,
    pub priority: i16,
    pub status: ModerationCaseStatus,
    pub assigned_to: Option<UserId>,
    pub version: i32,
    pub created_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct ModerationDecision {
    pub id: DecisionId,
    pub case_id: CaseId,
    pub actor_id: UserId,
    pub decision: String,
    pub reason_code: Option<ModerationReasonCode>,
    pub note: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct BusinessClaim {
    pub id: ClaimId,
    pub business_id: BusinessId,
    pub claimant_id: UserId,
    pub status: ClaimStatus,
    pub method: ClaimMethod,
    pub evidence_text: Option<String>,
    pub submitted_at: DateTime<Utc>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub reviewed_by: Option<UserId>,
}

#[derive(Debug, Clone)]
pub struct BusinessReport {
    pub id: ReportId,
    pub business_id: BusinessId,
    pub reporter_id: UserId,
    pub reason: ReportReason,
    pub description: Option<String>,
    pub status: ReportStatus,
    pub created_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
}