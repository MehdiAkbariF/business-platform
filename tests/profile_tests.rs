use domain::profile::{validate_safe_url, BusinessHoursInterval};
use application::use_cases::profile::calculate_completeness;

#[test]
fn test_safe_url_validation() {
    assert!(validate_safe_url("https://instagram.com/mybusiness").is_ok());
    assert!(validate_safe_url("http://example.com").is_ok());

    // Insecure & malicious schemes must be rejected
    assert!(validate_safe_url("javascript:alert(1)").is_err());
    assert!(validate_safe_url("data:text/html;base64,PHNjcmlwdD4=").is_err());
}

#[test]
fn test_business_hours_validation() {
    let regular = BusinessHoursInterval {
        day_of_week: 0,
        opens_at: Some("09:00".to_string()),
        closes_at: Some("18:00".to_string()),
        is_24_hours: false,
        sort_order: 0,
    };
    assert!(regular.validate().is_ok());

    // Overnight hours (e.g. 22:00 to 03:00)
    let overnight = BusinessHoursInterval {
        day_of_week: 4,
        opens_at: Some("22:00".to_string()),
        closes_at: Some("03:00".to_string()),
        is_24_hours: false,
        sort_order: 0,
    };
    assert!(overnight.validate().is_ok());

    // 24/7 hours
    let full_day = BusinessHoursInterval {
        day_of_week: 2,
        opens_at: None,
        closes_at: None,
        is_24_hours: true,
        sort_order: 0,
    };
    assert!(full_day.validate().is_ok());
}

#[test]
fn test_profile_completeness_calculation() {
    let empty_score = calculate_completeness(false, false, false, false, false, false);
    assert_eq!(empty_score, 0);

    let full_score = calculate_completeness(true, true, true, true, true, true);
    assert_eq!(full_score, 100);

    let partial_score = calculate_completeness(true, true, false, true, false, false);
    assert_eq!(partial_score, 55);
}