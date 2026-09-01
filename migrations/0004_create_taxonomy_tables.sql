-- Taxonomy Custom Enums
CREATE TYPE taxonomy_status AS ENUM ('ACTIVE', 'INACTIVE');
CREATE TYPE taxonomy_type AS ENUM ('CATEGORY', 'SERVICE');

-- Categories Table
CREATE TABLE IF NOT EXISTS categories (
    id UUID PRIMARY KEY,
    parent_id UUID NULL REFERENCES categories(id) ON DELETE RESTRICT,
    name VARCHAR(150) NOT NULL,
    normalized_name VARCHAR(150) NOT NULL,
    slug VARCHAR(150) NOT NULL UNIQUE,
    description TEXT NULL,
    status taxonomy_status NOT NULL DEFAULT 'ACTIVE',
    sort_order INT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_categories_parent_id ON categories(parent_id);
CREATE INDEX IF NOT EXISTS idx_categories_status ON categories(status);
CREATE UNIQUE INDEX IF NOT EXISTS idx_categories_sibling_unique ON categories(COALESCE(parent_id, '00000000-0000-0000-0000-000000000000'::uuid), normalized_name) WHERE status = 'ACTIVE';

-- Services Table
CREATE TABLE IF NOT EXISTS services (
    id UUID PRIMARY KEY,
    parent_id UUID NULL REFERENCES services(id) ON DELETE RESTRICT,
    name VARCHAR(150) NOT NULL,
    normalized_name VARCHAR(150) NOT NULL,
    slug VARCHAR(150) NOT NULL UNIQUE,
    description TEXT NULL,
    status taxonomy_status NOT NULL DEFAULT 'ACTIVE',
    sort_order INT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_services_parent_id ON services(parent_id);
CREATE INDEX IF NOT EXISTS idx_services_status ON services(status);
CREATE UNIQUE INDEX IF NOT EXISTS idx_services_sibling_unique ON services(COALESCE(parent_id, '00000000-0000-0000-0000-000000000000'::uuid), normalized_name) WHERE status = 'ACTIVE';

-- Category-Service Compatibility Matrix
CREATE TABLE IF NOT EXISTS category_services (
    category_id UUID NOT NULL REFERENCES categories(id) ON DELETE RESTRICT,
    service_id UUID NOT NULL REFERENCES services(id) ON DELETE RESTRICT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (category_id, service_id)
);

CREATE INDEX IF NOT EXISTS idx_category_services_service ON category_services(service_id);

-- Business Categories Classification Table
CREATE TABLE IF NOT EXISTS business_categories (
    business_id UUID NOT NULL REFERENCES businesses(id) ON DELETE CASCADE,
    category_id UUID NOT NULL REFERENCES categories(id) ON DELETE RESTRICT,
    is_primary BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (business_id, category_id)
);

CREATE INDEX IF NOT EXISTS idx_business_categories_category ON business_categories(category_id);

-- Enforce exactly at most 1 primary category per Business
CREATE UNIQUE INDEX IF NOT EXISTS idx_business_one_primary_category 
ON business_categories (business_id) 
WHERE is_primary = TRUE;

-- Business Services Offering Table
CREATE TABLE IF NOT EXISTS business_services (
    business_id UUID NOT NULL REFERENCES businesses(id) ON DELETE CASCADE,
    service_id UUID NOT NULL REFERENCES services(id) ON DELETE RESTRICT,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    sort_order INT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (business_id, service_id)
);

CREATE INDEX IF NOT EXISTS idx_business_services_service ON business_services(service_id);

-- Taxonomy Aliases for Synonyms and Multilingual Search Pre-computation
CREATE TABLE IF NOT EXISTS taxonomy_aliases (
    id UUID PRIMARY KEY,
    taxonomy_type taxonomy_type NOT NULL,
    taxonomy_id UUID NOT NULL,
    term VARCHAR(150) NOT NULL,
    normalized_term VARCHAR(150) NOT NULL,
    language VARCHAR(10) NOT NULL DEFAULT 'fa',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_taxonomy_alias UNIQUE (taxonomy_type, taxonomy_id, normalized_term, language)
);

CREATE INDEX IF NOT EXISTS idx_taxonomy_aliases_lookup ON taxonomy_aliases(taxonomy_type, normalized_term);