use serde::{Deserialize, Serialize};
use std::fmt;
use utoipa::ToSchema;
use uuid::Uuid;

macro_rules! define_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
        #[serde(transparent)]
        pub struct $name(pub Uuid);

        impl $name {
            pub fn new() -> Self { Self(Uuid::now_v7()) }
            pub fn from_uuid(uuid: Uuid) -> Self { Self(uuid) }
        }

        impl Default for $name {
            fn default() -> Self { Self::new() }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

// 1. Identity & Auth (Phase 12)
define_id!(UserId);
define_id!(SessionId);
define_id!(TokenFamilyId);
define_id!(AuditLogId);

// 2. Business & Ownership (Phase 13)
define_id!(BusinessId);
define_id!(MembershipId);
define_id!(LocationId);

// 3. Taxonomy (Phase 14)
define_id!(CategoryId);
define_id!(ServiceId);
define_id!(AliasId);

// 4. Media & Profile (Phase 15)
define_id!(MediaId);
define_id!(AttributeId);
define_id!(SocialLinkId);

// 5. Moderation & Trust (Phase 16)
define_id!(CaseId);
define_id!(DecisionId);
define_id!(ClaimId);
define_id!(ReportId);

// 6. Monetization & Ads (Phase 20)
define_id!(PlanId);
define_id!(SubscriptionId);
define_id!(PaymentId);
define_id!(TransactionId);
define_id!(CampaignId);
define_id!(CreativeId);

// 7. SEO & Growth (Phase 21)
define_id!(RedirectId);
define_id!(LandingPageId);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientMetadata {
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}