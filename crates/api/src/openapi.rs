use utoipa::{
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
    Modify, OpenApi,
};

use crate::errors;
use crate::routes::{auth, business, health, moderation, profile, search, taxonomy, user};

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
        taxonomy::get_categories,
        taxonomy::get_category,
        taxonomy::get_services,
        taxonomy::get_service,
        taxonomy::get_business_categories,
        taxonomy::add_business_category,
        taxonomy::remove_business_category_endpoint,
        taxonomy::set_primary_category_endpoint,
        taxonomy::get_business_services,
        taxonomy::add_business_service_endpoint,
        taxonomy::remove_business_service_endpoint,
        profile::get_presentation,
        profile::set_business_hours,
        profile::set_business_attributes,
        profile::set_business_social_links,
        profile::delete_media_item,
        moderation::get_cases,
        moderation::get_case_detail,
        moderation::start_review_endpoint,
        moderation::approve_endpoint,
        moderation::reject_endpoint,
        moderation::escalate_endpoint,
        moderation::suspend_business_endpoint,
        moderation::restore_business_endpoint,
        moderation::claim_business_endpoint,
        moderation::report_business_endpoint,
        search::search_businesses,
        search::autocomplete_suggestions,
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
            application::use_cases::taxonomy::CategoryDto,
            application::use_cases::taxonomy::ServiceDto,
            application::use_cases::taxonomy::BusinessCategoryDto,
            application::use_cases::taxonomy::BusinessServiceDto,
            application::use_cases::taxonomy::AssignCategoryCommand,
            application::use_cases::taxonomy::AssignServiceCommand,
            application::use_cases::profile::PublicPresentationDto,
            application::use_cases::profile::MediaDto,
            application::use_cases::profile::AttributeItemDto,
            application::use_cases::profile::SocialLinkDto,
            application::use_cases::profile::SaveAttributesCommand,
            application::use_cases::profile::AttributePayload,
            application::use_cases::profile::SaveSocialLinksCommand,
            application::use_cases::profile::SocialLinkPayload,
            application::use_cases::moderation::ModerationCaseDto,
            application::use_cases::moderation::DecisionCommand,
            application::use_cases::moderation::SubmitClaimCommand,
            application::use_cases::moderation::CreateReportCommand,
            application::ports::search::SearchResultItemDto,
            application::ports::search::SearchResponseDto,
            application::ports::search::SuggestionItemDto,
            application::ports::search::SuggestionType,
            business::UpdateProfileRequest,
            domain::membership::MembershipRole,
            domain::profile::BusinessHoursInterval,
            domain::profile::MediaType,
            domain::profile::SocialPlatform,
            domain::moderation::ClaimMethod,
            domain::moderation::ReportReason,
            domain::moderation::ModerationReasonCode,
        )
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "Health", description = "Health and readiness probes"),
        (name = "Auth", description = "Authentication and session management"),
        (name = "User", description = "User profile operations"),
        (name = "Business", description = "Business aggregate & membership operations"),
        (name = "Taxonomy", description = "Category & Service taxonomy management"),
        (name = "Profile", description = "Structured profile, media, hours, and attributes"),
        (name = "Moderation", description = "Admin moderation cases, claims & trust verification"),
        (name = "Search", description = "Full text search, geo-discovery & autocomplete"),
    ),
    info(
        title = "Business Discovery Platform API",
        version = "0.6.0",
        description = "Production-grade Modular Monolith API with Text Search, PostGIS Geo-queries & Autocomplete"
    )
)]
pub struct ApiDoc;