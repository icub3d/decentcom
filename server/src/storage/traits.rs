use async_trait::async_trait;
use std::time::Duration;

use super::models::{Channel, Message, Session, User};
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
    async fn update_channel(
        &self,
        id: &str,
        name: Option<&str>,
        topic: Option<&str>,
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
    async fn delete(&self, id: &str) -> Result<(), StorageError>;
}

pub trait Storage: UserStore + ChannelStore + MessageStore + SessionStore {}

impl<T> Storage for T where T: UserStore + ChannelStore + MessageStore + SessionStore {}
