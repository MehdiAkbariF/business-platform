use domain::business::{Business, BusinessSlug, BusinessStatus};
use domain::membership::{BusinessMembership, MembershipRole};
use shared::{BusinessId, MembershipId, UserId};

#[test]
fn test_business_slug_validation() {
    assert!(BusinessSlug::parse("valid-business-slug_123").is_ok());
    assert!(BusinessSlug::parse("").is_err());
    assert!(BusinessSlug::parse("invalid slug with spaces!").is_err());
}

#[test]
fn test_business_state_machine_transitions() {
    let mut business = Business::new(
        BusinessId::new(),
        BusinessSlug::parse("my-store").unwrap(),
        "My Store".to_string(),
        None,
        None,
        UserId::new(),
    );

    // Initial state is Draft
    assert_eq!(business.status, BusinessStatus::Draft);

    // Draft -> Published directly is forbidden
    assert!(!business.can_transition_to(BusinessStatus::Published));

    // Draft -> PendingReview requires both primary location and primary category
    assert!(business.submit_for_review(false, false).is_err());
    assert!(business.submit_for_review(true, false).is_err());
    assert!(business.submit_for_review(false, true).is_err());
    assert!(business.submit_for_review(true, true).is_ok());
    assert_eq!(business.status, BusinessStatus::PendingReview);

    // PendingReview -> Archived is allowed
    assert!(business.archive().is_ok());
    assert_eq!(business.status, BusinessStatus::Archived);
}

#[test]
fn test_membership_permissions_enforcement() {
    let business_id = BusinessId::new();
    let owner = BusinessMembership::new(MembershipId::new(), business_id, UserId::new(), MembershipRole::Owner);
    let admin = BusinessMembership::new(MembershipId::new(), business_id, UserId::new(), MembershipRole::Admin);
    let editor = BusinessMembership::new(MembershipId::new(), business_id, UserId::new(), MembershipRole::Editor);

    // Editor cannot manage members or archive
    assert!(editor.can_edit_profile());
    assert!(!editor.can_manage_members());
    assert!(!editor.can_submit());
    assert!(!editor.can_archive());
    assert!(!editor.can_change_roles());

    // Admin can edit, manage members and submit, but cannot archive or change roles
    assert!(admin.can_edit_profile());
    assert!(admin.can_manage_members());
    assert!(admin.can_submit());
    assert!(!admin.can_archive());
    assert!(!admin.can_change_roles());

    // Owner has full control
    assert!(owner.can_edit_profile());
    assert!(owner.can_manage_members());
    assert!(owner.can_submit());
    assert!(owner.can_archive());
    assert!(owner.can_change_roles());
}