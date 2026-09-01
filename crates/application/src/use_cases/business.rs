use std::sync::Arc;
use domain::business::{Business, BusinessSlug, BusinessStatus};
use domain::membership::{BusinessMembership, MembershipRole};
use shared::{BusinessId, ClientMetadata, LocationId, MembershipId, UserId};
use utoipa::ToSchema;
use serde::{Deserialize, Serialize};
use crate::errors::AppError;
use crate::ports::repositories::{AuditRepository, BusinessRepository, MembershipRepository, TaxonomyRepository};

#[derive(Deserialize, ToSchema)]
pub struct CreateBusinessCommand {
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct BusinessDto {
    pub id: BusinessId,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub created_by: UserId,
}

#[derive(Serialize, ToSchema)]
pub struct PublicBusinessProfileDto {
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub primary_location: Option<LocationDto>,
    pub contact: Option<ContactDto>,
}

#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct LocationDto {
    pub id: LocationId,
    pub label: String,
    pub latitude: f64,
    pub longitude: f64,
    pub formatted_address: String,
    pub is_primary: bool,
}

#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct ContactDto {
    pub phone: Option<String>,
    pub mobile: Option<String>,
    pub email: Option<String>,
    pub website: Option<String>,
}

pub async fn create_business(
    business_repo: Arc<dyn BusinessRepository>,
    audit_repo: Arc<dyn AuditRepository>,
    user_id: UserId,
    cmd: CreateBusinessCommand,
    metadata: ClientMetadata,
) -> Result<BusinessDto, AppError> {
    let slug = BusinessSlug::parse(&cmd.slug).map_err(|e| AppError::Validation(e.to_string()))?;
    let name = cmd.name.trim();
    if name.is_empty() {
        return Err(AppError::Validation("Business name is required".to_string()));
    }

    if let Some(_) = business_repo.find_by_slug(slug.as_str()).await? {
        return Err(AppError::Conflict("A business with this slug already exists".to_string()));
    }

    let business_id = BusinessId::new();
    let business = Business::new(business_id, slug, name.to_string(), cmd.description, user_id);
    let owner_membership = BusinessMembership::new(MembershipId::new(), business_id, user_id, MembershipRole::Owner);

    business_repo.create_with_owner(&business, &owner_membership).await?;

    let _ = audit_repo.record(
        Some(user_id),
        "BUSINESS_CREATED",
        metadata.ip_address,
        metadata.user_agent,
        Some(serde_json::json!({ "business_id": business_id.to_string() })),
    ).await;

    Ok(BusinessDto {
        id: business.id,
        slug: business.slug.as_str().to_string(),
        name: business.name,
        description: business.description,
        status: business.status.to_string(),
        created_by: business.created_by,
    })
}

pub async fn update_business_profile(
    business_repo: Arc<dyn BusinessRepository>,
    membership_repo: Arc<dyn MembershipRepository>,
    audit_repo: Arc<dyn AuditRepository>,
    business_id: BusinessId,
    user_id: UserId,
    name: String,
    description: Option<String>,
    metadata: ClientMetadata,
) -> Result<(), AppError> {
    let membership = membership_repo
        .find_membership(business_id, user_id)
        .await?
        .ok_or_else(|| AppError::Forbidden("You are not a member of this business".to_string()))?;

    if !membership.can_edit_profile() {
        return Err(AppError::Forbidden("Insufficient permissions to edit business profile".to_string()));
    }

    let trimmed_name = name.trim();
    if trimmed_name.is_empty() {
        return Err(AppError::Validation("Business name cannot be empty".to_string()));
    }

    business_repo.update_profile(business_id, trimmed_name, description.as_deref()).await?;

    let _ = audit_repo.record(
        Some(user_id),
        "BUSINESS_UPDATED",
        metadata.ip_address,
        metadata.user_agent,
        Some(serde_json::json!({ "business_id": business_id.to_string() })),
    ).await;

    Ok(())
}

