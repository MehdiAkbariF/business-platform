use domain::business::{Business, BusinessSlug, BusinessStatus};
use shared::{BusinessId, UserId};

#[test]
fn test_publication_and_moderation_state_machine() {
    let mut business = Business::new(
        BusinessId::new(),
        BusinessSlug::parse("moderated-business").unwrap(),
        "Moderated Business".to_string(),
        None,
        None,
        UserId::new(),
    );

    // Initial state: DRAFT
    assert_eq!(business.status, BusinessStatus::Draft);

    // Direct transition to PUBLISHED forbidden
    assert!(!business.can_transition_to(BusinessStatus::Published));

    // Submit for review -> PENDING_REVIEW
    assert!(business.submit_for_review(true, true).is_ok());
    assert_eq!(business.status, BusinessStatus::PendingReview);

    // Moderator approves -> PUBLISHED
    assert!(business.can_transition_to(BusinessStatus::Published));

    // Or Moderator rejects -> REJECTED
    assert!(business.can_transition_to(BusinessStatus::Rejected));

    // Rejected business can be moved back to DRAFT for owner corrections
    let mut rejected_business = business.clone();
    rejected_business.status = BusinessStatus::Rejected;
    assert!(rejected_business.can_transition_to(BusinessStatus::Draft));
}