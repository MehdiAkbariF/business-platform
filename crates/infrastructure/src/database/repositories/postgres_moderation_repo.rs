use async_trait::async_trait;
use chrono::{DateTime, Utc};
use domain::moderation::{
    BusinessClaim, BusinessReport, ClaimMethod, ClaimStatus, ModerationCase, ModerationCaseStatus,
    ModerationCaseType, ModerationDecision, ReportReason, ReportStatus,
};
use shared::{BusinessId, CaseId, ClaimId, DecisionId, ReportId, UserId};
use sqlx::PgPool;
use application::errors::AppError;
use application::ports::repositories::ModerationRepository;
use uuid::Uuid;

pub struct PostgresModerationRepository {
    pool: PgPool,
}

impl PostgresModerationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct CaseRow {
    id: Uuid,
    business_id: Uuid,
    case_type: String,
    priority: i16,
    status: String,
    assigned_to: Option<Uuid>,
    version: i32,
    created_at: DateTime<Utc>,
    resolved_at: Option<DateTime<Utc>>,
}

impl From<CaseRow> for ModerationCase {
    fn from(r: CaseRow) -> Self {
        let case_type = match r.case_type.as_str() {
            "BUSINESS_SUBMISSION" => ModerationCaseType::BusinessSubmission,
            "PROFILE_UPDATE" => ModerationCaseType::ProfileUpdate,
            "CLAIM_REVIEW" => ModerationCaseType::ClaimReview,
            _ => ModerationCaseType::ReportReview,
        };
        let status = match r.status.as_str() {
            "PENDING" => ModerationCaseStatus::Pending,
            "IN_REVIEW" => ModerationCaseStatus::InReview,
            "APPROVED" => ModerationCaseStatus::Approved,
            "REJECTED" => ModerationCaseStatus::Rejected,
            _ => ModerationCaseStatus::Escalated,
        };

        ModerationCase {
            id: CaseId::from_uuid(r.id),
            business_id: BusinessId::from_uuid(r.business_id),
            case_type,
            priority: r.priority,
            status,
            assigned_to: r.assigned_to.map(UserId::from_uuid),
            version: r.version,
            created_at: r.created_at,
            resolved_at: r.resolved_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct ClaimRow {
    id: Uuid,
    business_id: Uuid,
    claimant_id: Uuid,
    status: String,
    method: String,
    evidence_text: Option<String>,
    submitted_at: DateTime<Utc>,
    reviewed_at: Option<DateTime<Utc>>,
    reviewed_by: Option<Uuid>,
}

#[derive(sqlx::FromRow)]
struct ReportRow {
    id: Uuid,
    business_id: Uuid,
    reporter_id: Uuid,
    reason: String,
    description: Option<String>,
    status: String,
    created_at: DateTime<Utc>,
    resolved_at: Option<DateTime<Utc>>,
}

#[async_trait]
impl ModerationRepository for PostgresModerationRepository {
    async fn create_case(&self, c: &ModerationCase) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO moderation_cases (id, business_id, case_type, priority, status, assigned_to, version, created_at) 
             VALUES ($1, $2, $3::moderation_case_type, $4, $5::moderation_case_status, $6, $7, $8)"
        )
        .bind(c.id.0)
        .bind(c.business_id.0)
        .bind(c.case_type.to_string())
        .bind(c.priority)
        .bind(c.status.to_string())
        .bind(c.assigned_to.map(|u| u.0))
        .bind(c.version)
        .bind(c.created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        Ok(())
    }

    async fn find_case_by_id(&self, id: CaseId) -> Result<Option<ModerationCase>, AppError> {
        let row = sqlx::query_as::<_, CaseRow>(
            "SELECT id, business_id, case_type::text, priority, status::text, assigned_to, version, created_at, resolved_at FROM moderation_cases WHERE id = $1"
        )
        .bind(id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        Ok(row.map(ModerationCase::from))
    }

    async fn list_cases(&self, limit: i64) -> Result<Vec<ModerationCase>, AppError> {
        let rows = sqlx::query_as::<_, CaseRow>(
            "SELECT id, business_id, case_type::text, priority, status::text, assigned_to, version, created_at, resolved_at 
             FROM moderation_cases ORDER BY priority DESC, created_at ASC LIMIT $1"
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        Ok(rows.into_iter().map(ModerationCase::from).collect())
    }

    async fn start_review(&self, case_id: CaseId, moderator_id: UserId, expected_version: i32) -> Result<(), AppError> {
        let res = sqlx::query(
            "UPDATE moderation_cases SET status = 'IN_REVIEW'::moderation_case_status, assigned_to = $1, version = version + 1 WHERE id = $2 AND version = $3"
        )
        .bind(moderator_id.0)
        .bind(case_id.0)
        .bind(expected_version)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        if res.rows_affected() == 0 {
            return Err(AppError::Conflict("Moderation case was modified by another user (Version Conflict)".to_string()));
        }
        Ok(())
    }

    async fn resolve_case_approve(&self, case_id: CaseId, decision: &ModerationDecision, expected_version: i32) -> Result<(), AppError> {
        let mut tx = self.pool.begin().await.map_err(|e| AppError::internal(e))?;

        let case_row = sqlx::query_as::<_, CaseRow>(
            "SELECT id, business_id, case_type::text, priority, status::text, assigned_to, version, created_at, resolved_at FROM moderation_cases WHERE id = $1"
        )
        .bind(case_id.0)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| AppError::internal(e))?;

        let res = sqlx::query(
            "UPDATE moderation_cases SET status = 'APPROVED'::moderation_case_status, resolved_at = NOW(), version = version + 1 WHERE id = $1 AND version = $2"
        )
        .bind(case_id.0)
        .bind(expected_version)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::internal(e))?;

        if res.rows_affected() == 0 {
            return Err(AppError::Conflict("Concurrent modification conflict on moderation case".to_string()));
        }

        sqlx::query(
            "INSERT INTO moderation_decisions (id, case_id, actor_id, decision, reason_code, note, created_at) 
             VALUES ($1, $2, $3, $4, $5::moderation_reason_code, $6, $7)"
        )
        .bind(decision.id.0)
        .bind(decision.case_id.0)
        .bind(decision.actor_id.0)
        .bind(&decision.decision)
        .bind(decision.reason_code.map(|r| r.to_string()))
        .bind(&decision.note)
        .bind(decision.created_at)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::internal(e))?;

        if case_row.case_type == "BUSINESS_SUBMISSION" {
            sqlx::query("UPDATE businesses SET status = 'PUBLISHED'::business_status, published_at = NOW(), updated_at = NOW() WHERE id = $1")
                .bind(case_row.business_id)
                .execute(&mut *tx)
                .await
                .map_err(|e| AppError::internal(e))?;
        }

        tx.commit().await.map_err(|e| AppError::internal(e))?;
        Ok(())
    }

    async fn resolve_case_reject(&self, case_id: CaseId, decision: &ModerationDecision, expected_version: i32) -> Result<(), AppError> {
        let mut tx = self.pool.begin().await.map_err(|e| AppError::internal(e))?;

        let case_row = sqlx::query_as::<_, CaseRow>(
            "SELECT id, business_id, case_type::text, priority, status::text, assigned_to, version, created_at, resolved_at FROM moderation_cases WHERE id = $1"
        )
        .bind(case_id.0)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| AppError::internal(e))?;

        let res = sqlx::query(
            "UPDATE moderation_cases SET status = 'REJECTED'::moderation_case_status, resolved_at = NOW(), version = version + 1 WHERE id = $1 AND version = $2"
        )
        .bind(case_id.0)
        .bind(expected_version)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::internal(e))?;

