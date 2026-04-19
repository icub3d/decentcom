//! Wire types for REST request and response bodies.
//!
//! These mirror the server-side types in `server/src/**/models.rs` without
//! depending on the server crate. Field names and semantics must stay in
//! lockstep with the server.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ── Channels ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct CreateChannelRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<i32>,
}

/// PATCH body for a channel. Use `None` to leave a field alone; use
/// `Some(None)` on `category` to clear it explicitly (matches the server's
/// three-valued semantics).
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateChannelRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub id: String,
    pub name: String,
    pub topic: Option<String>,
    pub category: Option<String>,
    pub position: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(rename = "type")]
    pub channel_type: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListChannelsResponse {
    pub channels: Vec<Channel>,
}

// ── Messages ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct CreateMessageRequest {
    pub content: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attachment_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateMessageRequest {
    pub content: String,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ListMessagesQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReactionSummary {
    pub emoji: String,
    pub count: i64,
    pub me: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadSummary {
    pub thread_id: String,
    pub reply_count: i64,
    pub last_reply_at: Option<DateTime<Utc>>,
    pub last_reply_author_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub channel_id: String,
    pub author_id: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub edited_at: Option<DateTime<Utc>>,
    pub deleted: bool,
    pub attachments: Vec<Attachment>,
    pub reactions: Vec<ReactionSummary>,
    pub thread: Option<ThreadSummary>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MessagePage {
    pub messages: Vec<Message>,
    pub has_more: bool,
}

// ── Members ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct RoleRef {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
    pub position: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Member {
    pub user_id: String,
    pub pubkey: String,
    pub display_name: Option<String>,
    pub avatar_hash: Option<String>,
    pub is_bot: bool,
    pub joined_at: DateTime<Utc>,
    pub roles: Vec<RoleRef>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListMembersResponse {
    pub members: Vec<Member>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JoinMemberResponse {
    pub member: Member,
}

#[derive(Debug, Clone, Serialize)]
pub struct BanRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Ban {
    pub pubkey: String,
    pub banned_by: String,
    pub reason: Option<String>,
    pub banned_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListBansResponse {
    pub bans: Vec<Ban>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AllowlistRequest {
    pub pubkey: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AllowlistEntry {
    pub pubkey: String,
    pub added_by: String,
    pub added_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListAllowlistResponse {
    pub entries: Vec<AllowlistEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BotApproval {
    pub pubkey: String,
    pub approved_by: Option<String>,
    pub approved_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListPendingBotsResponse {
    pub bots: Vec<BotApproval>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ApproveBotRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

// ── Roles ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct CreateRoleRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    pub permissions: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<i32>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateRoleRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
    pub permissions: i64,
    pub position: i32,
    pub is_builtin: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListRolesResponse {
    pub roles: Vec<Role>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SetChannelOverrideRequest {
    pub allow: i64,
    pub deny: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelOverride {
    pub channel_id: String,
    pub role_id: String,
    pub allow: i64,
    pub deny: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListChannelOverridesResponse {
    pub overrides: Vec<ChannelOverride>,
}

// ── Profiles ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct UpdateProfileRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Profile {
    pub user_id: String,
    pub pubkey: String,
    pub display_name: Option<String>,
    pub avatar_hash: Option<String>,
    pub joined_at: Option<String>,
    pub roles: Vec<RoleRef>,
}

// ── Invites ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateInviteRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_uses: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_in_seconds: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grant_role_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Invite {
    pub code: String,
    pub created_by: String,
    pub grant_role_id: Option<String>,
    pub max_uses: i64,
    pub use_count: i64,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub invite_link: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListInvitesResponse {
    pub invites: Vec<Invite>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InvitePreview {
    pub code: String,
    pub server_name: String,
    pub server_icon: Option<String>,
    pub member_count: usize,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JoinedMember {
    pub pubkey: String,
    pub roles: Vec<String>,
    pub joined_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JoinInviteResponse {
    pub member: JoinedMember,
}

// ── Threads ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct CreateThreadResponse {
    pub thread_id: String,
    pub parent_message_id: String,
    pub channel_id: String,
    pub created_at: DateTime<Utc>,
    pub reply_count: i64,
    pub follower_count: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Thread {
    pub id: String,
    pub channel_id: String,
    pub parent_message_id: String,
    pub creator_id: String,
    pub created_at: DateTime<Utc>,
    pub last_reply_at: Option<DateTime<Utc>>,
    pub reply_count: i64,
    pub follower_count: i64,
    pub is_following: bool,
}

// ── Attachments ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub id: String,
    pub message_id: Option<String>,
    pub channel_id: String,
    pub uploader_id: String,
    pub filename: String,
    pub content_hash: String,
    pub size: i64,
    pub mime_type: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub created_at: DateTime<Utc>,
}

// ── Server info ────────────────────────────────────────────────────────────

pub use shared::ServerInfo;
