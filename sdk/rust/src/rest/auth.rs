use crate::error::Result;
use crate::rest::Client;

/// `/api/v1/auth/*` endpoints that aren't covered by [`Client::authenticate`].
pub struct AuthApi<'c> {
    client: &'c Client,
}

impl<'c> AuthApi<'c> {
    pub(crate) fn new(client: &'c Client) -> Self {
        Self { client }
    }

    /// GET `/api/v1/auth/me` — the current session's user id.
    pub async fn me(&self) -> Result<shared::auth::AuthMeResponse> {
        self.client.get("/api/v1/auth/me").await
    }
}
