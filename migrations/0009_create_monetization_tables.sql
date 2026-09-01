-- Monetization Custom Enums
CREATE TYPE subscription_status AS ENUM ('TRIAL', 'ACTIVE', 'PAST_DUE', 'PAUSED', 'CANCELED', 'EXPIRED');
CREATE TYPE payment_status AS ENUM ('INITIATED', 'PENDING', 'PROCESSING', 'SUCCEEDED', 'FAILED', 'REFUNDED');
CREATE TYPE ledger_entry_type AS ENUM ('DEBIT', 'CREDIT');
CREATE TYPE campaign_status AS ENUM ('DRAFT', 'PENDING_REVIEW', 'ACTIVE', 'PAUSED', 'EXHAUSTED', 'COMPLETED', 'REJECTED');
CREATE TYPE creative_status AS ENUM ('PENDING', 'APPROVED', 'REJECTED');

-- Versioned Subscription Plans Table
CREATE TABLE IF NOT EXISTS subscription_plans (
    id UUID PRIMARY KEY,
    identifier VARCHAR(50) NOT NULL,
    version INT NOT NULL DEFAULT 1,
    name VARCHAR(100) NOT NULL,
    price_amount BIGINT NOT NULL,
    currency VARCHAR(10) NOT NULL DEFAULT 'IRR',
    billing_period VARCHAR(20) NOT NULL, -- 'MONTHLY', 'YEARLY'
    entitlements JSONB NOT NULL DEFAULT '[]',
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_plan_identifier_version UNIQUE (identifier, version)
);

-- Business Subscriptions Table
CREATE TABLE IF NOT EXISTS business_subscriptions (
    id UUID PRIMARY KEY,
    business_id UUID NOT NULL REFERENCES businesses(id) ON DELETE CASCADE,
    plan_id UUID NOT NULL REFERENCES subscription_plans(id),
    status subscription_status NOT NULL DEFAULT 'ACTIVE',
    current_period_start TIMESTAMPTZ NOT NULL,
    current_period_end TIMESTAMPTZ NOT NULL,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_business_subscriptions_business ON business_subscriptions(business_id);

-- Payments Table (Idempotent)
CREATE TABLE IF NOT EXISTS payments (
    id UUID PRIMARY KEY,
    business_id UUID NOT NULL REFERENCES businesses(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id),
    idempotency_key VARCHAR(100) NOT NULL UNIQUE,
    amount BIGINT NOT NULL,
    currency VARCHAR(10) NOT NULL DEFAULT 'IRR',
    provider VARCHAR(50) NOT NULL,
    provider_ref VARCHAR(100) NULL,
    status payment_status NOT NULL DEFAULT 'INITIATED',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_payments_business ON payments(business_id);
CREATE INDEX IF NOT EXISTS idx_payments_status ON payments(status);

-- Financial Transactions (Aggregate Root for Double-Entry Accounting)
CREATE TABLE IF NOT EXISTS financial_transactions (
    id UUID PRIMARY KEY,
    payment_id UUID NULL REFERENCES payments(id) ON DELETE SET NULL,
    description TEXT NOT NULL,
    currency VARCHAR(10) NOT NULL DEFAULT 'IRR',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Immutable Double-Entry Ledger
CREATE TABLE IF NOT EXISTS ledger_entries (
    id UUID PRIMARY KEY,
    transaction_id UUID NOT NULL REFERENCES financial_transactions(id) ON DELETE RESTRICT,
    account VARCHAR(100) NOT NULL, -- e.g. 'ASSETS:BANK', 'REVENUE:SUBSCRIPTIONS', 'LIABILITY:AD_CREDITS'
    entry_type ledger_entry_type NOT NULL,
    amount BIGINT NOT NULL CHECK (amount > 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_ledger_entries_transaction ON ledger_entries(transaction_id);

-- Advertising Campaigns Table
CREATE TABLE IF NOT EXISTS ad_campaigns (
    id UUID PRIMARY KEY,
    business_id UUID NOT NULL REFERENCES businesses(id) ON DELETE CASCADE,
    name VARCHAR(150) NOT NULL,
    status campaign_status NOT NULL DEFAULT 'DRAFT',
    total_budget BIGINT NOT NULL,
    spent_amount BIGINT NOT NULL DEFAULT 0,
    reserved_amount BIGINT NOT NULL DEFAULT 0,
    currency VARCHAR(10) NOT NULL DEFAULT 'IRR',
    start_date TIMESTAMPTZ NOT NULL,
    end_date TIMESTAMPTZ NOT NULL,
    targeting JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_ad_campaigns_business ON ad_campaigns(business_id);
CREATE INDEX IF NOT EXISTS idx_ad_campaigns_status ON ad_campaigns(status);

-- Advertising Creatives Table
CREATE TABLE IF NOT EXISTS ad_creatives (
    id UUID PRIMARY KEY,
    campaign_id UUID NOT NULL REFERENCES ad_campaigns(id) ON DELETE CASCADE,
    title VARCHAR(150) NOT NULL,
    description TEXT NOT NULL,
    image_url VARCHAR(500) NULL,
    destination_url VARCHAR(500) NOT NULL,
    status creative_status NOT NULL DEFAULT 'PENDING',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_ad_creatives_campaign ON ad_creatives(campaign_id);