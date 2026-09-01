use domain::security::{sanitize_storage_key, validate_public_destination_url};

#[test]
fn test_ssrf_guard_blocks_internal_and_cloud_metadata() {
    // Valid public URLs
    assert!(validate_public_destination_url("https://example.com").is_ok());
    assert!(validate_public_destination_url("https://api.github.com/users").is_ok());

    // Prohibited internal IP ranges & localhost (SSRF Protection)
    assert!(validate_public_destination_url("http://127.0.0.1/admin").is_err());
    assert!(validate_public_destination_url("http://localhost:8080").is_err());
    assert!(validate_public_destination_url("http://192.168.1.1/router").is_err());
    assert!(validate_public_destination_url("http://10.0.0.1/secrets").is_err());
    assert!(validate_public_destination_url("http://169.254.169.254/latest/meta-data").is_err());
    assert!(validate_public_destination_url("gopher://127.0.0.1:25").is_err());
}

#[test]
fn test_path_traversal_guard_sanitizes_keys() {
    // Valid storage keys
    assert!(sanitize_storage_key("businesses/0191/logo.jpg").is_ok());
    assert!(sanitize_storage_key("assets/cover-image_123.webp").is_ok());

    // Path traversal attacks must be strictly rejected
    assert!(sanitize_storage_key("../../../etc/passwd").is_err());
    assert!(sanitize_storage_key("..\\..\\windows\\system32").is_err());
    assert!(sanitize_storage_key("/root/.ssh/id_rsa").is_err());
    assert!(sanitize_storage_key("businesses/test\0.jpg").is_err());
}