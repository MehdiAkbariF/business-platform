use std::sync::Arc;
use domain::taxonomy::TaxonomyStatus;
use shared::{BusinessId, CategoryId, ClientMetadata, ServiceId, UserId};
use utoipa::ToSchema;
use serde::{Deserialize, Serialize};
use crate::errors::AppError;
use crate::ports::repositories::{AuditRepository, MembershipRepository, TaxonomyRepository};

#[derive(Serialize, ToSchema)]
pub struct CategoryDto {
    pub id: CategoryId,
    pub parent_id: Option<CategoryId>,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub sort_order: i32,
}

#[derive(Serialize, ToSchema)]
pub struct ServiceDto {
    pub id: ServiceId,
    pub parent_id: Option<ServiceId>,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub sort_order: i32,
}

#[derive(Serialize, ToSchema)]
pub struct BusinessCategoryDto {
    pub category_id: CategoryId,
    pub name: String,
    pub slug: String,
    pub is_primary: bool,
}

#[derive(Serialize, ToSchema)]
pub struct BusinessServiceDto {
    pub service_id: ServiceId,
    pub name: String,
    pub slug: String,
    pub is_active: bool,
    pub sort_order: i32,
}

#[derive(Deserialize, ToSchema)]
pub struct AssignCategoryCommand {
    pub category_id: CategoryId,
    pub is_primary: bool,
}

#[derive(Deserialize, ToSchema)]
pub struct AssignServiceCommand {
    pub service_id: ServiceId,
    pub sort_order: Option<i32>,
}

pub async fn list_categories(tax_repo: Arc<dyn TaxonomyRepository>) -> Result<Vec<CategoryDto>, AppError> {
    let list = tax_repo.list_active_categories().await?;
    Ok(list.into_iter().map(|c| CategoryDto {
        id: c.id,
        parent_id: c.parent_id,
        name: c.name,
        slug: c.slug.as_str().to_string(),
        description: c.description,
        sort_order: c.sort_order,
    }).collect())
}

pub async fn get_category_by_slug(tax_repo: Arc<dyn TaxonomyRepository>, slug: &str) -> Result<CategoryDto, AppError> {
    let cat = tax_repo
        .find_category_by_slug(slug)
        .await?
        .ok_or_else(|| AppError::NotFound("Category not found".to_string()))?;

    if cat.status != TaxonomyStatus::Active {
        return Err(AppError::NotFound("Category not found".to_string()));
    }

    Ok(CategoryDto {
        id: cat.id,
        parent_id: cat.parent_id,
        name: cat.name,
        slug: cat.slug.as_str().to_string(),
        description: cat.description,
        sort_order: cat.sort_order,
    })
}

pub async fn list_services(tax_repo: Arc<dyn TaxonomyRepository>) -> Result<Vec<ServiceDto>, AppError> {
    let list = tax_repo.list_active_services().await?;
    Ok(list.into_iter().map(|s| ServiceDto {
        id: s.id,
        parent_id: s.parent_id,
        name: s.name,
        slug: s.slug.as_str().to_string(),
        description: s.description,
        sort_order: s.sort_order,
    }).collect())
}

pub async fn get_service_by_slug(tax_repo: Arc<dyn TaxonomyRepository>, slug: &str) -> Result<ServiceDto, AppError> {
    let s = tax_repo
        .find_service_by_slug(slug)
        .await?
        .ok_or_else(|| AppError::NotFound("Service not found".to_string()))?;

    if s.status != TaxonomyStatus::Active {
        return Err(AppError::NotFound("Service not found".to_string()));
    }

    Ok(ServiceDto {
        id: s.id,
        parent_id: s.parent_id,
        name: s.name,
        slug: s.slug.as_str().to_string(),
        description: s.description,
        sort_order: s.sort_order,
    })
}

