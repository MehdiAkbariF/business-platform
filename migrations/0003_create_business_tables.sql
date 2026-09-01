-- Business Enums
CREATE TYPE business_status AS ENUM ('DRAFT', 'PENDING_REVIEW', 'PUBLISHED', 'SUSPENDED', 'ARCHIVED');
CREATE TYPE membership_role AS ENUM ('OWNER', 'ADMIN', 'EDITOR');
CREATE TYPE membership_status AS ENUM ('ACTIVE', 'INACTIVE');
CREATE TYPE location_accuracy AS ENUM ('EXACT', 'APPROXIMATE');

-- Businesses Table
CREATE TABLE IF NOT EXISTS businesses (
    id UUID PRIMARY KEY,
    slug VARCHAR(150) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    description TEXT NULL,
    status business_status NOT NULL DEFAULT 'DRAFT',
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    published_at TIMESTAMPTZ NULL
);

CREATE INDEX IF NOT EXISTS idx_businesses_status ON businesses(status);
CREATE INDEX IF NOT EXISTS idx_businesses_created_by ON businesses(created_by);

-- Business Memberships Table
CREATE TABLE IF NOT EXISTS business_memberships (
    id UUID PRIMARY KEY,
    business_id UUID NOT NULL REFERENCES businesses(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role membership_role NOT NULL,
    status membership_status NOT NULL DEFAULT 'ACTIVE',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_business_member UNIQUE (user_id, business_id)
);

CREATE INDEX IF NOT EXISTS idx_business_memberships_business ON business_memberships(business_id);
CREATE INDEX IF NOT EXISTS idx_business_memberships_user ON business_memberships(user_id);

-- Enforce at most 1 active OWNER per Business at database index level
CREATE UNIQUE INDEX IF NOT EXISTS idx_business_one_active_owner 
ON business_memberships (business_id) 
WHERE role = 'OWNER' AND status = 'ACTIVE';

-- Business Locations with PostGIS Point (SRID 4326)
CREATE TABLE IF NOT EXISTS business_locations (
    id UUID PRIMARY KEY,
    business_id UUID NOT NULL REFERENCES businesses(id) ON DELETE CASCADE,
    label VARCHAR(100) NOT NULL,
    geom GEOMETRY(Point, 4326) NOT NULL,
    country VARCHAR(100) NOT NULL DEFAULT 'IR',
    province VARCHAR(100) NOT NULL,
    city VARCHAR(100) NOT NULL,
    district VARCHAR(100) NULL,
    street VARCHAR(255) NOT NULL,
    postal_code VARCHAR(20) NULL,
    formatted_address TEXT NOT NULL,
    accuracy location_accuracy NOT NULL DEFAULT 'EXACT',
    is_primary BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_business_locations_business_id ON business_locations(business_id);
CREATE INDEX IF NOT EXISTS idx_business_locations_geom ON business_locations USING GIST (geom);

-- Enforce exactly at most 1 primary location per Business
CREATE UNIQUE INDEX IF NOT EXISTS idx_business_primary_location 
ON business_locations (business_id) 
WHERE is_primary = TRUE;

-- Business Contacts Table
CREATE TABLE IF NOT EXISTS business_contacts (
    business_id UUID PRIMARY KEY REFERENCES businesses(id) ON DELETE CASCADE,
    phone VARCHAR(50) NULL,
    mobile VARCHAR(50) NULL,
    email VARCHAR(255) NULL,
    website VARCHAR(255) NULL,
    preferred_contact_method VARCHAR(50) NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);