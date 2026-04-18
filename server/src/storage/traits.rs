use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::time::Duration;

use super::models::{
    AllowlistEntry, Attachment, Ban, BotApproval, Channel, ChannelPermissionOverride, Invite,
    Member, MemberRole, Message, Reaction, ReactionCount, Role, Session, Thread, ThreadSummary,
    User,
};
use super::StorageError;

#[async_trait]
pub trait UserStore: Send + Sync {
    async fn create_user(
        &self,
        pubkey: &str,
        display_name: Option<&str>,
        is_bot: bool,
    ) -> Result<User, StorageError>;
    async fn get_user_by_id(&self, id: &str) -> Result<Option<User>, StorageError>;
    async fn get_user_by_pubkey(&self, pubkey: &str) -> Result<Option<User>, StorageError>;
    async fn update_user(
        &self,
        id: &str,
        display_name: Option<&str>,
        avatar_hash: Option<&str>,
    ) -> Result<User, StorageError>;
    async fn clear_avatar_hash(&self, id: &str) -> Result<User, StorageError>;
    async fn list_users(&self) -> Result<Vec<User>, StorageError>;
}

#[async_trait]
pub trait ChannelStore: Send + Sync {
    async fn create_channel(
        &self,
        name: &str,
        category: Option<&str>,
        position: i32,
    ) -> Result<Channel, StorageError>;
    async fn get_channel(&self, id: &str) -> Result<Option<Channel>, StorageError>;
    async fn list_channels(&self) -> Result<Vec<Channel>, StorageError>;
    /// Update a channel. For `category`:
    /// - `None` = leave unchanged
    /// - `Some(None)` = set to uncategorized
    /// - `Some(Some(name))` = assign to category by name
    async fn update_channel(
        &self,
        id: &str,
        name: Option<&str>,
        topic: Option<&str>,
        category: Option<Option<&str>>,
        position: Option<i32>,
    ) -> Result<Channel, StorageError>;
    async fn delete_channel(&self, id: &str) -> Result<(), StorageError>;
}

#[async_trait]
pub trait MessageStore: Send + Sync {
    async fn create_message(
        &self,
        channel_id: &str,
        author_id: &str,
        content: &str,
        thread_id: Option<&str>,
    ) -> Result<Message, StorageError>;
    async fn get_message(&self, id: &str) -> Result<Option<Message>, StorageError>;
    async fn list_messages(
        &self,
        channel_id: &str,
        before: Option<&str>,
        limit: u32,
    ) -> Result<Vec<Message>, StorageError>;
    async fn list_thread_messages(
        &self,
        thread_id: &str,
        before: Option<&str>,
        after: Option<&str>,
        limit: u32,
    ) -> Result<Vec<Message>, StorageError>;
    async fn update_message(&self, id: &str, content: &str) -> Result<Message, StorageError>;
    async fn delete_message(&self, id: &str) -> Result<(), StorageError>;
}

#[async_trait]
pub trait SessionStore: Send + Sync {
    async fn create_session(
        &self,
        user_id: &str,
        duration: Duration,
        is_read_only: bool,
    ) -> Result<Session, StorageError>;
    async fn get_session(&self, token: &str) -> Result<Option<Session>, StorageError>;
    async fn delete_session(&self, token: &str) -> Result<(), StorageError>;
    async fn delete_expired_sessions(&self) -> Result<u64, StorageError>;
}

#[async_trait]
pub trait RoleStore: Send + Sync {
    async fn create_role(
        &self,
        name: &str,
        color: Option<&str>,
        permissions: i64,
        position: i32,
        is_builtin: bool,
    ) -> Result<Role, StorageError>;
    async fn get_role(&self, id: &str) -> Result<Option<Role>, StorageError>;
    async fn list_roles(&self) -> Result<Vec<Role>, StorageError>;
    async fn update_role(
        &self,
        id: &str,
        name: Option<&str>,
        color: Option<Option<&str>>,
        permissions: Option<i64>,
        position: Option<i32>,
    ) -> Result<Role, StorageError>;
    async fn delete_role(&self, id: &str) -> Result<(), StorageError>;

