use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::time::Duration;

use super::models::{
    AllowlistEntry, Ban, Category, Channel, ChannelPermissionOverride, Invite, Member, MemberRole,
    Message, Role, Session, User,
};
use super::StorageError;

#[async_trait]
pub trait UserStore: Send + Sync {
    async fn create_user(
        &self,
        pubkey: &str,
        display_name: Option<&str>,
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
        category_id: Option<&str>,
        position: i32,
    ) -> Result<Channel, StorageError>;
    async fn get_channel(&self, id: &str) -> Result<Option<Channel>, StorageError>;
    async fn list_channels(&self) -> Result<Vec<Channel>, StorageError>;
    /// Update a channel. For `category_id`:
    /// - `None` = leave unchanged
    /// - `Some(None)` = set to uncategorized
    /// - `Some(Some(id))` = assign to category
    async fn update_channel(
        &self,
        id: &str,
        name: Option<&str>,
        topic: Option<&str>,
        category_id: Option<Option<&str>>,
        position: Option<i32>,
    ) -> Result<Channel, StorageError>;
    async fn delete_channel(&self, id: &str) -> Result<(), StorageError>;
}

#[async_trait]
pub trait CategoryStore: Send + Sync {
    async fn create_category(&self, name: &str, position: i32) -> Result<Category, StorageError>;
    async fn get_category(&self, id: &str) -> Result<Option<Category>, StorageError>;
    async fn list_categories(&self) -> Result<Vec<Category>, StorageError>;
    async fn update_category(
        &self,
        id: &str,
        name: Option<&str>,
        position: Option<i32>,
    ) -> Result<Category, StorageError>;
    async fn delete_category(&self, id: &str) -> Result<(), StorageError>;
}

#[async_trait]
pub trait MessageStore: Send + Sync {
    async fn create_message(
        &self,
        channel_id: &str,
        author_id: &str,
        content: &str,
    ) -> Result<Message, StorageError>;
    async fn get_message(&self, id: &str) -> Result<Option<Message>, StorageError>;
    async fn list_messages(
        &self,
        channel_id: &str,
        before: Option<&str>,
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

pub trait Storage:
    UserStore
    + ChannelStore
    + CategoryStore
    + MessageStore
    + SessionStore
    + RoleStore
    + InviteStore
    + MemberStore
    + MediaStore
{
}

impl<T> Storage for T where
    T: UserStore
        + ChannelStore
        + CategoryStore
        + MessageStore
        + SessionStore
        + RoleStore
        + InviteStore
        + MemberStore
        + MediaStore
{
}
