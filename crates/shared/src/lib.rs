use serde::{Deserialize, Serialize};
use std::fmt;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(transparent)]
pub struct UserId(pub Uuid);

impl UserId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
}
impl Default for UserId {
    fn default() -> Self {
        Self::new()
    }
}
impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(transparent)]
pub struct BusinessId(pub Uuid);

impl BusinessId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
}
impl Default for BusinessId {
    fn default() -> Self {
        Self::new()
    }
}
impl fmt::Display for BusinessId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(transparent)]
pub struct MediaId(pub Uuid);

impl MediaId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
}
impl Default for MediaId {
    fn default() -> Self {
        Self::new()
    }
}
impl fmt::Display for MediaId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(transparent)]
pub struct AttributeId(pub Uuid);

impl AttributeId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
}
impl Default for AttributeId {
    fn default() -> Self {
        Self::new()
    }
}
impl fmt::Display for AttributeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(transparent)]
pub struct SocialLinkId(pub Uuid);

impl SocialLinkId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
}
impl Default for SocialLinkId {
    fn default() -> Self {
        Self::new()
    }
}
impl fmt::Display for SocialLinkId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(transparent)]
pub struct CategoryId(pub Uuid);

impl CategoryId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
}
impl Default for CategoryId {
    fn default() -> Self {
        Self::new()
    }
}
impl fmt::Display for CategoryId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(transparent)]
pub struct ServiceId(pub Uuid);

impl ServiceId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
}
impl Default for ServiceId {
    fn default() -> Self {
        Self::new()
    }
}
impl fmt::Display for ServiceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(transparent)]
pub struct AliasId(pub Uuid);

impl AliasId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
}
impl Default for AliasId {
    fn default() -> Self {
        Self::new()
    }
}
impl fmt::Display for AliasId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(transparent)]
pub struct MembershipId(pub Uuid);

impl MembershipId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
}
impl Default for MembershipId {
    fn default() -> Self {
        Self::new()
    }
}
impl fmt::Display for MembershipId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(transparent)]
pub struct LocationId(pub Uuid);

impl LocationId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
}
impl Default for LocationId {
    fn default() -> Self {
        Self::new()
    }
}
impl fmt::Display for LocationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(transparent)]
pub struct SessionId(pub Uuid);

impl SessionId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
}
impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}
impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TokenFamilyId(pub Uuid);

impl TokenFamilyId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
}
impl Default for TokenFamilyId {
    fn default() -> Self {
        Self::new()
    }
}
impl fmt::Display for TokenFamilyId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AuditLogId(pub Uuid);

impl AuditLogId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}
impl Default for AuditLogId {
    fn default() -> Self {
        Self::new()
    }
}
impl fmt::Display for AuditLogId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientMetadata {
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}