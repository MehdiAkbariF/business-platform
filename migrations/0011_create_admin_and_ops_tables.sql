-- Appeals Table (Moderation Appeals)
CREATE TABLE IF NOT EXISTS business_appeals (
    id UUID PRIMARY KEY,
    business_id UUID NOT NULL REFERENCES businesses(id) ON DELETE CASCADE,
    case_id UUID NULL REFERENCES moderation_cases(id) ON DELETE SET NULL,
    reason TEXT NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'PENDING', -- 'PENDING', 'UNDER_REVIEW', 'ACCEPTED', 'REJECTED'
    submitted_by UUID NOT NULL REFERENCES users(id),
    resolved_by UUID NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    resolved_at TIMESTAMPTZ NULL
);

CREATE INDEX IF NOT EXISTS idx_business_appeals_business ON business_appeals(business_id);
CREATE INDEX IF NOT EXISTS idx_business_appeals_status ON business_appeals(status);

-- Feature Flags & Kill Switches Table
CREATE TABLE IF NOT EXISTS feature_flags (
    key VARCHAR(50) PRIMARY KEY,
    description TEXT NOT NULL,
    is_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    target_type VARCHAR(20) NOT NULL DEFAULT 'GLOBAL', -- 'GLOBAL', 'ROLE', 'USER'
    updated_by UUID NULL REFERENCES users(id),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Insert Default Safety Kill-Switches
INSERT INTO feature_flags (key, description, is_enabled, target_type, updated_at) VALUES
('disable_ads', 'Emergency kill-switch to pause ad delivery platform-wide', FALSE, 'GLOBAL', NOW()),
('disable_registration', 'Kill-switch to pause new user registrations', FALSE, 'GLOBAL', NOW()),
('disable_business_submission', 'Kill-switch to pause new business review submissions', FALSE, 'GLOBAL', NOW())
ON CONFLICT (key) DO NOTHING;

-- System Runtime Configurations (Non-Secret)
CREATE TABLE IF NOT EXISTS system_runtime_configs (
    key VARCHAR(50) PRIMARY KEY,
    value_json JSONB NOT NULL,
    category VARCHAR(50) NOT NULL, -- 'GENERAL', 'TAXONOMY', 'SECURITY', 'SEARCH'
    updated_by UUID NULL REFERENCES users(id),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);