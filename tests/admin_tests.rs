use domain::admin::AppealStatus;

#[test]
fn test_appeal_status_transitions() {
    let pending = AppealStatus::Pending;
    assert_eq!(pending.to_string(), "PENDING");

    let accepted = AppealStatus::Accepted;
    assert_eq!(accepted.to_string(), "ACCEPTED");

    let rejected = AppealStatus::Rejected;
    assert_eq!(rejected.to_string(), "REJECTED");
}