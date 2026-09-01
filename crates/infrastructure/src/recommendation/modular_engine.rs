use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use application::errors::AppError;
use application::ports::recommendation::{RecommendationEnginePort, RecommendationRepository};
use domain::recommendation::{CandidateSource, RecommendationCandidate, RecommendationContext, RecommendationSurface};
use shared::BusinessId;

pub struct ModularRecommendationEngine {
    repo: Arc<dyn RecommendationRepository>,
}

impl ModularRecommendationEngine {
    pub fn new(repo: Arc<dyn RecommendationRepository>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl RecommendationEnginePort for ModularRecommendationEngine {
    async fn generate_recommendations(&self, context: &RecommendationContext) -> Result<Vec<RecommendationCandidate>, AppError> {
        let mut candidate_map: HashMap<BusinessId, Vec<CandidateSource>> = HashMap::new();
        let generator_limit = context.limit.max(10);

        match context.surface {
            RecommendationSurface::Home => {
                // 1. Nearby Generator (if geo available)
                if let (Some(lat), Some(lon)) = (context.lat, context.lon) {
                    let nearby = self.repo.get_nearby_candidates(lat, lon, context.radius_km.unwrap_or(25.0), generator_limit).await?;
                    for id in nearby {
                        candidate_map.entry(id).or_default().push(CandidateSource::Nearby);
                    }
                }
                // 2. Trending Generator
                let trending = self.repo.get_trending_candidates(generator_limit).await?;
                for id in trending {
                    candidate_map.entry(id).or_default().push(CandidateSource::Trending);
                }
                // 3. Popular Generator
                let popular = self.repo.get_popular_candidates(context.city.as_deref(), generator_limit).await?;
                for id in popular {
                    candidate_map.entry(id).or_default().push(CandidateSource::Popular);
                }
                // 4. Fresh Generator
                let fresh = self.repo.get_fresh_candidates(generator_limit).await?;
                for id in fresh {
                    candidate_map.entry(id).or_default().push(CandidateSource::Fresh);
                }
            }
            RecommendationSurface::BusinessDetail => {
                if let Some(target_id) = context.business_id {
                    let similar = self.repo.get_similar_candidates(target_id, generator_limit).await?;
                    for id in similar {
                        candidate_map.entry(id).or_default().push(CandidateSource::Similar);
                    }
                }
            }
            RecommendationSurface::Category => {
                if let Some(cat_id) = context.category_id {
                    let cat_candidates = self.repo.get_category_candidates(cat_id, generator_limit).await?;
                    for id in cat_candidates {
                        candidate_map.entry(id).or_default().push(CandidateSource::Category);
                    }
                }
            }
            _ => {
                let popular = self.repo.get_popular_candidates(context.city.as_deref(), generator_limit).await?;
                for id in popular {
                    candidate_map.entry(id).or_default().push(CandidateSource::Popular);
                }
            }
        }

        // Deduplication & Source Merging
        let candidates = candidate_map
            .into_iter()
            .map(|(business_id, sources)| RecommendationCandidate { business_id, sources })
            .collect();

        Ok(candidates)
    }
}