    async fn add_member_role(
        &self,
        user_id: &str,
        role_id: &str,
    ) -> Result<MemberRole, StorageError>;
    async fn remove_member_role(&self, user_id: &str, role_id: &str) -> Result<(), StorageError>;
    async fn remove_all_member_roles(&self, user_id: &str) -> Result<(), StorageError>;
    async fn list_member_roles(&self, user_id: &str) -> Result<Vec<Role>, StorageError>;
    async fn list_role_members(&self, role_id: &str) -> Result<Vec<MemberRole>, StorageError>;
    async fn user_has_role(&self, user_id: &str, role_id: &str) -> Result<bool, StorageError>;

    async fn upsert_channel_permission_override(
        &self,
        channel_id: &str,
        role_id: &str,
        allow: i64,
        deny: i64,
    ) -> Result<ChannelPermissionOverride, StorageError>;
    async fn delete_channel_permission_override(
        &self,
        channel_id: &str,
        role_id: &str,
    ) -> Result<(), StorageError>;
    async fn list_channel_permission_overrides(
        &self,
        channel_id: &str,
    ) -> Result<Vec<ChannelPermissionOverride>, StorageError>;
}

#[async_trait]
pub trait InviteStore: Send + Sync {
    async fn create_invite(
        &self,
        code: &str,
        created_by: &str,
        grant_role_id: Option<&str>,
        max_uses: i64,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<Invite, StorageError>;
    async fn get_invite(&self, code: &str) -> Result<Option<Invite>, StorageError>;
    async fn list_active_invites(&self, now: DateTime<Utc>) -> Result<Vec<Invite>, StorageError>;
    async fn revoke_invite(&self, code: &str) -> Result<(), StorageError>;
    async fn consume_invite(&self, code: &str, now: DateTime<Utc>) -> Result<Invite, StorageError>;
    async fn delete_expired_invites(&self, now: DateTime<Utc>) -> Result<u64, StorageError>;
}

#[async_trait]
pub trait MemberStore: Send + Sync {
    async fn add_member(&self, user_id: &str) -> Result<Member, StorageError>;
    async fn remove_member(&self, user_id: &str) -> Result<(), StorageError>;
    async fn is_member(&self, user_id: &str) -> Result<bool, StorageError>;
    async fn list_members(&self) -> Result<Vec<Member>, StorageError>;
    async fn get_member_by_pubkey(&self, pubkey: &str) -> Result<Option<Member>, StorageError>;

    async fn add_ban(
        &self,
        pubkey: &str,
        banned_by: &str,
        reason: Option<&str>,
    ) -> Result<Ban, StorageError>;
    async fn remove_ban(&self, pubkey: &str) -> Result<(), StorageError>;
    async fn is_banned_pubkey(&self, pubkey: &str) -> Result<bool, StorageError>;
    async fn list_bans(&self) -> Result<Vec<Ban>, StorageError>;

    async fn add_allowlist_entry(
        &self,
        pubkey: &str,
        added_by: &str,
    ) -> Result<AllowlistEntry, StorageError>;
    async fn remove_allowlist_entry(&self, pubkey: &str) -> Result<(), StorageError>;
    async fn is_allowlisted_pubkey(&self, pubkey: &str) -> Result<bool, StorageError>;
    async fn list_allowlist_entries(&self) -> Result<Vec<AllowlistEntry>, StorageError>;
}

/// Media storage is defined here but implementation is deferred to the
/// file-uploads feature.
#[async_trait]
pub trait MediaStore: Send + Sync {
    async fn put(
        &self,
        content_hash: &str,
        mime_type: &str,
        size_bytes: u64,
        uploader_id: &str,
        bytes: &[u8],
    ) -> Result<String, StorageError>;
    async fn get(&self, id: &str) -> Result<Option<Vec<u8>>, StorageError>;
    /// Retrieve media bytes and MIME type by content hash.
    async fn get_by_content_hash(
        &self,
        content_hash: &str,
    ) -> Result<Option<(String, Vec<u8>)>, StorageError>;
    async fn delete(&self, id: &str) -> Result<(), StorageError>;
}

#[async_trait]
pub trait AttachmentStore: Send + Sync {
    async fn create_attachment(
        &self,
        params: CreateAttachmentParams<'_>,
    ) -> Result<Attachment, StorageError>;

    async fn get_attachment(&self, id: &str) -> Result<Option<Attachment>, StorageError>;

    async fn associate_attachments(
        &self,
        attachment_ids: &[String],
        message_id: &str,
        uploader_id: &str,
    ) -> Result<Vec<Attachment>, StorageError>;

