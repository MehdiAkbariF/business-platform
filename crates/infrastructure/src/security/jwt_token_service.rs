use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use application::errors::AppError;
use application::ports::security::{AccessTokenClaims, TokenServicePort};
use domain::user::GlobalRole;
use shared::{SessionId, UserId};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    session_id: String,
    role: String,
    iat: usize,
    exp: usize,
}

pub struct JwtTokenService {
    secret: String,
    ttl_seconds: i64,
}

impl JwtTokenService {
    pub fn new(secret: String, ttl_seconds: i64) -> Self {
        Self { secret, ttl_seconds }
    }
}

impl TokenServicePort for JwtTokenService {
    fn generate_access_token(&self, claims: AccessTokenClaims) -> Result<String, AppError> {
        let now = Utc::now().timestamp() as usize;
        let exp = now + self.ttl_seconds as usize;

        let payload = Claims {
            sub: claims.user_id.to_string(),
            session_id: claims.session_id.to_string(),
            role: claims.role.to_string(),
            iat: now,
            exp,
        };

        encode(
            &Header::default(),
            &payload,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(|e| AppError::internal(e))
    }

    fn verify_access_token(&self, token: &str) -> Result<AccessTokenClaims, AppError> {
        let decoded = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| AppError::Unauthorized("Invalid or expired access token".to_string()))?;

        let user_uuid = Uuid::parse_str(&decoded.claims.sub)
            .map_err(|_| AppError::Unauthorized("Malformed user identifier".to_string()))?;
        let session_uuid = Uuid::parse_str(&decoded.claims.session_id)
            .map_err(|_| AppError::Unauthorized("Malformed session identifier".to_string()))?;

        let role = match decoded.claims.role.as_str() {
            "USER" => GlobalRole::User,
            "MODERATOR" => GlobalRole::Moderator,
            "ADMIN" => GlobalRole::Admin,
            "SUPER_ADMIN" => GlobalRole::SuperAdmin,
            _ => return Err(AppError::Unauthorized("Invalid role claim".to_string())),
        };

        Ok(AccessTokenClaims {
            user_id: UserId::from_uuid(user_uuid),
            session_id: SessionId::from_uuid(session_uuid),
            role,
        })
    }

    fn generate_refresh_token(&self) -> String {
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(64)
            .map(char::from)
            .collect()
    }

    fn hash_refresh_token(&self, raw_token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(raw_token.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}