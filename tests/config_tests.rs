use infrastructure::config::AppConfig;

#[test]
fn test_config_missing_mandatory_fields_fails() {
    std::env::remove_var("DATABASE_URL");
    let result = AppConfig::load();
    assert!(result.is_err());
}