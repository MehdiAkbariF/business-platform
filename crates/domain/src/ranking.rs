use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared::BusinessId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RankingProfileType {
    Default,
    Local,
    Quality,
    Discovery,
}

impl Default for RankingProfileType {
    fn default() -> Self {
        Self::Default
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankingWeights {
    pub relevance: f32,
    pub distance: f32,
    pub quality: f32,
    pub verification: f32,
    pub review_quality: f32,
    pub popularity: f32,
    pub freshness: f32,
}

impl RankingWeights {
    pub fn for_profile(profile: RankingProfileType) -> Self {
        match profile {
            RankingProfileType::Default => Self {
                relevance: 0.35,
                distance: 0.20,
                quality: 0.15,
                verification: 0.10,
                review_quality: 0.10,
                popularity: 0.05,
                freshness: 0.05,
            },
            RankingProfileType::Local => Self {
                relevance: 0.30,
                distance: 0.40,
                quality: 0.10,
                verification: 0.10,
                review_quality: 0.05,
                popularity: 0.03,
                freshness: 0.02,
            },
            RankingProfileType::Quality => Self {
                relevance: 0.30,
                distance: 0.10,
                quality: 0.25,
                verification: 0.15,
                review_quality: 0.15,
                popularity: 0.03,
                freshness: 0.02,
            },
            RankingProfileType::Discovery => Self {
                relevance: 0.30,
                distance: 0.15,
                quality: 0.15,
                verification: 0.10,
                review_quality: 0.05,
                popularity: 0.05,
                freshness: 0.20,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct RawRankingFeatures {
    pub business_id: BusinessId,
    pub text_similarity: f32,
    pub distance_meters: Option<f64>,
    pub completeness_score: u8,
    pub is_verified: bool,
    pub raw_average_rating: Option<f32>,
    pub review_count: u32,
    pub trusted_views: u64,
    pub trusted_clicks: u64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct NormalizedFeatures {
    pub text_relevance: f32,
    pub distance_score: f32,
    pub completeness: f32,
    pub verification: f32,
    pub review_quality: f32,
    pub popularity: f32,
    pub freshness: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct RankingExplanation {
    pub business_id: BusinessId,
    pub final_score: f32,
    pub ranking_version: &'static str,
    pub feature_schema_version: &'static str,
    pub profile: RankingProfileType,
    pub features: NormalizedFeatures,
}

// Distance exponential decay formula: exp(-d / scale)
pub fn calculate_distance_score(distance_meters: Option<f64>, scale_km: f64) -> f32 {
    match distance_meters {
        Some(d) => {
            let scale_m = (scale_km * 1000.0).max(100.0);
            ((-d / scale_m).exp() as f32).clamp(0.0, 1.0)
        }
        None => 0.5, // Neutral score when location query is absent
    }
}

// Bayesian shrinkage for ratings: (v / (v + m)) * R + (m / (v + m)) * C
pub fn calculate_bayesian_rating(raw_rating: Option<f32>, review_count: u32, confidence_m: f32, global_prior_c: f32) -> f32 {
    let v = review_count as f32;
    let r = raw_rating.unwrap_or(global_prior_c);
    let adjusted = ((v / (v + confidence_m)) * r) + ((confidence_m / (v + confidence_m)) * global_prior_c);
    (adjusted / 5.0).clamp(0.0, 1.0) // Normalize 0..5 scale to 0..1
}

// Logarithmic engagement normalization
pub fn calculate_popularity_score(views: u64, clicks: u64) -> f32 {
    let raw_points = (views as f64 * 0.1) + (clicks as f64 * 1.0);
    let log_score = (1.0 + raw_points).ln() / (1.0 + 1000.0_f64).ln();
    (log_score as f32).clamp(0.0, 1.0)
}

// Freshness decay: exp(-days / half_life)
pub fn calculate_freshness_score(created_at: DateTime<Utc>, half_life_days: f64) -> f32 {
    let days_old = (Utc::now() - created_at).num_days().max(0) as f64;
    ((-days_old / half_life_days).exp() as f32).clamp(0.0, 1.0)
}