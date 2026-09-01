use domain::user::GlobalRole;
use shared::{SessionId, UserId};

#[derive(Debug, Clone)]
pub struct AuthenticatedPrincipal {
    pub user_id: UserId,
    pub session_id: SessionId,
    pub role: GlobalRole,
}