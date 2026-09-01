use std::sync::Arc;
use domain::membership::{BusinessMembership, MembershipRole};
use shared::{BusinessId, ClientMetadata, MembershipId, UserId};
use utoipa::ToSchema;
use serde::{Deserialize, Serialize};
use crate::errors::AppError;
use crate::ports::repositories::{AuditRepository, MembershipRepository, UserRepository};

#[derive(Serialize, ToSchema)]
pub struct MemberDto {
    pub membership_id: MembershipId,
    pub user_id: UserId,
    pub role: String,
    pub status: String,
}

#[derive(Deserialize, ToSchema)]
pub struct AddMemberCommand {
    pub user_email: String,
    pub role: MembershipRole,
}

#[derive(Deserialize, ToSchema)]
pub struct ChangeRoleCommand {
    pub role: MembershipRole,
}

pub async fn list_members(
    membership_repo: Arc<dyn MembershipRepository>,
    business_id: BusinessId,
    user_id: UserId,
) -> Result<Vec<MemberDto>, AppError> {
    let caller_membership = membership_repo
        .find_membership(business_id, user_id)
        .await?
        .ok_or_else(|| AppError::Forbidden("You are not a member of this business".to_string()))?;

    if !caller_membership.can_manage_members() {
        return Err(AppError::Forbidden("Insufficient permissions to view members".to_string()));
    }

    let members = membership_repo.list_members(business_id).await?;
    Ok(members.into_iter().map(|m| MemberDto {
        membership_id: m.id,
        user_id: m.user_id,
        role: m.role.to_string(),
        status: m.status.to_string(),
    }).collect())
}

pub async fn add_member(
    user_repo: Arc<dyn UserRepository>,
    membership_repo: Arc<dyn MembershipRepository>,
    audit_repo: Arc<dyn AuditRepository>,
    business_id: BusinessId,
    caller_id: UserId,
    cmd: AddMemberCommand,
    metadata: ClientMetadata,
) -> Result<(), AppError> {
    let caller_membership = membership_repo
        .find_membership(business_id, caller_id)
        .await?
        .ok_or_else(|| AppError::Forbidden("You are not a member of this business".to_string()))?;

    if !caller_membership.can_manage_members() {
        return Err(AppError::Forbidden("Insufficient permissions to add members".to_string()));
    }

    if cmd.role == MembershipRole::Owner {
        return Err(AppError::Forbidden("Cannot add another OWNER. Ownership must be transferred.".to_string()));
    }

    let target_user = user_repo
        .find_by_email(&cmd.user_email.trim().to_lowercase())
        .await?
        .ok_or_else(|| AppError::NotFound("Target user not found".to_string()))?;

    if let Some(_) = membership_repo.find_membership(business_id, target_user.id).await? {
        return Err(AppError::Conflict("User is already a member of this business".to_string()));
    }

    let new_membership = BusinessMembership::new(MembershipId::new(), business_id, target_user.id, cmd.role);
    membership_repo.add_member(&new_membership).await?;

    let _ = audit_repo.record(
        Some(caller_id),
        "MEMBER_ADDED",
        metadata.ip_address,
        metadata.user_agent,
        Some(serde_json::json!({ "business_id": business_id.to_string(), "target_user_id": target_user.id.to_string(), "role": cmd.role.to_string() })),
    ).await;

    Ok(())
}

pub async fn remove_member(
    membership_repo: Arc<dyn MembershipRepository>,
    audit_repo: Arc<dyn AuditRepository>,
    business_id: BusinessId,
    caller_id: UserId,
    target_user_id: UserId,
    metadata: ClientMetadata,
) -> Result<(), AppError> {
    let caller_membership = membership_repo
        .find_membership(business_id, caller_id)
        .await?
        .ok_or_else(|| AppError::Forbidden("You are not a member of this business".to_string()))?;

    if !caller_membership.can_manage_members() {
        return Err(AppError::Forbidden("Insufficient permissions to remove members".to_string()));
    }

    let target_membership = membership_repo
        .find_membership(business_id, target_user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Membership not found".to_string()))?;

    if target_membership.role == MembershipRole::Owner {
        return Err(AppError::Forbidden("Cannot remove the OWNER of the business".to_string()));
    }

    membership_repo.remove_member(business_id, target_user_id).await?;

    let _ = audit_repo.record(
        Some(caller_id),
        "MEMBER_REMOVED",
        metadata.ip_address,
        metadata.user_agent,
        Some(serde_json::json!({ "business_id": business_id.to_string(), "removed_user_id": target_user_id.to_string() })),
    ).await;

    Ok(())
}

pub async fn change_member_role(
    membership_repo: Arc<dyn MembershipRepository>,
    audit_repo: Arc<dyn AuditRepository>,
    business_id: BusinessId,
    caller_id: UserId,
    target_user_id: UserId,
    cmd: ChangeRoleCommand,
    metadata: ClientMetadata,
) -> Result<(), AppError> {
    let caller_membership = membership_repo
        .find_membership(business_id, caller_id)
        .await?
        .ok_or_else(|| AppError::Forbidden("You are not a member of this business".to_string()))?;

    if !caller_membership.can_change_roles() {
        return Err(AppError::Forbidden("Only the OWNER can change member roles".to_string()));
    }

    let target_membership = membership_repo
        .find_membership(business_id, target_user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Membership not found".to_string()))?;

    if target_membership.role == MembershipRole::Owner && cmd.role != MembershipRole::Owner {
        return Err(AppError::Forbidden("Cannot demote the OWNER. Single active owner invariant required.".to_string()));
    }

    if cmd.role == MembershipRole::Owner {
        return Err(AppError::Forbidden("Direct promotion to OWNER is not permitted. Use ownership transfer.".to_string()));
    }

    membership_repo.change_role(business_id, target_user_id, cmd.role).await?;

    let _ = audit_repo.record(
        Some(caller_id),
        "MEMBER_ROLE_CHANGED",
        metadata.ip_address,
        metadata.user_agent,
        Some(serde_json::json!({ "business_id": business_id.to_string(), "target_user_id": target_user_id.to_string(), "new_role": cmd.role.to_string() })),
    ).await;

    Ok(())
}