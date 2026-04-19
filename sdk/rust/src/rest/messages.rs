use reqwest::Method;

use crate::error::Result;
use crate::rest::models::{
    CreateMessageRequest, ListMessagesQuery, Message, MessagePage, UpdateMessageRequest,
};
use crate::rest::Client;

pub struct MessagesApi<'c> {
    client: &'c Client,
}

impl<'c> MessagesApi<'c> {
    pub(crate) fn new(client: &'c Client) -> Self {
        Self { client }
    }

    /// Send a message to a channel. Pass `attachment_ids` inside
    /// [`CreateMessageRequest`] if the message carries attachments.
    pub async fn send(&self, channel_id: &str, req: &CreateMessageRequest) -> Result<Message> {
        self.client
            .post(&format!("/api/v1/channels/{channel_id}/messages"), req)
            .await
    }

    /// Convenience: send a plain-text message with no attachments.
    pub async fn send_text(&self, channel_id: &str, content: impl Into<String>) -> Result<Message> {
        let req = CreateMessageRequest {
            content: content.into(),
            attachment_ids: Vec::new(),
        };
        self.send(channel_id, &req).await
    }

    pub async fn list(&self, channel_id: &str, query: &ListMessagesQuery) -> Result<MessagePage> {
        let path = format!("/api/v1/channels/{channel_id}/messages");
        let req = self.client.request(Method::GET, &path)?.query(query);
        let resp = req.send().await?;
        super::decode_response(resp).await
    }

    pub async fn get(&self, channel_id: &str, message_id: &str) -> Result<Message> {
        self.client
            .get(&format!(
                "/api/v1/channels/{channel_id}/messages/{message_id}"
            ))
            .await
    }

    pub async fn update(
        &self,
        channel_id: &str,
        message_id: &str,
        req: &UpdateMessageRequest,
    ) -> Result<Message> {
        self.client
            .patch(
                &format!("/api/v1/channels/{channel_id}/messages/{message_id}"),
                req,
            )
            .await
    }

    pub async fn delete(&self, channel_id: &str, message_id: &str) -> Result<()> {
        self.client
            .delete(&format!(
                "/api/v1/channels/{channel_id}/messages/{message_id}"
            ))
            .await
    }
}
