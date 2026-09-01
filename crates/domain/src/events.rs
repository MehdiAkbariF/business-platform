use serde::Serialize;
use shared::{SessionId, UserId};

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum SecurityEvent {
    UserRegistered { user_id: UserId, email: String },
    UserLoggedIn { user_id: UserId, session_id: SessionId },
    LoginFailed { email: String, reason: String },
    SessionCreated { session_id: SessionId, user_id: UserId },
    SessionRevoked { session_id: SessionId, user_id: UserId },
    RefreshTokenRotated { session_id: SessionId, user_id: UserId },
    RefreshTokenReuseDetected { session_id: SessionId, user_id: UserId },
    Logout { session_id: SessionId, user_id: UserId },
    LogoutAll { user_id: UserId },
}