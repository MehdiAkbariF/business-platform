#[test]
fn test_exponential_backoff_calculation() {
    let calculate_backoff = |attempts: u32| -> i64 {
        (2_i64.pow(attempts) * 10).min(3600)
    };

    assert_eq!(calculate_backoff(1), 20);
    assert_eq!(calculate_backoff(2), 40);
    assert_eq!(calculate_backoff(3), 80);
    assert_eq!(calculate_backoff(10), 3600); // Caps safely at 1 hour
}