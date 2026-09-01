use application::errors::AppError;
use application::ports::ranking::{RankedItem, RankingCandidate, RankingContext, RankingEnginePort};
use domain::ranking::{
    calculate_bayesian_rating, calculate_distance_score, calculate_freshness_score, calculate_popularity_score,
    NormalizedFeatures, RankingExplanation, RankingWeights,
};

pub struct DeterministicRankingEngine {
    distance_scale_km: f64,
    freshness_half_life_days: f64,
    rating_confidence_threshold: f32,
    global_rating_prior: f32,
}

impl DeterministicRankingEngine {
    pub fn new(distance_scale_km: f64) -> Self {
        Self {
            distance_scale_km,
            freshness_half_life_days: 45.0,
            rating_confidence_threshold: 5.0,
            global_rating_prior: 3.5,
        }
    }
}

impl RankingEnginePort for DeterministicRankingEngine {
    fn rank(&self, candidates: Vec<RankingCandidate>, context: &RankingContext) -> Result<Vec<RankedItem>, AppError> {
        let weights = RankingWeights::for_profile(context.profile);
        let mut ranked_items: Vec<RankedItem> = Vec::with_capacity(candidates.len());

        for candidate in candidates {
            let raw = candidate.raw_features;

            // Feature Normalization into [0.0, 1.0]
            let norm_features = NormalizedFeatures {
                text_relevance: raw.text_similarity.clamp(0.0, 1.0),
                distance_score: calculate_distance_score(raw.distance_meters, self.distance_scale_km),
                completeness: (raw.completeness_score as f32 / 100.0).clamp(0.0, 1.0),
                verification: if raw.is_verified { 1.0 } else { 0.0 },
                review_quality: calculate_bayesian_rating(
                    raw.raw_average_rating,
                    raw.review_count,
                    self.rating_confidence_threshold,
                    self.global_rating_prior,
                ),
                popularity: calculate_popularity_score(raw.trusted_views, raw.trusted_clicks),
                freshness: calculate_freshness_score(raw.created_at, self.freshness_half_life_days),
            };

            // Transparent Linear Weighted Scoring Formula
            let final_score = (weights.relevance * norm_features.text_relevance)
                + (weights.distance * norm_features.distance_score)
                + (weights.quality * norm_features.completeness)
                + (weights.verification * norm_features.verification)
                + (weights.review_quality * norm_features.review_quality)
                + (weights.popularity * norm_features.popularity)
                + (weights.freshness * norm_features.freshness);

            let explanation = RankingExplanation {
                business_id: raw.business_id,
                final_score,
                ranking_version: "v1",
                feature_schema_version: "v1",
                profile: context.profile,
                features: norm_features,
            };

            ranked_items.push(RankedItem {
                business_id: raw.business_id,
                final_score,
                explanation,
            });
        }

        // Deterministic Ordering with Explicit Tie-Breakers:
        // 1. final_score DESC
        // 2. business_id ASC
        ranked_items.sort_by(|a, b| {
            b.final_score
                .partial_cmp(&a.final_score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.business_id.0.cmp(&b.business_id.0))
        });

        Ok(ranked_items)
    }
}