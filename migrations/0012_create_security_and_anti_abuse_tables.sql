-- Security Incidents & Threat Telemetry Table
CREATE TABLE IF NOT EXISTS security_incidents (
    id UUID PRIMARY KEY,
    severity VARCHAR(20) NOT NULL, -- 'LOW', 'MEDIUM', 'HIGH', 'CRITICAL'
    event_type VARCHAR(50) NOT NULL, -- 'SSRF_ATTEMPT', 'PATH_TRAVERSAL', 'RATE_LIMIT_BURST', 'SUSPICIOUS_LOGIN'
    actor_id UUID NULL REFERENCES users(id) ON DELETE SET NULL,
    ip_address VARCHAR(45) NOT NULL,
    details JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_security_incidents_severity ON security_incidents(severity);
CREATE INDEX IF NOT EXISTS idx_security_incidents_ip ON security_incidents(ip_address);
CREATE INDEX IF NOT EXISTS idx_security_incidents_created ON security_incidents(created_at DESC);

-- Blocked IPs and Autonomous Abuse Defense
CREATE TABLE IF NOT EXISTS blocked_ip_records (
    ip_address VARCHAR(45) PRIMARY KEY,
    reason TEXT NOT NULL,
    blocked_until TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);