    async fn list_attachments_for_message(
        &self,
        message_id: &str,
    ) -> Result<Vec<Attachment>, StorageError>;
}

#[async_trait]
pub trait ReactionStore: Send + Sync {
    /// Add or replace the current user's reaction.
    /// Returns `Some((reaction, replaced_emoji))` on add/replace, where `replaced_emoji` is
    /// `Some(old_emoji)` when a previous reaction was overwritten.
    /// Returns `None` when the same emoji was toggled off (removed).
    async fn upsert_reaction(
        &self,
        message_id: &str,
        user_id: &str,
        emoji: &str,
    ) -> Result<Option<(Reaction, Option<String>)>, StorageError>;

    async fn get_user_reaction(
        &self,
        message_id: &str,
        user_id: &str,
    ) -> Result<Option<Reaction>, StorageError>;

    async fn remove_reaction(&self, message_id: &str, user_id: &str) -> Result<(), StorageError>;

    /// Aggregate counts per emoji, with a `me` flag for `viewer_id`.
    async fn list_reaction_counts(
        &self,
        message_id: &str,
        viewer_id: &str,
    ) -> Result<Vec<ReactionCount>, StorageError>;

    /// List users who reacted with a specific emoji (paginated).
    async fn list_emoji_reactors(
        &self,
        message_id: &str,
        emoji: &str,
        limit: u32,
        before: Option<&str>,
    ) -> Result<Vec<Reaction>, StorageError>;
}

#[async_trait]
pub trait ThreadStore: Send + Sync {
    async fn create_thread(
        &self,
        channel_id: &str,
        parent_message_id: &str,
        creator_id: &str,
    ) -> Result<Thread, StorageError>;
    async fn get_thread(&self, id: &str) -> Result<Option<Thread>, StorageError>;
    async fn get_thread_by_parent(&self, parent_message_id: &str) -> Result<Option<Thread>, StorageError>;
    async fn get_thread_summary(&self, parent_message_id: &str) -> Result<Option<ThreadSummary>, StorageError>;
    async fn list_thread_summaries(&self, parent_message_ids: &[String]) -> Result<Vec<ThreadSummary>, StorageError>;

    async fn add_thread_follower(&self, thread_id: &str, user_id: &str) -> Result<(), StorageError>;
    async fn remove_thread_follower(&self, thread_id: &str, user_id: &str) -> Result<(), StorageError>;
    async fn is_thread_follower(&self, thread_id: &str, user_id: &str) -> Result<bool, StorageError>;
    async fn list_thread_followers(&self, thread_id: &str) -> Result<Vec<String>, StorageError>;
    async fn update_thread_last_read(&self, thread_id: &str, user_id: &str) -> Result<(), StorageError>;
}

#[async_trait]
pub trait BotStore: Send + Sync {
    async fn upsert_bot_pending(&self, pubkey: &str) -> Result<BotApproval, StorageError>;
    async fn approve_bot(
        &self,
        pubkey: &str,
        approved_by: &str,
        note: Option<&str>,
    ) -> Result<BotApproval, StorageError>;
    async fn revoke_bot(&self, pubkey: &str) -> Result<BotApproval, StorageError>;
    async fn get_bot_approval(&self, pubkey: &str) -> Result<Option<BotApproval>, StorageError>;
    async fn list_pending_bots(&self) -> Result<Vec<BotApproval>, StorageError>;
    async fn is_bot_approved(&self, pubkey: &str) -> Result<bool, StorageError>;
}

pub struct CreateAttachmentParams<'a> {
    pub channel_id: &'a str,
    pub uploader_id: &'a str,
    pub filename: &'a str,
    pub content_hash: &'a str,
    pub size: i64,
    pub mime_type: &'a str,
    pub width: Option<i32>,
    pub height: Option<i32>,
}

pub trait Storage:
    UserStore
    + ChannelStore
    + MessageStore
    + SessionStore
    + RoleStore
    + InviteStore
    + MemberStore
    + MediaStore
    + AttachmentStore
    + ReactionStore
    + ThreadStore
    + BotStore
{
}

impl<T> Storage for T where
    T: UserStore
        + ChannelStore
        + MessageStore
        + SessionStore
        + RoleStore
        + InviteStore
        + MemberStore
        + MediaStore
        + AttachmentStore
        + ReactionStore
        + ThreadStore
        + BotStore
{
}
