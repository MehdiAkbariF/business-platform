-- Search Trigram & Text Search Indexes

-- Trigram index on business name and description for typo-tolerant fast fuzzy search
CREATE INDEX IF NOT EXISTS idx_businesses_name_trgm ON businesses USING GIN (name gin_trgm_ops);
CREATE INDEX IF NOT EXISTS idx_businesses_short_desc_trgm ON businesses USING GIN (short_description gin_trgm_ops);

-- Trigram index on category and service names for fast autocomplete
CREATE INDEX IF NOT EXISTS idx_categories_name_trgm ON categories USING GIN (name gin_trgm_ops);
CREATE INDEX IF NOT EXISTS idx_services_name_trgm ON services USING GIN (name gin_trgm_ops);

-- Compound index for published business retrieval
CREATE INDEX IF NOT EXISTS idx_businesses_published_search ON businesses (status, id) WHERE status = 'PUBLISHED';