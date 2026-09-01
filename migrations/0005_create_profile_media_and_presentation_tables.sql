-- Alter businesses table to add short_description and timezone
ALTER TABLE businesses ADD COLUMN IF NOT EXISTS short_description VARCHAR(300) NULL;
ALTER TABLE businesses ADD COLUMN IF NOT EXISTS timezone VARCHAR(50) NOT NULL DEFAULT 'Asia/Tehran';

-- Media Enums
CREATE TYPE media_type AS ENUM ('LOGO', 'COVER', 'GALLERY');
CREATE TYPE media_status AS ENUM ('PROCESSING', 'ACTIVE', 'REJECTED');

-- Business Media Table
CREATE TABLE IF NOT EXISTS business_media (
    id UUID PRIMARY KEY,
    business_id UUID NOT NULL REFERENCES businesses(id) ON DELETE CASCADE,
    media_type media_type NOT NULL,
    storage_key VARCHAR(500) NOT NULL,
    mime_type VARCHAR(50) NOT NULL,
    size_bytes BIGINT NOT NULL,
    width INT NOT NULL,
    height INT NOT NULL,
    alt_text VARCHAR(255) NULL,
    sort_order INT NOT NULL DEFAULT 0,
    status media_status NOT NULL DEFAULT 'ACTIVE',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_business_media_business_id ON business_media(business_id);

-- Invariants: At most 1 active LOGO and 1 active COVER per Business
CREATE UNIQUE INDEX IF NOT EXISTS idx_business_one_active_logo 
ON business_media (business_id) 
WHERE media_type = 'LOGO' AND status = 'ACTIVE';

CREATE UNIQUE INDEX IF NOT EXISTS idx_business_one_active_cover 
ON business_media (business_id) 
WHERE media_type = 'COVER' AND status = 'ACTIVE';

-- Structured Business Hours Table
CREATE TABLE IF NOT EXISTS business_hours (
    id UUID PRIMARY KEY,
    business_id UUID NOT NULL REFERENCES businesses(id) ON DELETE CASCADE,
    day_of_week SMALLINT NOT NULL CHECK (day_of_week BETWEEN 0 AND 6),
    opens_at VARCHAR(5) NULL,
    closes_at VARCHAR(5) NULL,
    is_24_hours BOOLEAN NOT NULL DEFAULT FALSE,
    sort_order INT NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_business_hours_business_id ON business_hours(business_id);

-- Business Attributes Enums & Definition Table
CREATE TYPE attribute_type AS ENUM ('BOOLEAN', 'ENUM');

CREATE TABLE IF NOT EXISTS attribute_definitions (
    id UUID PRIMARY KEY,
    key VARCHAR(50) NOT NULL UNIQUE,
    name VARCHAR(100) NOT NULL,
    attribute_type attribute_type NOT NULL,
    allowed_values JSONB NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'ACTIVE',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Category-Attribute Mapping (Which attributes apply to which categories)
CREATE TABLE IF NOT EXISTS category_attributes (
    category_id UUID NOT NULL REFERENCES categories(id) ON DELETE CASCADE,
    attribute_id UUID NOT NULL REFERENCES attribute_definitions(id) ON DELETE CASCADE,
    PRIMARY KEY (category_id, attribute_id)
);

-- Business Attributes Values
CREATE TABLE IF NOT EXISTS business_attributes (
    business_id UUID NOT NULL REFERENCES businesses(id) ON DELETE CASCADE,
    attribute_id UUID NOT NULL REFERENCES attribute_definitions(id) ON DELETE CASCADE,
    value_json JSONB NOT NULL,
    PRIMARY KEY (business_id, attribute_id)
);

-- Social Links
CREATE TYPE social_platform AS ENUM ('INSTAGRAM', 'TELEGRAM', 'WHATSAPP', 'LINKEDIN', 'FACEBOOK', 'YOUTUBE');

CREATE TABLE IF NOT EXISTS business_social_links (
    id UUID PRIMARY KEY,
    business_id UUID NOT NULL REFERENCES businesses(id) ON DELETE CASCADE,
    platform social_platform NOT NULL,
    url VARCHAR(500) NOT NULL,
    is_public BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_business_social_platform UNIQUE (business_id, platform)
);