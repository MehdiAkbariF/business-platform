use domain::recommendation::{CandidateSource, RecommendationCandidate, RecommendationSurface};
use shared::BusinessId;

#[test]
fn test_recommendation_surface_and_sources() {
    let home = RecommendationSurface::Home;
    assert_eq!(home.to_string(), "HOME");

    let detail = RecommendationSurface::BusinessDetail;
    assert_eq!(detail.to_string(), "BUSINESS_DETAIL");

    let b_id = BusinessId::new();
    let candidate = RecommendationCandidate {
        business_id: b_id,
        sources: vec![CandidateSource::Nearby, CandidateSource::Popular],
    };

    assert_eq!(candidate.sources.len(), 2);
    assert!(candidate.sources.contains(&CandidateSource::Nearby));
    assert!(candidate.sources.contains(&CandidateSource::Popular));
}