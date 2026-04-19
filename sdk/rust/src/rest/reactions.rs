use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::rest::Client;

#[derive(Debug, Clone, Default, Serialize)]
pub struct ListReactorsQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReactorEntry {
    pub id: String,
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListReactorsResponse {
    pub users: Vec<ReactorEntry>,
    pub total: usize,
}

pub struct ReactionsApi<'c> {
    client: &'c Client,
}

impl<'c> ReactionsApi<'c> {
    pub(crate) fn new(client: &'c Client) -> Self {
        Self { client }
    }

    /// PUT — add or toggle a reaction with the given emoji.
    pub async fn add(
        &self,
        channel_id: &str,
        message_id: &str,
        emoji: &str,
    ) -> Result<()> {
        self.client
            .put_empty(&format!(
                "/api/v1/channels/{channel_id}/messages/{message_id}/reactions/{emoji}"
            ))
            .await
    }

    /// DELETE — remove the authenticated user's reaction on this message.
    pub async fn remove_own(
        &self,
        channel_id: &str,
        message_id: &str,
    ) -> Result<()> {
        self.client
            .delete(&format!(
                "/api/v1/channels/{channel_id}/messages/{message_id}/reactions"
            ))
            .await
    }

    /// DELETE — remove another user's reaction (requires `MANAGE_MESSAGES`).
    pub async fn remove_user(
        &self,
        channel_id: &str,
        message_id: &str,
        user_id: &str,
    ) -> Result<()> {
        self.client
            .delete(&format!(
                "/api/v1/channels/{channel_id}/messages/{message_id}/reactions/users/{user_id}"
            ))
            .await
    }

    /// GET — list users who reacted with the given emoji.
    pub async fn list_reactors(
        &self,
        channel_id: &str,
        message_id: &str,
        emoji: &str,
        query: &ListReactorsQuery,
    ) -> Result<ListReactorsResponse> {
        let path = format!(
            "/api/v1/channels/{channel_id}/messages/{message_id}/reactions/{emoji}"
        );
        let req = self.client.request(Method::GET, &path)?.query(query);
        let resp = req.send().await?;
        super::decode_response(resp).await
    }
}
