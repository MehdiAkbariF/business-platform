-- Transactional Outbox Table (Eliminates Dual-Write Inconsistencies)
CREATE TABLE IF NOT EXISTS outbox_events (
    id UUID PRIMARY KEY,
    event_type VARCHAR(100) NOT NULL,
    payload JSONB NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'PENDING', -- 'PENDING', 'PROCESSED', 'FAILED'
    retry_count INT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    processed_at TIMESTAMPTZ NULL
);

CREATE INDEX IF NOT EXISTS idx_outbox_events_status ON outbox_events(status, created_at ASC);

-- Background Jobs & Dead Letter Queue (DLQ) Tracking
CREATE TABLE IF NOT EXISTS background_jobs (
    id UUID PRIMARY KEY,
    job_type VARCHAR(100) NOT NULL,
    payload JSONB NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'QUEUED', -- 'QUEUED', 'PROCESSING', 'COMPLETED', 'DEAD_LETTER'
    attempts INT NOT NULL DEFAULT 0,
    max_attempts INT NOT NULL DEFAULT 5,
    last_error TEXT NULL,
    scheduled_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_background_jobs_polling ON background_jobs(status, scheduled_at ASC);
CREATE INDEX IF NOT EXISTS idx_background_jobs_dlq ON background_jobs(status) WHERE status = 'DEAD_LETTER';