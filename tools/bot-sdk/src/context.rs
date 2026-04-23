use std::sync::Arc;

use decentcom_sdk::rest::{
    models::{Ban, BanRequest, Message, UpdateMessageRequest},
    Client,
};

use crate::error::Result;
use crate::events::MessageEvent;

/// Handle passed to every event handler that provides REST action helpers.
///
/// Cloning is cheap — the inner client is `Arc`-backed.
#[derive(Clone)]
pub struct Context {
    pub(crate) client: Arc<Client>,
    pub(crate) bot_user_id: String,
}

impl Context {
    pub(crate) fn new(client: Client) -> Self {
        let bot_user_id = client.user_id().unwrap_or_default();
        Self {
            client: Arc::new(client),
            bot_user_id,
        }
    }

    // ── Message actions ────────────────────────────────────────────────────

    /// Send a plain-text message to a channel.
    pub async fn send(&self, channel_id: &str, content: impl Into<String>) -> Result<Message> {
        Ok(self.client.messages().send_text(channel_id, content).await?)
    }

    /// Reply to a specific message by sending in the same channel.
    pub async fn reply(&self, evt: &MessageEvent, content: impl Into<String>) -> Result<Message> {
        self.send(&evt.channel_id, content).await
    }

    /// Edit a message. Only messages authored by the bot may be edited.
    pub async fn edit_message(
        &self,
        channel_id: &str,
        message_id: &str,
        content: impl Into<String>,
    ) -> Result<Message> {
        Ok(self
            .client
            .messages()
            .update(
                channel_id,
                message_id,
                &UpdateMessageRequest { content: content.into() },
            )
            .await?)
    }

    /// Delete a message by ID.
    pub async fn delete_message(&self, channel_id: &str, message_id: &str) -> Result<()> {
        Ok(self.client.messages().delete(channel_id, message_id).await?)
    }

    // ── Moderation actions ─────────────────────────────────────────────────

    /// Kick a member (by pubkey) from the server.
    pub async fn kick(&self, pubkey: &str) -> Result<()> {
        Ok(self.client.members().kick(pubkey).await?)
    }

    /// Permanently ban a member (by pubkey) with an optional reason.
    pub async fn ban(&self, pubkey: &str, reason: Option<String>) -> Result<Ban> {
        Ok(self.client.members().ban(pubkey, &BanRequest { reason }).await?)
    }

    // ── Accessors ──────────────────────────────────────────────────────────

    /// The user ID the bot is authenticated as.
    pub fn bot_user_id(&self) -> &str {
        &self.bot_user_id
    }

    /// Access the underlying SDK client for operations not covered by the
    /// convenience helpers above.
    pub fn sdk(&self) -> &Client {
        &self.client
    }
}
