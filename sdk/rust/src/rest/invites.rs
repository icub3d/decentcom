use crate::error::Result;
use crate::rest::models::{
    CreateInviteRequest, Invite, InvitePreview, JoinInviteResponse, JoinedMember,
    ListInvitesResponse,
};
use crate::rest::Client;

pub struct InvitesApi<'c> {
    client: &'c Client,
}

impl<'c> InvitesApi<'c> {
    pub(crate) fn new(client: &'c Client) -> Self {
        Self { client }
    }

    pub async fn create(&self, req: &CreateInviteRequest) -> Result<Invite> {
        self.client.post("/api/v1/invites", req).await
    }

    pub async fn list(&self) -> Result<Vec<Invite>> {
        let resp: ListInvitesResponse = self.client.get("/api/v1/invites").await?;
        Ok(resp.invites)
    }

    /// Invite previews are public — no session required.
    pub async fn preview(&self, code: &str) -> Result<InvitePreview> {
        let resp = self
            .client
            .request_anonymous(reqwest::Method::GET, &format!("/api/v1/invites/{code}"))
            .send()
            .await?;
        super::decode_response(resp).await
    }

    pub async fn revoke(&self, code: &str) -> Result<()> {
        self.client
            .delete(&format!("/api/v1/invites/{code}"))
            .await
    }

    pub async fn join(&self, code: &str) -> Result<JoinedMember> {
        let resp: JoinInviteResponse = self
            .client
            .post_empty(&format!("/api/v1/invites/{code}/join"))
            .await?;
        Ok(resp.member)
    }
}
