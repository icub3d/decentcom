use reqwest::Method;

use crate::error::{Error, Result};
use crate::rest::Client;

pub struct AttachmentsApi<'c> {
    client: &'c Client,
}

impl<'c> AttachmentsApi<'c> {
    pub(crate) fn new(client: &'c Client) -> Self {
        Self { client }
    }

    /// Fetch raw media bytes by content hash. Returns `(mime_type, bytes)`.
    ///
    /// The media endpoint is public — no session token required.
    pub async fn fetch_media(&self, content_hash: &str) -> Result<(String, Vec<u8>)> {
        let resp = self
            .client
            .request_anonymous(Method::GET, &format!("/api/v1/media/{content_hash}"))
            .send()
            .await?;
        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let message = match resp.text().await {
                Ok(t) => t,
                Err(e) => e.to_string(),
            };
            return Err(Error::Api { status, message });
        }
        let mime = resp
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("application/octet-stream")
            .to_string();
        let bytes = resp.bytes().await?.to_vec();
        Ok((mime, bytes))
    }

    /// Build an absolute URL for a media hash (useful for `<img src=...>` in
    /// downstream apps).
    pub fn media_url(&self, content_hash: &str) -> String {
        let base = self.client.base_url().as_str().trim_end_matches('/');
        format!("{base}/api/v1/media/{content_hash}")
    }
}
