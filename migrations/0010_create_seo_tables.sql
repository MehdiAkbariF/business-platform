-- Slug Redirects Table (Prevents Broken Links and Redirect Chains)
CREATE TABLE IF NOT EXISTS slug_redirects (
    id UUID PRIMARY KEY,
    entity_type VARCHAR(20) NOT NULL, -- 'BUSINESS', 'CATEGORY'
    source_slug VARCHAR(150) NOT NULL UNIQUE,
    target_slug VARCHAR(150) NOT NULL,
    http_status INT NOT NULL DEFAULT 301,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_slug_redirects_source ON slug_redirects(source_slug);

-- SEO Landing Pages (Curated & High-Value Category + Location combinations)
CREATE TABLE IF NOT EXISTS seo_landing_pages (
    id UUID PRIMARY KEY,
    slug_path VARCHAR(200) NOT NULL UNIQUE, -- e.g. 'tehran/restaurants'
    city VARCHAR(100) NOT NULL,
    category_id UUID NOT NULL REFERENCES categories(id) ON DELETE CASCADE,
    title_override VARCHAR(255) NULL,
    meta_description_override TEXT NULL,
    is_indexable BOOLEAN NOT NULL DEFAULT TRUE,
    business_count INT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_seo_landing_city_cat ON seo_landing_pages(city, category_id);