use reqwest::Method;

use crate::error::Result;
use crate::rest::models::{
    CreateMessageRequest, CreateThreadResponse, ListMessagesQuery, Message, MessagePage, Thread,
};
use crate::rest::Client;

pub struct ThreadsApi<'c> {
    client: &'c Client,
}

impl<'c> ThreadsApi<'c> {
    pub(crate) fn new(client: &'c Client) -> Self {
        Self { client }
    }

    pub async fn create(
        &self,
        channel_id: &str,
        message_id: &str,
    ) -> Result<CreateThreadResponse> {
        self.client
            .post_empty(&format!(
                "/api/v1/channels/{channel_id}/messages/{message_id}/threads"
            ))
            .await
    }

    pub async fn get(&self, thread_id: &str) -> Result<Thread> {
        self.client
            .get(&format!("/api/v1/threads/{thread_id}"))
            .await
    }

    pub async fn list_messages(
        &self,
        thread_id: &str,
        query: &ListMessagesQuery,
    ) -> Result<MessagePage> {
        let path = format!("/api/v1/threads/{thread_id}/messages");
        let req = self.client.request(Method::GET, &path)?.query(query);
        let resp = req.send().await?;
        super::decode_response(resp).await
    }

    pub async fn post_message(
        &self,
        thread_id: &str,
        req: &CreateMessageRequest,
    ) -> Result<Message> {
        self.client
            .post(&format!("/api/v1/threads/{thread_id}/messages"), req)
            .await
    }

    pub async fn follow(&self, thread_id: &str) -> Result<()> {
        self.client
            .put_empty(&format!("/api/v1/threads/{thread_id}/follow"))
            .await
    }

    pub async fn unfollow(&self, thread_id: &str) -> Result<()> {
        self.client
            .delete(&format!("/api/v1/threads/{thread_id}/follow"))
            .await
    }

    pub async fn mark_read(&self, thread_id: &str) -> Result<()> {
        self.client
            .put_empty(&format!("/api/v1/threads/{thread_id}/read"))
            .await
    }
}
