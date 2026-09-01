use chrono::Utc;
use domain::ranking::{
    calculate_bayesian_rating, calculate_distance_score, calculate_freshness_score, calculate_popularity_score,
    RankingProfileType, RawRankingFeatures,
};
use application::ports::ranking::{RankingCandidate, RankingContext, RankingEnginePort};
use infrastructure::ranking::deterministic_engine::DeterministicRankingEngine;
use shared::BusinessId;

#[test]
fn test_distance_decay_exponential() {
    let close = calculate_distance_score(Some(500.0), 10.0);
    let far = calculate_distance_score(Some(15000.0), 10.0);
    assert!(close > far);
    assert!(close <= 1.0 && close >= 0.0);
    assert!(far <= 1.0 && far >= 0.0);
}

#[test]
fn test_bayesian_rating_shrinkage() {
    let few_reviews = calculate_bayesian_rating(Some(5.0), 1, 5.0, 3.5);
    let many_reviews = calculate_bayesian_rating(Some(4.8), 100, 5.0, 3.5);
    assert!(many_reviews > few_reviews);
}

#[test]
fn test_cold_start_does_not_zero_out() {
    let freshness = calculate_freshness_score(Utc::now(), 45.0);
    assert_eq!(freshness, 1.0);

    let popularity = calculate_popularity_score(0, 0);
    assert_eq!(popularity, 0.0);
}

#[test]
fn test_deterministic_ranking_ordering() {
    let engine = DeterministicRankingEngine::new(10.0);
    let id1 = BusinessId::new();
    let id2 = BusinessId::new();

    let candidate1 = RankingCandidate {
        raw_features: RawRankingFeatures {
            business_id: id1,
            text_similarity: 0.9,
            distance_meters: Some(200.0),
            completeness_score: 100,
            is_verified: true,
            raw_average_rating: Some(4.9),
            review_count: 50,
            trusted_views: 500,
            trusted_clicks: 50,
            created_at: Utc::now(),
        },
    };

    let candidate2 = RankingCandidate {
        raw_features: RawRankingFeatures {
            business_id: id2,
            text_similarity: 0.3,
            distance_meters: Some(20000.0),
            completeness_score: 30,
            is_verified: false,
            raw_average_rating: None,
            review_count: 0,
            trusted_views: 0,
            trusted_clicks: 0,
            created_at: Utc::now() - chrono::Duration::days(100),
        },
    };

    let context = RankingContext {
        profile: RankingProfileType::Default,
        has_geo_query: true,
        max_radius_km: 25.0,
    };

    let ranked = engine.rank(vec![candidate2, candidate1], &context).unwrap();
    assert_eq!(ranked[0].business_id, id1);
    assert!(ranked[0].final_score > ranked[1].final_score);
}