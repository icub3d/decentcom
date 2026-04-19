use crate::error::Result;
use crate::rest::models::{Profile, UpdateProfileRequest};
use crate::rest::Client;

pub struct ProfilesApi<'c> {
    client: &'c Client,
}

impl<'c> ProfilesApi<'c> {
    pub(crate) fn new(client: &'c Client) -> Self {
        Self { client }
    }

    pub async fn update(&self, req: &UpdateProfileRequest) -> Result<Profile> {
        self.client.patch("/api/v1/profile", req).await
    }

    pub async fn delete_avatar(&self) -> Result<Profile> {
        // Handler returns JSON profile, so we can't use the empty-body helper.
        let req = self
            .client
            .request(reqwest::Method::DELETE, "/api/v1/profile/avatar")?;
        let resp = req.send().await?;
        super::decode_response(resp).await
    }

    pub async fn get_member_profile(&self, pubkey: &str) -> Result<Profile> {
        self.client
            .get(&format!("/api/v1/members/{pubkey}/profile"))
            .await
    }
}
