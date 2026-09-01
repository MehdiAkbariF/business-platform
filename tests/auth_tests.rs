use application::ports::security::{PasswordHasherPort, TokenServicePort};
use infrastructure::security::{argon2_hasher::Argon2PasswordHasher, jwt_token_service::JwtTokenService};
use domain::user::GlobalRole;
use shared::{SessionId, UserId};

#[test]
fn test_argon2id_password_hashing_and_verification() {
    let hasher = Argon2PasswordHasher;
    let password = "SuperSecretPassword123!";
    let hash = hasher.hash_password(password).expect("Hashing failed");

    assert_ne!(password, hash);
    assert!(hasher.verify_password(password, &hash).unwrap());
    assert!(!hasher.verify_password("WrongPassword123!", &hash).unwrap());
}

#[test]
fn test_jwt_access_token_generation_and_validation() {
    let token_service = JwtTokenService::new("test_jwt_secret_key_at_least_32_bytes_long".to_string(), 900);
    let user_id = UserId::new();
    let session_id = SessionId::new();

    let claims = application::ports::security::AccessTokenClaims {
        user_id,
        session_id,
        role: GlobalRole::User,
    };

    let token = token_service.generate_access_token(claims).expect("Token creation failed");
    let verified = token_service.verify_access_token(&token).expect("Token verification failed");

    assert_eq!(verified.user_id, user_id);
    assert_eq!(verified.session_id, session_id);
    assert_eq!(verified.role, GlobalRole::User);
}

#[test]
fn test_refresh_token_hashing_is_deterministic() {
    let token_service = JwtTokenService::new("test_secret".to_string(), 900);
    let token = token_service.generate_refresh_token();
    let hash1 = token_service.hash_refresh_token(&token);
    let hash2 = token_service.hash_refresh_token(&token);

    assert_eq!(hash1, hash2);
    assert_ne!(token, hash1);
}