use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateInviteRequest {
    pub max_uses: Option<i64>,
    pub expires_in_seconds: Option<i64>,
    pub grant_role_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InviteResponse {
    pub code: String,
    pub created_by: String,
    pub grant_role_id: Option<String>,
    pub max_uses: i64,
    pub use_count: i64,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub invite_link: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListInvitesResponse {
    pub invites: Vec<InviteResponse>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InvitePreviewResponse {
    pub code: String,
    pub server_name: String,
    pub server_icon: Option<String>,
    pub member_count: usize,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinedMember {
    pub pubkey: String,
    pub roles: Vec<String>,
    pub joined_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinInviteResponse {
    pub member: JoinedMember,
}

impl InviteResponse {
    pub fn from_invite(invite: crate::storage::models::Invite, invite_link: String) -> Self {
        Self {
            code: invite.code,
            created_by: invite.created_by,
            grant_role_id: invite.grant_role_id,
            max_uses: invite.max_uses,
            use_count: invite.use_count,
            expires_at: invite.expires_at,
            created_at: invite.created_at,
            invite_link,
        }
    }
}
