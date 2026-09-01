use async_trait::async_trait;
use chrono::{DateTime, Utc};
use domain::business::Business;
use domain::contact::BusinessContact;
use domain::location::BusinessLocation;
use domain::membership::{BusinessMembership, MembershipRole};
use domain::profile::{BusinessHoursInterval, BusinessMedia, SocialPlatform};
use domain::session::Session;
use domain::taxonomy::{Category, Service};
use domain::user::User;
use shared::{AttributeId, BusinessId, CategoryId, MediaId, ServiceId, SessionId, SocialLinkId, TokenFamilyId, UserId};
use crate::errors::AppError;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, id: UserId) -> Result<Option<User>, AppError>;
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError>;
    async fn create(&self, user: &User) -> Result<(), AppError>;
    async fn update_last_login(&self, id: UserId, login_time: DateTime<Utc>) -> Result<(), AppError>;
}

#[async_trait]
pub trait SessionRepository: Send + Sync {
    async fn create(&self, session: &Session) -> Result<(), AppError>;
    async fn find_by_id(&self, id: SessionId) -> Result<Option<Session>, AppError>;
    async fn find_by_refresh_token_hash_for_update(&self, hash: &str) -> Result<Option<Session>, AppError>;
    async fn rotate_token(&self, old_session_id: SessionId, new_session: &Session) -> Result<(), AppError>;
    async fn revoke_session(&self, id: SessionId) -> Result<(), AppError>;
    async fn revoke_token_family(&self, family_id: TokenFamilyId) -> Result<(), AppError>;
    async fn revoke_all_user_sessions(&self, user_id: UserId) -> Result<(), AppError>;
}

#[async_trait]
pub trait BusinessRepository: Send + Sync {
    async fn create_with_owner(&self, business: &Business, owner_membership: &BusinessMembership) -> Result<(), AppError>;
    async fn find_by_id(&self, id: BusinessId) -> Result<Option<Business>, AppError>;
    async fn find_by_slug(&self, slug: &str) -> Result<Option<Business>, AppError>;
    async fn update_profile(&self, id: BusinessId, name: &str, short_desc: Option<&str>, description: Option<&str>, timezone: &str) -> Result<(), AppError>;
    async fn update_status(&self, id: BusinessId, status: domain::business::BusinessStatus) -> Result<(), AppError>;
    async fn count_primary_locations(&self, business_id: BusinessId) -> Result<i64, AppError>;
    
    // Locations & Contacts
    async fn add_location(&self, location: &BusinessLocation) -> Result<(), AppError>;
    async fn get_locations(&self, business_id: BusinessId) -> Result<Vec<BusinessLocation>, AppError>;
    async fn get_primary_location(&self, business_id: BusinessId) -> Result<Option<BusinessLocation>, AppError>;
    async fn save_contact(&self, contact: &BusinessContact) -> Result<(), AppError>;
    async fn get_contact(&self, business_id: BusinessId) -> Result<Option<BusinessContact>, AppError>;
}

#[async_trait]
pub trait MembershipRepository: Send + Sync {
    async fn find_membership(&self, business_id: BusinessId, user_id: UserId) -> Result<Option<BusinessMembership>, AppError>;
    async fn list_members(&self, business_id: BusinessId) -> Result<Vec<BusinessMembership>, AppError>;
    async fn add_member(&self, membership: &BusinessMembership) -> Result<(), AppError>;
    async fn remove_member(&self, business_id: BusinessId, user_id: UserId) -> Result<(), AppError>;
    async fn change_role(&self, business_id: BusinessId, user_id: UserId, role: MembershipRole) -> Result<(), AppError>;
    async fn count_active_owners(&self, business_id: BusinessId) -> Result<i64, AppError>;
}

pub struct BusinessCategoryRow {
    pub category_id: CategoryId,
    pub name: String,
    pub slug: String,
    pub is_primary: bool,
}

pub struct BusinessServiceRow {
    pub service_id: ServiceId,
    pub name: String,
    pub slug: String,
    pub is_active: bool,
    pub sort_order: i32,
}

