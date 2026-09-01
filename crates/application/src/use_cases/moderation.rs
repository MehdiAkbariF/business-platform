use std::sync::Arc;
use domain::moderation::{
    BusinessClaim, BusinessReport, ClaimMethod, ClaimStatus, ModerationCase, ModerationCaseStatus,
    ModerationCaseType, ModerationDecision, ModerationReasonCode, ReportReason, ReportStatus,
};
use shared::{BusinessId, CaseId, ClaimId, ClientMetadata, DecisionId, ReportId, UserId};
use utoipa::ToSchema;
use serde::{Deserialize, Serialize};
use crate::errors::AppError;
use crate::ports::repositories::{AuditRepository, ModerationRepository};

#[derive(Serialize, ToSchema)]
pub struct ModerationCaseDto {
    pub id: CaseId,
    pub business_id: BusinessId,
    pub case_type: String,
    pub priority: i16,
    pub status: String,
    pub assigned_to: Option<UserId>,
    pub version: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Deserialize, ToSchema)]
pub struct DecisionCommand {
    pub reason_code: Option<ModerationReasonCode>,
    pub note: Option<String>,
    pub expected_version: i32,
}

#[derive(Deserialize, ToSchema)]
pub struct SubmitClaimCommand {
    pub method: ClaimMethod,
    pub evidence_text: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct BusinessClaimDto {
    pub id: ClaimId,
    pub business_id: BusinessId,
    pub claimant_id: UserId,
    pub status: String,
    pub method: String,
    pub submitted_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Deserialize, ToSchema)]
