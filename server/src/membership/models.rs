use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::storage::models::{AllowlistEntry, Ban, BotApproval, Role};

#[derive(Debug, Clone, Deserialize)]
pub struct BanRequest {
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AllowlistRequest {
    pub pubkey: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct MemberWithRoles {
    pub user_id: String,
    pub pubkey: String,
    pub display_name: Option<String>,
    pub avatar_hash: Option<String>,
    pub is_bot: bool,
    pub joined_at: DateTime<Utc>,
    pub roles: Vec<RoleSummary>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RoleSummary {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
    pub position: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct JoinMemberResponse {
    pub member: MemberWithRoles,
}

#[derive(Debug, Clone, Serialize)]
pub struct ListMembersResponse {
    pub members: Vec<MemberWithRoles>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BanResponse {
    pub pubkey: String,
    pub banned_by: String,
    pub reason: Option<String>,
    pub banned_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ListBansResponse {
    pub bans: Vec<BanResponse>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AllowlistEntryResponse {
    pub pubkey: String,
    pub added_by: String,
    pub added_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ListAllowlistResponse {
    pub entries: Vec<AllowlistEntryResponse>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BotApprovalResponse {
    pub pubkey: String,
    pub approved_by: Option<String>,
    pub approved_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ListPendingBotsResponse {
    pub bots: Vec<BotApprovalResponse>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApproveBotRequest {
    pub note: Option<String>,
}

impl From<Role> for RoleSummary {
    fn from(value: Role) -> Self {
        Self {
            id: value.id,
            name: value.name,
            color: value.color,
            position: value.position,
        }
    }
}

impl From<Ban> for BanResponse {
    fn from(value: Ban) -> Self {
        Self {
            pubkey: value.pubkey,
            banned_by: value.banned_by,
            reason: value.reason,
            banned_at: value.banned_at,
        }
    }
}

impl From<AllowlistEntry> for AllowlistEntryResponse {
    fn from(value: AllowlistEntry) -> Self {
        Self {
            pubkey: value.pubkey,
            added_by: value.added_by,
            added_at: value.added_at,
        }
    }
}

impl From<BotApproval> for BotApprovalResponse {
    fn from(value: BotApproval) -> Self {
        Self {
            pubkey: value.pubkey,
            approved_by: value.approved_by,
            approved_at: value.approved_at,
            revoked_at: value.revoked_at,
            note: value.note,
        }
    }
}