#[async_trait]
pub trait TaxonomyRepository: Send + Sync {
    async fn list_active_categories(&self) -> Result<Vec<Category>, AppError>;
    async fn find_category_by_id(&self, id: CategoryId) -> Result<Option<Category>, AppError>;
    async fn find_category_by_slug(&self, slug: &str) -> Result<Option<Category>, AppError>;
    async fn calculate_category_depth(&self, id: CategoryId) -> Result<usize, AppError>;

    async fn list_active_services(&self) -> Result<Vec<Service>, AppError>;
    async fn find_service_by_id(&self, id: ServiceId) -> Result<Option<Service>, AppError>;
    async fn find_service_by_slug(&self, slug: &str) -> Result<Option<Service>, AppError>;

    async fn is_service_compatible_with_categories(&self, service_id: ServiceId, category_ids: &[CategoryId]) -> Result<bool, AppError>;

    async fn get_business_categories(&self, business_id: BusinessId) -> Result<Vec<BusinessCategoryRow>, AppError>;
    async fn add_business_category(&self, business_id: BusinessId, category_id: CategoryId, is_primary: bool) -> Result<(), AppError>;
    async fn remove_business_category(&self, business_id: BusinessId, category_id: CategoryId) -> Result<(), AppError>;
    async fn set_primary_category(&self, business_id: BusinessId, category_id: CategoryId) -> Result<(), AppError>;
    async fn count_business_categories(&self, business_id: BusinessId) -> Result<i64, AppError>;
    async fn has_primary_category(&self, business_id: BusinessId) -> Result<bool, AppError>;

    async fn get_business_services(&self, business_id: BusinessId) -> Result<Vec<BusinessServiceRow>, AppError>;
    async fn add_business_service(&self, business_id: BusinessId, service_id: ServiceId, sort_order: i32) -> Result<(), AppError>;
    async fn remove_business_service(&self, business_id: BusinessId, service_id: ServiceId) -> Result<(), AppError>;
    async fn count_business_services(&self, business_id: BusinessId) -> Result<i64, AppError>;
}

pub struct BusinessAttributeRow {
    pub attribute_id: AttributeId,
    pub key: String,
    pub name: String,
    pub value_json: serde_json::Value,
}

pub struct BusinessSocialLinkRow {
    pub id: SocialLinkId,
    pub platform: SocialPlatform,
    pub url: String,
}

#[async_trait]
pub trait ProfileRepository: Send + Sync {
    // Media
    async fn save_media(&self, media: &BusinessMedia) -> Result<(), AppError>;
    async fn get_media(&self, business_id: BusinessId) -> Result<Vec<BusinessMedia>, AppError>;
    async fn find_media_by_id(&self, media_id: MediaId) -> Result<Option<BusinessMedia>, AppError>;
    async fn delete_media(&self, media_id: MediaId) -> Result<(), AppError>;

    // Hours
    async fn save_hours(&self, business_id: BusinessId, hours: &[BusinessHoursInterval]) -> Result<(), AppError>;
    async fn get_hours(&self, business_id: BusinessId) -> Result<Vec<BusinessHoursInterval>, AppError>;

    // Attributes
    async fn save_attributes(&self, business_id: BusinessId, attributes: &[(AttributeId, serde_json::Value)]) -> Result<(), AppError>;
    async fn get_attributes(&self, business_id: BusinessId) -> Result<Vec<BusinessAttributeRow>, AppError>;

    // Social Links
    async fn save_social_links(&self, business_id: BusinessId, links: &[(SocialPlatform, String)]) -> Result<(), AppError>;
    async fn get_social_links(&self, business_id: BusinessId) -> Result<Vec<BusinessSocialLinkRow>, AppError>;
}

#[async_trait]
pub trait AuditRepository: Send + Sync {
    async fn record(
        &self,
        actor: Option<UserId>,
        action: &str,
        ip: Option<String>,
        user_agent: Option<String>,
        metadata: Option<serde_json::Value>,
    ) -> Result<(), AppError>;
}