pub struct CreateReportCommand {
    pub reason: ReportReason,
    pub description: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct BusinessReportDto {
    pub id: ReportId,
    pub business_id: BusinessId,
    pub reporter_id: UserId,
    pub reason: String,
    pub description: Option<String>,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub async fn list_moderation_cases(
    mod_repo: Arc<dyn ModerationRepository>,
    limit: i64,
) -> Result<Vec<ModerationCaseDto>, AppError> {
    let cases = mod_repo.list_cases(limit).await?;
    Ok(cases.into_iter().map(|c| ModerationCaseDto {
        id: c.id,
        business_id: c.business_id,
        case_type: c.case_type.to_string(),
        priority: c.priority,
        status: c.status.to_string(),
        assigned_to: c.assigned_to,
        version: c.version,
        created_at: c.created_at,
    }).collect())
}

pub async fn get_moderation_case(
    mod_repo: Arc<dyn ModerationRepository>,
    case_id: CaseId,
) -> Result<ModerationCaseDto, AppError> {
    let c = mod_repo.find_case_by_id(case_id).await?
        .ok_or_else(|| AppError::NotFound("Moderation case not found".to_string()))?;

    Ok(ModerationCaseDto {
        id: c.id,
        business_id: c.business_id,
        case_type: c.case_type.to_string(),
        priority: c.priority,
        status: c.status.to_string(),
        assigned_to: c.assigned_to,
        version: c.version,
        created_at: c.created_at,
    })
}

pub async fn start_case_review(
    mod_repo: Arc<dyn ModerationRepository>,
    audit_repo: Arc<dyn AuditRepository>,
    case_id: CaseId,
    moderator_id: UserId,
    expected_version: i32,
    metadata: ClientMetadata,
) -> Result<(), AppError> {
    mod_repo.start_review(case_id, moderator_id, expected_version).await?;

    let _ = audit_repo.record(
        Some(moderator_id),
        "MODERATION_REVIEW_STARTED",
        metadata.ip_address,
        metadata.user_agent,
        Some(serde_json::json!({ "case_id": case_id.to_string() })),
    ).await;

    Ok(())
}

pub async fn approve_case(
    mod_repo: Arc<dyn ModerationRepository>,
    audit_repo: Arc<dyn AuditRepository>,
    case_id: CaseId,
    moderator_id: UserId,
    cmd: DecisionCommand,
    metadata: ClientMetadata,
) -> Result<(), AppError> {
    let decision = ModerationDecision {
        id: DecisionId::new(),
        case_id,
        actor_id: moderator_id,
        decision: "APPROVE".to_string(),
        reason_code: cmd.reason_code,
        note: cmd.note,
        created_at: chrono::Utc::now(),
    };

    mod_repo.resolve_case_approve(case_id, &decision, cmd.expected_version).await?;

    let _ = audit_repo.record(
        Some(moderator_id),
        "MODERATION_APPROVED",
        metadata.ip_address,
        metadata.user_agent,
        Some(serde_json::json!({ "case_id": case_id.to_string() })),
    ).await;

    Ok(())
}

pub async fn reject_case(
    mod_repo: Arc<dyn ModerationRepository>,
    audit_repo: Arc<dyn AuditRepository>,
    case_id: CaseId,
    moderator_id: UserId,
    cmd: DecisionCommand,
    metadata: ClientMetadata,
) -> Result<(), AppError> {
    let decision = ModerationDecision {
        id: DecisionId::new(),
        case_id,
        actor_id: moderator_id,
        decision: "REJECT".to_string(),
        reason_code: cmd.reason_code,
        note: cmd.note,
        created_at: chrono::Utc::now(),
    };

    mod_repo.resolve_case_reject(case_id, &decision, cmd.expected_version).await?;

    let _ = audit_repo.record(
        Some(moderator_id),
        "MODERATION_REJECTED",
        metadata.ip_address,
        metadata.user_agent,
        Some(serde_json::json!({ "case_id": case_id.to_string() })),
    ).await;

    Ok(())
}

pub async fn escalate_case(
    mod_repo: Arc<dyn ModerationRepository>,
    audit_repo: Arc<dyn AuditRepository>,
    case_id: CaseId,
    moderator_id: UserId,
    cmd: DecisionCommand,
    metadata: ClientMetadata,
) -> Result<(), AppError> {
    let decision = ModerationDecision {
        id: DecisionId::new(),
        case_id,
        actor_id: moderator_id,
        decision: "ESCALATE".to_string(),
        reason_code: cmd.reason_code,
        note: cmd.note,
        created_at: chrono::Utc::now(),
    };

    mod_repo.escalate_case(case_id, &decision, cmd.expected_version).await?;

    let _ = audit_repo.record(
        Some(moderator_id),
        "MODERATION_ESCALATED",
        metadata.ip_address,
        metadata.user_agent,
        Some(serde_json::json!({ "case_id": case_id.to_string() })),
    ).await;

    Ok(())
}

pub async fn suspend_business_cmd(
    mod_repo: Arc<dyn ModerationRepository>,
    audit_repo: Arc<dyn AuditRepository>,
    business_id: BusinessId,
    moderator_id: UserId,
    note: Option<String>,
    metadata: ClientMetadata,
) -> Result<(), AppError> {
    mod_repo.suspend_business(business_id, moderator_id, note).await?;

    let _ = audit_repo.record(
        Some(moderator_id),
        "BUSINESS_SUSPENDED_BY_ADMIN",
        metadata.ip_address,
        metadata.user_agent,
        Some(serde_json::json!({ "business_id": business_id.to_string() })),
    ).await;

    Ok(())
}

pub async fn restore_business_cmd(
    mod_repo: Arc<dyn ModerationRepository>,
    audit_repo: Arc<dyn AuditRepository>,
    business_id: BusinessId,
    moderator_id: UserId,
    metadata: ClientMetadata,
) -> Result<(), AppError> {
    mod_repo.restore_business(business_id, moderator_id).await?;

    let _ = audit_repo.record(
        Some(moderator_id),
        "BUSINESS_RESTORED_BY_ADMIN",
        metadata.ip_address,
        metadata.user_agent,
        Some(serde_json::json!({ "business_id": business_id.to_string() })),
    ).await;

    Ok(())
}

pub async fn submit_claim(
    mod_repo: Arc<dyn ModerationRepository>,
    audit_repo: Arc<dyn AuditRepository>,
    business_id: BusinessId,
    claimant_id: UserId,
    cmd: SubmitClaimCommand,
    metadata: ClientMetadata,
) -> Result<ClaimId, AppError> {
    let claim_id = ClaimId::new();
    let claim = BusinessClaim {
        id: claim_id,
        business_id,
        claimant_id,
        status: ClaimStatus::Pending,
        method: cmd.method,
        evidence_text: cmd.evidence_text,
        submitted_at: chrono::Utc::now(),
        reviewed_at: None,
        reviewed_by: None,
    };

    mod_repo.create_claim(&claim).await?;

    let case = ModerationCase {
        id: CaseId::new(),
        business_id,
        case_type: ModerationCaseType::ClaimReview,
        priority: 1,
        status: ModerationCaseStatus::Pending,
        assigned_to: None,
        version: 1,
        created_at: chrono::Utc::now(),
        resolved_at: None,
    };
    mod_repo.create_case(&case).await?;

    let _ = audit_repo.record(
        Some(claimant_id),
        "CLAIM_SUBMITTED",
        metadata.ip_address,
        metadata.user_agent,
        Some(serde_json::json!({ "business_id": business_id.to_string(), "claim_id": claim_id.to_string() })),
    ).await;

    Ok(claim_id)
}

pub async fn submit_report(
    mod_repo: Arc<dyn ModerationRepository>,
    audit_repo: Arc<dyn AuditRepository>,
    business_id: BusinessId,
    reporter_id: UserId,
    cmd: CreateReportCommand,
    metadata: ClientMetadata,
) -> Result<ReportId, AppError> {
    let report_id = ReportId::new();
    let report = BusinessReport {
        id: report_id,
        business_id,
        reporter_id,
        reason: cmd.reason,
        description: cmd.description,
        status: ReportStatus::Pending,
        created_at: chrono::Utc::now(),
        resolved_at: None,
    };

    mod_repo.create_report(&report).await?;

    let case = ModerationCase {
        id: CaseId::new(),
        business_id,
        case_type: ModerationCaseType::ReportReview,
        priority: 0,
        status: ModerationCaseStatus::Pending,
        assigned_to: None,
        version: 1,
        created_at: chrono::Utc::now(),
        resolved_at: None,
    };
    mod_repo.create_case(&case).await?;

    let _ = audit_repo.record(
        Some(reporter_id),
        "REPORT_SUBMITTED",
        metadata.ip_address,
        metadata.user_agent,
        Some(serde_json::json!({ "business_id": business_id.to_string(), "report_id": report_id.to_string() })),
    ).await;

    Ok(report_id)
}