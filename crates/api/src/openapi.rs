use utoipa::{
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
    Modify, OpenApi,
};

use crate::errors;
use crate::routes::{auth, business, health, user};

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .description(Some("Enter your JWT access token"))
                        .build(),
                ),
            );
        }
    }
}

#[derive(OpenApi)]
#[openapi(
    paths(
        health::liveness,
        health::readiness,
        auth::register,
        auth::login,
        auth::refresh,
        auth::logout,
        auth::logout_all,
        user::get_me,
        business::create,
        business::get_public,
        business::get_management,
        business::update_profile,
        business::submit,
        business::archive,
        business::get_members,
        business::add_business_member,
        business::remove_business_member,
        business::change_role,
    ),
    components(
        schemas(
            health::LivenessResponse,
            health::ReadinessResponse,
            health::DependencyHealth,
            errors::ApiErrorResponse,
            errors::ErrorDetail,
            auth::RegisterRequest,
            auth::LoginRequest,
            auth::RefreshRequest,
            auth::AuthSuccessResponse,
            application::use_cases::get_me::UserProfileDto,
            application::use_cases::business::CreateBusinessCommand,
            application::use_cases::business::BusinessDto,
            application::use_cases::business::PublicBusinessProfileDto,
            application::use_cases::business::LocationDto,
            application::use_cases::business::ContactDto,
            application::use_cases::members::MemberDto,
            application::use_cases::members::AddMemberCommand,
            application::use_cases::members::ChangeRoleCommand,
            business::UpdateProfileRequest,
            domain::membership::MembershipRole,
        )
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "Health", description = "Health and readiness probes"),
        (name = "Auth", description = "Authentication and session management"),
        (name = "User", description = "User profile operations"),
        (name = "Business", description = "Business aggregate & membership operations"),
    ),
    info(
        title = "Business Discovery Platform API",
        version = "0.2.0",
        description = "Production-grade Modular Monolith API with PostGIS & Membership-based Ownership"
    )
)]
pub struct ApiDoc;