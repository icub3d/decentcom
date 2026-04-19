use crate::error::Result;
use crate::rest::models::{
    Channel, CreateChannelRequest, ListChannelsResponse, UpdateChannelRequest,
};
use crate::rest::Client;

pub struct ChannelsApi<'c> {
    client: &'c Client,
}

impl<'c> ChannelsApi<'c> {
    pub(crate) fn new(client: &'c Client) -> Self {
        Self { client }
    }

    pub async fn list(&self) -> Result<Vec<Channel>> {
        let resp: ListChannelsResponse = self.client.get("/api/v1/channels").await?;
        Ok(resp.channels)
    }

    pub async fn create(&self, req: &CreateChannelRequest) -> Result<Channel> {
        self.client.post("/api/v1/channels", req).await
    }

    pub async fn get(&self, channel_id: &str) -> Result<Channel> {
        self.client
            .get(&format!("/api/v1/channels/{channel_id}"))
            .await
    }

    pub async fn update(&self, channel_id: &str, req: &UpdateChannelRequest) -> Result<Channel> {
        self.client
            .patch(&format!("/api/v1/channels/{channel_id}"), req)
            .await
    }

    pub async fn delete(&self, channel_id: &str) -> Result<()> {
        self.client
            .delete(&format!("/api/v1/channels/{channel_id}"))
            .await
    }
}