        if res.rows_affected() == 0 {
            return Err(AppError::Conflict("Concurrent modification conflict on moderation case".to_string()));
        }

        sqlx::query(
            "INSERT INTO moderation_decisions (id, case_id, actor_id, decision, reason_code, note, created_at) 
             VALUES ($1, $2, $3, $4, $5::moderation_reason_code, $6, $7)"
        )
        .bind(decision.id.0)
        .bind(decision.case_id.0)
        .bind(decision.actor_id.0)
        .bind(&decision.decision)
        .bind(decision.reason_code.map(|r| r.to_string()))
        .bind(&decision.note)
        .bind(decision.created_at)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::internal(e))?;

        if case_row.case_type == "BUSINESS_SUBMISSION" {
            sqlx::query("UPDATE businesses SET status = 'REJECTED'::business_status, updated_at = NOW() WHERE id = $1")
                .bind(case_row.business_id)
                .execute(&mut *tx)
                .await
                .map_err(|e| AppError::internal(e))?;
        }

        tx.commit().await.map_err(|e| AppError::internal(e))?;
        Ok(())
    }

    async fn escalate_case(&self, case_id: CaseId, decision: &ModerationDecision, expected_version: i32) -> Result<(), AppError> {
        let mut tx = self.pool.begin().await.map_err(|e| AppError::internal(e))?;

        let res = sqlx::query(
            "UPDATE moderation_cases SET status = 'ESCALATED'::moderation_case_status, priority = priority + 5, version = version + 1 WHERE id = $1 AND version = $2"
        )
        .bind(case_id.0)
        .bind(expected_version)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::internal(e))?;

        if res.rows_affected() == 0 {
            return Err(AppError::Conflict("Concurrent modification conflict on moderation case".to_string()));
        }

        sqlx::query(
            "INSERT INTO moderation_decisions (id, case_id, actor_id, decision, reason_code, note, created_at) 
             VALUES ($1, $2, $3, $4, $5::moderation_reason_code, $6, $7)"
        )
        .bind(decision.id.0)
        .bind(decision.case_id.0)
        .bind(decision.actor_id.0)
        .bind(&decision.decision)
        .bind(decision.reason_code.map(|r| r.to_string()))
        .bind(&decision.note)
        .bind(decision.created_at)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::internal(e))?;

        tx.commit().await.map_err(|e| AppError::internal(e))?;
        Ok(())
    }

    async fn suspend_business(&self, business_id: BusinessId, _actor_id: UserId, _note: Option<String>) -> Result<(), AppError> {
        sqlx::query("UPDATE businesses SET status = 'SUSPENDED'::business_status, updated_at = NOW() WHERE id = $1")
            .bind(business_id.0)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::internal(e))?;

        Ok(())
    }

    async fn restore_business(&self, business_id: BusinessId, _actor_id: UserId) -> Result<(), AppError> {
        sqlx::query("UPDATE businesses SET status = 'PUBLISHED'::business_status, updated_at = NOW() WHERE id = $1")
            .bind(business_id.0)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::internal(e))?;

        Ok(())
    }

    async fn create_claim(&self, claim: &BusinessClaim) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO business_claims (id, business_id, claimant_id, status, method, evidence_text, submitted_at) 
             VALUES ($1, $2, $3, $4::claim_status, $5::claim_method, $6, $7)"
        )
        .bind(claim.id.0)
        .bind(claim.business_id.0)
        .bind(claim.claimant_id.0)
        .bind(claim.status.to_string())
        .bind(claim.method.to_string())
        .bind(&claim.evidence_text)
        .bind(claim.submitted_at)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        Ok(())
    }

    async fn get_claims_by_business(&self, business_id: BusinessId) -> Result<Vec<BusinessClaim>, AppError> {
        let rows = sqlx::query_as::<_, ClaimRow>(
            "SELECT id, business_id, claimant_id, status::text, method::text, evidence_text, submitted_at, reviewed_at, reviewed_by FROM business_claims WHERE business_id = $1"
        )
        .bind(business_id.0)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        Ok(rows.into_iter().map(|r| BusinessClaim {
            id: ClaimId::from_uuid(r.id),
            business_id: BusinessId::from_uuid(r.business_id),
            claimant_id: UserId::from_uuid(r.claimant_id),
            status: match r.status.as_str() {
                "CLAIMED" => ClaimStatus::Claimed,
                "REJECTED" => ClaimStatus::Rejected,
                _ => ClaimStatus::Pending,
            },
            method: ClaimMethod::ManualReview,
            evidence_text: r.evidence_text,
            submitted_at: r.submitted_at,
            reviewed_at: r.reviewed_at,
            reviewed_by: r.reviewed_by.map(UserId::from_uuid),
        }).collect())
    }

    async fn get_claims_by_user(&self, user_id: UserId) -> Result<Vec<BusinessClaim>, AppError> {
        let rows = sqlx::query_as::<_, ClaimRow>(
            "SELECT id, business_id, claimant_id, status::text, method::text, evidence_text, submitted_at, reviewed_at, reviewed_by FROM business_claims WHERE claimant_id = $1"
        )
        .bind(user_id.0)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        Ok(rows.into_iter().map(|r| BusinessClaim {
            id: ClaimId::from_uuid(r.id),
            business_id: BusinessId::from_uuid(r.business_id),
            claimant_id: UserId::from_uuid(r.claimant_id),
            status: match r.status.as_str() {
                "CLAIMED" => ClaimStatus::Claimed,
                "REJECTED" => ClaimStatus::Rejected,
                _ => ClaimStatus::Pending,
            },
            method: ClaimMethod::ManualReview,
            evidence_text: r.evidence_text,
            submitted_at: r.submitted_at,
            reviewed_at: r.reviewed_at,
            reviewed_by: r.reviewed_by.map(UserId::from_uuid),
        }).collect())
    }

    async fn create_report(&self, report: &BusinessReport) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO business_reports (id, business_id, reporter_id, reason, description, status, created_at) 
             VALUES ($1, $2, $3, $4::report_reason, $5, $6::report_status, $7)"
        )
        .bind(report.id.0)
        .bind(report.business_id.0)
        .bind(report.reporter_id.0)
        .bind(report.reason.to_string())
        .bind(&report.description)
        .bind(report.status.to_string())
        .bind(report.created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        Ok(())
    }

    async fn list_reports(&self, limit: i64) -> Result<Vec<BusinessReport>, AppError> {
        let rows = sqlx::query_as::<_, ReportRow>(
            "SELECT id, business_id, reporter_id, reason::text, description, status::text, created_at, resolved_at FROM business_reports ORDER BY created_at DESC LIMIT $1"
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        Ok(rows.into_iter().map(|r| BusinessReport {
            id: ReportId::from_uuid(r.id),
            business_id: BusinessId::from_uuid(r.business_id),
            reporter_id: UserId::from_uuid(r.reporter_id),
            reason: ReportReason::FakeBusiness,
            description: r.description,
            status: ReportStatus::Pending,
            created_at: r.created_at,
            resolved_at: r.resolved_at,
        }).collect())
    }

    async fn find_report_by_id(&self, id: ReportId) -> Result<Option<BusinessReport>, AppError> {
        let row = sqlx::query_as::<_, ReportRow>(
            "SELECT id, business_id, reporter_id, reason::text, description, status::text, created_at, resolved_at FROM business_reports WHERE id = $1"
        )
        .bind(id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        Ok(row.map(|r| BusinessReport {
            id: ReportId::from_uuid(r.id),
            business_id: BusinessId::from_uuid(r.business_id),
            reporter_id: UserId::from_uuid(r.reporter_id),
            reason: ReportReason::FakeBusiness,
            description: r.description,
            status: ReportStatus::Pending,
            created_at: r.created_at,
            resolved_at: r.resolved_at,
        }))
    }

    async fn get_trust_signals(&self, business_id: BusinessId) -> Result<Vec<String>, AppError> {
        #[derive(sqlx::FromRow)]
        struct SignalRow { signal_type: String }

        let rows = sqlx::query_as::<_, SignalRow>("SELECT signal_type FROM trust_signals WHERE business_id = $1")
            .bind(business_id.0)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::internal(e))?;

        Ok(rows.into_iter().map(|r| r.signal_type).collect())
    }

    async fn add_trust_signal(&self, business_id: BusinessId, signal: &str) -> Result<(), AppError> {
        sqlx::query("INSERT INTO trust_signals (id, business_id, signal_type, created_at) VALUES ($1, $2, $3, NOW()) ON CONFLICT DO NOTHING")
            .bind(Uuid::now_v7())
            .bind(business_id.0)
            .bind(signal)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::internal(e))?;

        Ok(())
    }
}