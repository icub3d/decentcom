use crate::error::Result;
use crate::rest::models::{
    ChannelOverride, CreateRoleRequest, ListChannelOverridesResponse, ListRolesResponse, Role,
    SetChannelOverrideRequest, UpdateRoleRequest,
};
use crate::rest::Client;

pub struct RolesApi<'c> {
    client: &'c Client,
}

impl<'c> RolesApi<'c> {
    pub(crate) fn new(client: &'c Client) -> Self {
        Self { client }
    }

    pub async fn list(&self) -> Result<Vec<Role>> {
        let resp: ListRolesResponse = self.client.get("/api/v1/roles").await?;
        Ok(resp.roles)
    }

    pub async fn create(&self, req: &CreateRoleRequest) -> Result<Role> {
        self.client.post("/api/v1/roles", req).await
    }

    pub async fn update(&self, role_id: &str, req: &UpdateRoleRequest) -> Result<Role> {
        self.client
            .patch(&format!("/api/v1/roles/{role_id}"), req)
            .await
    }

    pub async fn delete(&self, role_id: &str) -> Result<()> {
        self.client
            .delete(&format!("/api/v1/roles/{role_id}"))
            .await
    }

    pub async fn add_member_role(&self, pubkey: &str, role_id: &str) -> Result<()> {
        self.client
            .put_empty(&format!("/api/v1/members/{pubkey}/roles/{role_id}"))
            .await
    }

    pub async fn remove_member_role(&self, pubkey: &str, role_id: &str) -> Result<()> {
        self.client
            .delete(&format!("/api/v1/members/{pubkey}/roles/{role_id}"))
            .await
    }

    pub async fn list_channel_overrides(
        &self,
        channel_id: &str,
    ) -> Result<Vec<ChannelOverride>> {
        let resp: ListChannelOverridesResponse = self
            .client
            .get(&format!("/api/v1/channels/{channel_id}/overrides"))
            .await?;
        Ok(resp.overrides)
    }

    pub async fn set_channel_override(
        &self,
        channel_id: &str,
        role_id: &str,
        req: &SetChannelOverrideRequest,
    ) -> Result<ChannelOverride> {
        self.client
            .put(
                &format!("/api/v1/channels/{channel_id}/overrides/{role_id}"),
                req,
            )
            .await
    }

    pub async fn delete_channel_override(
        &self,
        channel_id: &str,
        role_id: &str,
    ) -> Result<()> {
        self.client
            .delete(&format!(
                "/api/v1/channels/{channel_id}/overrides/{role_id}"
            ))
            .await
    }
}