pub async fn assign_business_category(
    tax_repo: Arc<dyn TaxonomyRepository>,
    membership_repo: Arc<dyn MembershipRepository>,
    audit_repo: Arc<dyn AuditRepository>,
    business_id: BusinessId,
    user_id: UserId,
    max_categories: usize,
    cmd: AssignCategoryCommand,
    metadata: ClientMetadata,
) -> Result<(), AppError> {
    let membership = membership_repo
        .find_membership(business_id, user_id)
        .await?
        .ok_or_else(|| AppError::Forbidden("You are not a member of this business".to_string()))?;

    if !membership.can_edit_profile() {
        return Err(AppError::Forbidden("Insufficient permissions".to_string()));
    }

    let category = tax_repo
        .find_category_by_id(cmd.category_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Category not found".to_string()))?;

    if category.status != TaxonomyStatus::Active {
        return Err(AppError::Validation("Cannot assign an inactive category".to_string()));
    }

    let current_count = tax_repo.count_business_categories(business_id).await? as usize;
    if current_count >= max_categories {
        return Err(AppError::Validation(format!("Maximum category limit ({}) reached", max_categories)));
    }

    tax_repo.add_business_category(business_id, cmd.category_id, cmd.is_primary).await?;

    let _ = audit_repo.record(
        Some(user_id),
        "BUSINESS_CATEGORY_ASSIGNED",
        metadata.ip_address,
        metadata.user_agent,
        Some(serde_json::json!({ "business_id": business_id.to_string(), "category_id": cmd.category_id.to_string(), "is_primary": cmd.is_primary })),
    ).await;

    Ok(())
}

pub async fn remove_business_category(
    tax_repo: Arc<dyn TaxonomyRepository>,
    membership_repo: Arc<dyn MembershipRepository>,
    audit_repo: Arc<dyn AuditRepository>,
    business_id: BusinessId,
    user_id: UserId,
    category_id: CategoryId,
    metadata: ClientMetadata,
) -> Result<(), AppError> {
    let membership = membership_repo
        .find_membership(business_id, user_id)
        .await?
        .ok_or_else(|| AppError::Forbidden("You are not a member of this business".to_string()))?;

    if !membership.can_edit_profile() {
        return Err(AppError::Forbidden("Insufficient permissions".to_string()));
    }

    tax_repo.remove_business_category(business_id, category_id).await?;

    let _ = audit_repo.record(
        Some(user_id),
        "BUSINESS_CATEGORY_REMOVED",
        metadata.ip_address,
        metadata.user_agent,
        Some(serde_json::json!({ "business_id": business_id.to_string(), "category_id": category_id.to_string() })),
    ).await;

    Ok(())
}

pub async fn set_business_primary_category(
    tax_repo: Arc<dyn TaxonomyRepository>,
    membership_repo: Arc<dyn MembershipRepository>,
    audit_repo: Arc<dyn AuditRepository>,
    business_id: BusinessId,
    user_id: UserId,
    category_id: CategoryId,
    metadata: ClientMetadata,
) -> Result<(), AppError> {
    let membership = membership_repo
        .find_membership(business_id, user_id)
        .await?
        .ok_or_else(|| AppError::Forbidden("You are not a member of this business".to_string()))?;

    if !membership.can_edit_profile() {
        return Err(AppError::Forbidden("Insufficient permissions".to_string()));
    }

    tax_repo.set_primary_category(business_id, category_id).await?;

    let _ = audit_repo.record(
        Some(user_id),
        "BUSINESS_PRIMARY_CATEGORY_SET",
        metadata.ip_address,
        metadata.user_agent,
        Some(serde_json::json!({ "business_id": business_id.to_string(), "category_id": category_id.to_string() })),
    ).await;

    Ok(())
}

pub async fn assign_business_service(
    tax_repo: Arc<dyn TaxonomyRepository>,
    membership_repo: Arc<dyn MembershipRepository>,
    audit_repo: Arc<dyn AuditRepository>,
    business_id: BusinessId,
    user_id: UserId,
    max_services: usize,
    cmd: AssignServiceCommand,
    metadata: ClientMetadata,
) -> Result<(), AppError> {
    let membership = membership_repo
        .find_membership(business_id, user_id)
        .await?
        .ok_or_else(|| AppError::Forbidden("You are not a member of this business".to_string()))?;

    if !membership.can_edit_profile() {
        return Err(AppError::Forbidden("Insufficient permissions".to_string()));
    }

    let service = tax_repo
        .find_service_by_id(cmd.service_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Service not found".to_string()))?;

    if service.status != TaxonomyStatus::Active {
        return Err(AppError::Validation("Cannot assign an inactive service".to_string()));
    }

    let business_cats = tax_repo.get_business_categories(business_id).await?;
    let category_ids: Vec<CategoryId> = business_cats.into_iter().map(|c| c.category_id).collect();

    if category_ids.is_empty() {
        return Err(AppError::Validation("Assign at least one category to the business before adding services".to_string()));
    }

    let is_compatible = tax_repo.is_service_compatible_with_categories(cmd.service_id, &category_ids).await?;
    if !is_compatible {
        return Err(AppError::Validation("This service is not compatible with any of the business's categories".to_string()));
    }

    let current_count = tax_repo.count_business_services(business_id).await? as usize;
    if current_count >= max_services {
        return Err(AppError::Validation(format!("Maximum service limit ({}) reached", max_services)));
    }

    tax_repo.add_business_service(business_id, cmd.service_id, cmd.sort_order.unwrap_or(0)).await?;

    let _ = audit_repo.record(
        Some(user_id),
        "BUSINESS_SERVICE_ASSIGNED",
        metadata.ip_address,
        metadata.user_agent,
        Some(serde_json::json!({ "business_id": business_id.to_string(), "service_id": cmd.service_id.to_string() })),
    ).await;

    Ok(())
}

pub async fn remove_business_service(
    tax_repo: Arc<dyn TaxonomyRepository>,
    membership_repo: Arc<dyn MembershipRepository>,
    audit_repo: Arc<dyn AuditRepository>,
    business_id: BusinessId,
    user_id: UserId,
    service_id: ServiceId,
    metadata: ClientMetadata,
) -> Result<(), AppError> {
    let membership = membership_repo
        .find_membership(business_id, user_id)
        .await?
        .ok_or_else(|| AppError::Forbidden("You are not a member of this business".to_string()))?;

    if !membership.can_edit_profile() {
        return Err(AppError::Forbidden("Insufficient permissions".to_string()));
    }

    tax_repo.remove_business_service(business_id, service_id).await?;

    let _ = audit_repo.record(
        Some(user_id),
        "BUSINESS_SERVICE_REMOVED",
        metadata.ip_address,
        metadata.user_agent,
        Some(serde_json::json!({ "business_id": business_id.to_string(), "service_id": service_id.to_string() })),
    ).await;

    Ok(())
}