pub async fn submit_business(
    business_repo: Arc<dyn BusinessRepository>,
    membership_repo: Arc<dyn MembershipRepository>,
    tax_repo: Arc<dyn TaxonomyRepository>,
    audit_repo: Arc<dyn AuditRepository>,
    business_id: BusinessId,
    user_id: UserId,
    metadata: ClientMetadata,
) -> Result<(), AppError> {
    let membership = membership_repo
        .find_membership(business_id, user_id)
        .await?
        .ok_or_else(|| AppError::Forbidden("You are not a member of this business".to_string()))?;

    if !membership.can_submit() {
        return Err(AppError::Forbidden("Only OWNER or ADMIN can submit the business for review".to_string()));
    }

    let mut business = business_repo
        .find_by_id(business_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Business not found".to_string()))?;

    let primary_location_count = business_repo.count_primary_locations(business_id).await?;
    let has_primary_location = primary_location_count > 0;

    let has_primary_category = tax_repo.has_primary_category(business_id).await?;

    business.submit_for_review(has_primary_location, has_primary_category).map_err(|e| AppError::Validation(e.to_string()))?;
    business_repo.update_status(business_id, business.status).await?;

    let _ = audit_repo.record(
        Some(user_id),
        "BUSINESS_SUBMITTED",
        metadata.ip_address,
        metadata.user_agent,
        Some(serde_json::json!({ "business_id": business_id.to_string() })),
    ).await;

    Ok(())
}

pub async fn archive_business(
    business_repo: Arc<dyn BusinessRepository>,
    membership_repo: Arc<dyn MembershipRepository>,
    audit_repo: Arc<dyn AuditRepository>,
    business_id: BusinessId,
    user_id: UserId,
    metadata: ClientMetadata,
) -> Result<(), AppError> {
    let membership = membership_repo
        .find_membership(business_id, user_id)
        .await?
        .ok_or_else(|| AppError::Forbidden("You are not a member of this business".to_string()))?;

    if !membership.can_archive() {
        return Err(AppError::Forbidden("Only the OWNER can archive the business".to_string()));
    }

    let mut business = business_repo
        .find_by_id(business_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Business not found".to_string()))?;

    business.archive().map_err(|e| AppError::Validation(e.to_string()))?;
    business_repo.update_status(business_id, business.status).await?;

    let _ = audit_repo.record(
        Some(user_id),
        "BUSINESS_ARCHIVED",
        metadata.ip_address,
        metadata.user_agent,
        Some(serde_json::json!({ "business_id": business_id.to_string() })),
    ).await;

    Ok(())
}

pub async fn get_public_profile(
    business_repo: Arc<dyn BusinessRepository>,
    slug: &str,
) -> Result<PublicBusinessProfileDto, AppError> {
    let business = business_repo
        .find_by_slug(slug)
        .await?
        .ok_or_else(|| AppError::NotFound("Business not found".to_string()))?;

    if business.status != BusinessStatus::Published {
        return Err(AppError::NotFound("Business not found".to_string()));
    }

    let primary_location = business_repo.get_primary_location(business.id).await?.map(|l| LocationDto {
        id: l.id,
        label: l.label,
        latitude: l.latitude,
        longitude: l.longitude,
        formatted_address: l.formatted_address,
        is_primary: l.is_primary,
    });

    let contact = business_repo.get_contact(business.id).await?.map(|c| ContactDto {
        phone: c.phone,
        mobile: c.mobile,
        email: c.email,
        website: c.website,
    });

    Ok(PublicBusinessProfileDto {
        slug: business.slug.as_str().to_string(),
        name: business.name,
        description: business.description,
        primary_location,
        contact,
    })
}

pub async fn get_management_profile(
    business_repo: Arc<dyn BusinessRepository>,
    membership_repo: Arc<dyn MembershipRepository>,
    business_id: BusinessId,
    user_id: UserId,
) -> Result<BusinessDto, AppError> {
    let _ = membership_repo
        .find_membership(business_id, user_id)
        .await?
        .ok_or_else(|| AppError::Forbidden("You are not a member of this business".to_string()))?;

    let business = business_repo
        .find_by_id(business_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Business not found".to_string()))?;

    Ok(BusinessDto {
        id: business.id,
        slug: business.slug.as_str().to_string(),
        name: business.name,
        description: business.description,
        status: business.status.to_string(),
        created_by: business.created_by,
    })
}