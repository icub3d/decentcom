use crate::error::Result;
use crate::rest::models::{
    AllowlistEntry, AllowlistRequest, ApproveBotRequest, Ban, BanRequest, BotApproval,
    JoinMemberResponse, ListAllowlistResponse, ListBansResponse, ListMembersResponse,
    ListPendingBotsResponse, Member,
};
use crate::rest::Client;

pub struct MembersApi<'c> {
    client: &'c Client,
}

impl<'c> MembersApi<'c> {
    pub(crate) fn new(client: &'c Client) -> Self {
        Self { client }
    }

    pub async fn list(&self) -> Result<Vec<Member>> {
        let resp: ListMembersResponse = self.client.get("/api/v1/members").await?;
        Ok(resp.members)
    }

    pub async fn get(&self, pubkey: &str) -> Result<Member> {
        self.client
            .get(&format!("/api/v1/members/{pubkey}"))
            .await
    }

    /// POST `/api/v1/members/join` — join the server as the authenticated user.
    pub async fn join(&self) -> Result<Member> {
        let resp: JoinMemberResponse = self.client.post_empty("/api/v1/members/join").await?;
        Ok(resp.member)
    }

    pub async fn leave(&self) -> Result<()> {
        self.client.delete("/api/v1/members/me").await
    }

    pub async fn kick(&self, pubkey: &str) -> Result<()> {
        self.client
            .send_empty(
                reqwest::Method::POST,
                &format!("/api/v1/members/{pubkey}/kick"),
            )
            .await
    }

    pub async fn ban(&self, pubkey: &str, req: &BanRequest) -> Result<Ban> {
        self.client
            .post(&format!("/api/v1/members/{pubkey}/ban"), req)
            .await
    }

    pub async fn list_bans(&self) -> Result<Vec<Ban>> {
        let resp: ListBansResponse = self.client.get("/api/v1/bans").await?;
        Ok(resp.bans)
    }

    pub async fn unban(&self, pubkey: &str) -> Result<()> {
        self.client
            .delete(&format!("/api/v1/bans/{pubkey}"))
            .await
    }

    pub async fn list_allowlist(&self) -> Result<Vec<AllowlistEntry>> {
        let resp: ListAllowlistResponse = self.client.get("/api/v1/allowlist").await?;
        Ok(resp.entries)
    }

    pub async fn add_allowlist(&self, pubkey: impl Into<String>) -> Result<AllowlistEntry> {
        let body = AllowlistRequest { pubkey: pubkey.into() };
        self.client.post("/api/v1/allowlist", &body).await
    }

    pub async fn remove_allowlist(&self, pubkey: &str) -> Result<()> {
        self.client
            .delete(&format!("/api/v1/allowlist/{pubkey}"))
            .await
    }

    pub async fn list_pending_bots(&self) -> Result<Vec<BotApproval>> {
        let resp: ListPendingBotsResponse = self.client.get("/api/v1/admin/bots/pending").await?;
        Ok(resp.bots)
    }

    pub async fn approve_bot(
        &self,
        pubkey: &str,
        req: &ApproveBotRequest,
    ) -> Result<BotApproval> {
        self.client
            .post(&format!("/api/v1/admin/bots/{pubkey}/approve"), req)
            .await
    }

    pub async fn revoke_bot(&self, pubkey: &str) -> Result<BotApproval> {
        self.client
            .post_empty(&format!("/api/v1/admin/bots/{pubkey}/revoke"))
            .await
    }
}
