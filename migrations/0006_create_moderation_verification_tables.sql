-- Add REJECTED to business_status if not present
ALTER TYPE business_status ADD VALUE IF NOT EXISTS 'REJECTED';

-- New Core Enums
CREATE TYPE verification_status AS ENUM ('UNVERIFIED', 'PENDING', 'VERIFIED', 'EXPIRED', 'REVOKED');
CREATE TYPE claim_status AS ENUM ('NOT_CLAIMED', 'PENDING', 'CLAIMED', 'REJECTED', 'REVOKED');
CREATE TYPE claim_method AS ENUM ('EMAIL', 'PHONE', 'DOCUMENT', 'MANUAL_REVIEW');

CREATE TYPE moderation_case_type AS ENUM ('BUSINESS_SUBMISSION', 'PROFILE_UPDATE', 'CLAIM_REVIEW', 'REPORT_REVIEW');
CREATE TYPE moderation_case_status AS ENUM ('PENDING', 'IN_REVIEW', 'APPROVED', 'REJECTED', 'ESCALATED');
CREATE TYPE moderation_reason_code AS ENUM (
    'DUPLICATE',
    'SPAM',
    'INVALID_INFORMATION',
    'PROHIBITED_CONTENT',
    'INSUFFICIENT_INFORMATION',
    'UNAUTHORIZED_CLAIM',
    'POLICY_VIOLATION'
);

CREATE TYPE report_reason AS ENUM (
    'FAKE_BUSINESS',
    'WRONG_INFORMATION',
    'DUPLICATE',
    'SCAM',
    'PROHIBITED_CONTENT',
    'CLOSED_BUSINESS'
);
CREATE TYPE report_status AS ENUM ('PENDING', 'IN_REVIEW', 'VALID', 'INVALID', 'DISMISSED');

-- Alter businesses table
ALTER TABLE businesses ADD COLUMN IF NOT EXISTS verification_status verification_status NOT NULL DEFAULT 'UNVERIFIED';
ALTER TABLE businesses ADD COLUMN IF NOT EXISTS claim_status claim_status NOT NULL DEFAULT 'NOT_CLAIMED';
ALTER TABLE businesses ADD COLUMN IF NOT EXISTS version INT NOT NULL DEFAULT 1;

-- Moderation Cases Table
CREATE TABLE IF NOT EXISTS moderation_cases (
    id UUID PRIMARY KEY,
    business_id UUID NOT NULL REFERENCES businesses(id) ON DELETE CASCADE,
    case_type moderation_case_type NOT NULL,
    priority SMALLINT NOT NULL DEFAULT 0,
    status moderation_case_status NOT NULL DEFAULT 'PENDING',
    assigned_to UUID NULL REFERENCES users(id) ON DELETE SET NULL,
    version INT NOT NULL DEFAULT 1,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    resolved_at TIMESTAMPTZ NULL
);

CREATE INDEX IF NOT EXISTS idx_moderation_cases_status ON moderation_cases(status);
CREATE INDEX IF NOT EXISTS idx_moderation_cases_business ON moderation_cases(business_id);

-- Moderation Decisions (Append-only)
CREATE TABLE IF NOT EXISTS moderation_decisions (
    id UUID PRIMARY KEY,
    case_id UUID NOT NULL REFERENCES moderation_cases(id) ON DELETE CASCADE,
    actor_id UUID NOT NULL REFERENCES users(id),
    decision VARCHAR(20) NOT NULL,
    reason_code moderation_reason_code NULL,
    note TEXT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_moderation_decisions_case ON moderation_decisions(case_id);

-- Business Claims Table
CREATE TABLE IF NOT EXISTS business_claims (
    id UUID PRIMARY KEY,
    business_id UUID NOT NULL REFERENCES businesses(id) ON DELETE CASCADE,
    claimant_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    status claim_status NOT NULL DEFAULT 'PENDING',
    method claim_method NOT NULL DEFAULT 'MANUAL_REVIEW',
    evidence_text TEXT NULL,
    submitted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    reviewed_at TIMESTAMPTZ NULL,
    reviewed_by UUID NULL REFERENCES users(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_business_claims_business ON business_claims(business_id);
CREATE INDEX IF NOT EXISTS idx_business_claims_claimant ON business_claims(claimant_id);

-- Business Reports Table
CREATE TABLE IF NOT EXISTS business_reports (
    id UUID PRIMARY KEY,
    business_id UUID NOT NULL REFERENCES businesses(id) ON DELETE CASCADE,
    reporter_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    reason report_reason NOT NULL,
    description TEXT NULL,
    status report_status NOT NULL DEFAULT 'PENDING',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    resolved_at TIMESTAMPTZ NULL
);

CREATE INDEX IF NOT EXISTS idx_business_reports_business ON business_reports(business_id);

-- Trust Signals Table
CREATE TABLE IF NOT EXISTS trust_signals (
    id UUID PRIMARY KEY,
    business_id UUID NOT NULL REFERENCES businesses(id) ON DELETE CASCADE,
    signal_type VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_business_signal UNIQUE (business_id, signal_type)
);

CREATE INDEX IF NOT EXISTS idx_trust_signals_business ON trust_signals(business_id);