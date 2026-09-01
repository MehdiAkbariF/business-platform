use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum IndexabilityStatus {
    Index,
    Noindex,
}

impl std::fmt::Display for IndexabilityStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Index => write!(f, "INDEX"),
            Self::Noindex => write!(f, "NOINDEX"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PageMetadata {
    pub title: String,
    pub description: String,
    pub canonical_url: String,
    pub robots: String,
    pub og_title: String,
    pub og_description: String,
    pub og_image: Option<String>,
    pub og_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct StructuredDataJsonLd {
    pub context: String, // "https://schema.org"
    pub schema_type: String, // "LocalBusiness", "Restaurant", etc.
    pub name: String,
    pub description: Option<String>,
    pub url: String,
    pub telephone: Option<String>,
    pub address: Option<StructuredAddress>,
    pub geo: Option<StructuredGeo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct StructuredAddress {
    pub street_address: String,
    pub address_locality: String,
    pub address_region: String,
    pub address_country: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct StructuredGeo {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SitemapEntry {
    pub loc: String,
    pub lastmod: DateTime<Utc>,
    pub changefreq: Option<String>,
    pub priority: Option<f32>,
}