-- Aggregated Analytics for Trusted Popularity & Trending Signals
CREATE TABLE IF NOT EXISTS business_analytics (
    business_id UUID PRIMARY KEY REFERENCES businesses(id) ON DELETE CASCADE,
    views_count BIGINT NOT NULL DEFAULT 0,
    clicks_count BIGINT NOT NULL DEFAULT 0,
    recent_views BIGINT NOT NULL DEFAULT 0,
    recent_clicks BIGINT NOT NULL DEFAULT 0,
    last_aggregated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Recommendation Audit & Analytics Events
CREATE TABLE IF NOT EXISTS recommendation_events (
    id UUID PRIMARY KEY,
    user_id UUID NULL REFERENCES users(id) ON DELETE SET NULL,
    surface VARCHAR(50) NOT NULL,
    business_id UUID NOT NULL REFERENCES businesses(id) ON DELETE CASCADE,
    event_type VARCHAR(50) NOT NULL, -- 'SHOWN', 'CLICKED', 'CONVERTED'
    position INT NOT NULL DEFAULT 0,
    strategy_version VARCHAR(20) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_recommendation_events_surface ON recommendation_events(surface);
CREATE INDEX IF NOT EXISTS idx_recommendation_events_created ON recommendation_events(created_at DESC);