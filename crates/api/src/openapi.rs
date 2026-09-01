use utoipa::OpenApi;
use crate::routes::health;
use crate::errors;

#[derive(OpenApi)]
#[openapi(
    paths(
        health::liveness,
        health::readiness,
    ),
    components(
        schemas(
            health::LivenessResponse,
            health::ReadinessResponse,
            health::DependencyHealth,
            errors::ApiErrorResponse,
            errors::ErrorDetail
        )
    ),
    tags(
        (name = "Health", description = "System liveness and readiness endpoints")
    ),
    info(
        title = "Business Discovery Platform API",
        version = "0.1.0",
        description = "Production-grade core backend API"
    )
)]
pub struct ApiDoc;