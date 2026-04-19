//! REST API wrappers for the decentcom server.

pub mod attachments;
pub mod auth;
pub mod channels;
pub mod invites;
pub mod members;
pub mod messages;
pub mod models;
pub mod profiles;
pub mod reactions;
pub mod roles;
pub mod threads;

use std::sync::{Arc, RwLock};
use std::time::Duration;

use reqwest::{header, Method, RequestBuilder, StatusCode};
use serde::{de::DeserializeOwned, Serialize};
use url::Url;

use crate::error::{Error, Result};
use crate::identity::Identity;

/// Configuration for a [`Client`].
#[derive(Debug, Clone)]
pub struct Config {
    pub base_url: Url,
    pub timeout: Duration,
    pub user_agent: String,
}

impl Config {
    pub fn new(base_url: impl Into<String>) -> Result<Self> {
        let parsed = Url::parse(&base_url.into())?;
        Ok(Self {
            base_url: parsed,
            timeout: Duration::from_secs(30),
            user_agent: format!("decentcom-sdk/{}", env!("CARGO_PKG_VERSION")),
        })
    }
}

#[derive(Debug, Default)]
struct SessionState {
    token: Option<String>,
    user_id: Option<String>,
}

/// Typed, session-aware REST client for a single decentcom server.
///
/// Multiple `Client` instances can be constructed independently to manage
/// sessions with multiple servers in parallel.
#[derive(Debug, Clone)]
pub struct Client {
    http: reqwest::Client,
    config: Arc<Config>,
    session: Arc<RwLock<SessionState>>,
}

impl Client {
    pub fn new(base_url: impl Into<String>) -> Result<Self> {
        Self::with_config(Config::new(base_url)?)
    }

    pub fn with_config(config: Config) -> Result<Self> {
        let http = reqwest::Client::builder()
            .user_agent(&config.user_agent)
            .timeout(config.timeout)
            .build()?;
        Ok(Self {
            http,
            config: Arc::new(config),
            session: Arc::new(RwLock::new(SessionState::default())),
        })
    }

    pub fn base_url(&self) -> &Url {
        &self.config.base_url
    }

    /// Perform the two-step challenge-response auth flow and store the
    /// resulting session token on this client.
    pub async fn authenticate(&self, identity: &Identity) -> Result<shared::auth::VerifyResponse> {
        self.authenticate_as(identity, false).await
    }

    /// Authenticate as a bot (the server binds `"|bot"` into the challenge
    /// text, so signing the returned challenge produces a bot claim).
    pub async fn authenticate_as_bot(
        &self,
        identity: &Identity,
    ) -> Result<shared::auth::VerifyResponse> {
        self.authenticate_as(identity, true).await
    }

    async fn authenticate_as(
        &self,
        identity: &Identity,
        is_bot: bool,
    ) -> Result<shared::auth::VerifyResponse> {
        let challenge_resp = self
            .request_anonymous(Method::POST, "/api/v1/auth/challenge")
            .json(&shared::auth::ChallengeRequest {
                pubkey: identity.pubkey().to_owned(),
                is_bot,
            })
            .send()
            .await?;
        let challenge: shared::auth::ChallengeResponse = decode_response(challenge_resp).await?;

        let signature = identity.sign(challenge.challenge.as_bytes());
        let verify_resp = self
            .request_anonymous(Method::POST, "/api/v1/auth/verify")
            .json(&shared::auth::VerifyRequest {
                pubkey: identity.pubkey().to_owned(),
                signature,
            })
            .send()
            .await?;
        let verified: shared::auth::VerifyResponse = decode_response(verify_resp).await?;

        self.set_session(Some(verified.token.clone()), Some(verified.user_id.clone()));
        Ok(verified)
    }

    /// Replace the session manually (e.g. restoring from storage).
    pub fn set_session(&self, token: Option<String>, user_id: Option<String>) {
        let mut s = self.session.write().expect("sdk session rwlock poisoned");
        s.token = token;
        s.user_id = user_id;
    }

    pub fn session_token(&self) -> Option<String> {
        self.session
            .read()
            .expect("sdk session rwlock poisoned")
            .token
            .clone()
    }

    pub fn user_id(&self) -> Option<String> {
        self.session
            .read()
            .expect("sdk session rwlock poisoned")
            .user_id
            .clone()
    }

    // ── Typed module entry points ──────────────────────────────────────────

    pub fn auth(&self) -> auth::AuthApi<'_> {
        auth::AuthApi::new(self)
    }
    pub fn channels(&self) -> channels::ChannelsApi<'_> {
        channels::ChannelsApi::new(self)
    }
    pub fn messages(&self) -> messages::MessagesApi<'_> {
        messages::MessagesApi::new(self)
    }
    pub fn members(&self) -> members::MembersApi<'_> {
        members::MembersApi::new(self)
    }
    pub fn roles(&self) -> roles::RolesApi<'_> {
        roles::RolesApi::new(self)
    }
    pub fn profiles(&self) -> profiles::ProfilesApi<'_> {
        profiles::ProfilesApi::new(self)
    }
    pub fn invites(&self) -> invites::InvitesApi<'_> {
        invites::InvitesApi::new(self)
    }
    pub fn threads(&self) -> threads::ThreadsApi<'_> {
        threads::ThreadsApi::new(self)
    }
    pub fn reactions(&self) -> reactions::ReactionsApi<'_> {
        reactions::ReactionsApi::new(self)
    }
    pub fn attachments(&self) -> attachments::AttachmentsApi<'_> {
        attachments::AttachmentsApi::new(self)
    }

    /// Build the low-level gateway (WebSocket) client, using the current
    /// session token. Returns an error if not authenticated.
    pub fn gateway(&self) -> crate::gateway::GatewayBuilder {
        crate::gateway::GatewayBuilder::new(self.config.base_url.clone(), self.session_token())
    }

    // ── Internal request helpers ───────────────────────────────────────────

    pub(crate) fn request_anonymous(&self, method: Method, path: &str) -> RequestBuilder {
        let url = self
            .config
            .base_url
            .join(path.trim_start_matches('/'))
            .expect("valid path");
        self.http.request(method, url)
    }

    pub(crate) fn request(&self, method: Method, path: &str) -> Result<RequestBuilder> {
        let token = self.session_token().ok_or(Error::Unauthenticated)?;
        Ok(self
            .request_anonymous(method, path)
            .header(header::AUTHORIZATION, format!("Bearer {token}")))
    }

    pub(crate) async fn send_json<B: Serialize, T: DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
        body: Option<&B>,
    ) -> Result<T> {
        let mut req = self.request(method, path)?;
        if let Some(body) = body {
            req = req.json(body);
        }
        let resp = req.send().await?;
        decode_response(resp).await
    }

    pub(crate) async fn send_empty(&self, method: Method, path: &str) -> Result<()> {
        let resp = self.request(method, path)?.send().await?;
        check_status(resp).await?;
        Ok(())
    }

    pub(crate) async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        self.send_json::<(), T>(Method::GET, path, None).await
    }

    pub(crate) async fn post<B: Serialize, T: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        self.send_json(Method::POST, path, Some(body)).await
    }

    pub(crate) async fn post_empty<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        self.send_json::<(), T>(Method::POST, path, None).await
    }

    pub(crate) async fn patch<B: Serialize, T: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        self.send_json(Method::PATCH, path, Some(body)).await
    }

    pub(crate) async fn put<B: Serialize, T: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        self.send_json(Method::PUT, path, Some(body)).await
    }

    pub(crate) async fn put_empty(&self, path: &str) -> Result<()> {
        self.send_empty(Method::PUT, path).await
    }

    pub(crate) async fn delete(&self, path: &str) -> Result<()> {
        self.send_empty(Method::DELETE, path).await
    }
}

async fn check_status(resp: reqwest::Response) -> Result<reqwest::Response> {
    if resp.status().is_success() {
        return Ok(resp);
    }
    let status = resp.status().as_u16();
    let message = extract_error_message(resp).await;
    Err(Error::Api { status, message })
}

async fn decode_response<T: DeserializeOwned>(resp: reqwest::Response) -> Result<T> {
    let resp = check_status(resp).await?;
    if resp.status() == StatusCode::NO_CONTENT {
        // 204 responses carry no body; try to decode `null` or `{}` so callers
        // using `Unit`/empty-struct response types still work.
        let empty = serde_json::from_str::<T>("null")
            .or_else(|_| serde_json::from_str::<T>("{}"));
        return empty.map_err(Error::Json);
    }
    Ok(resp.json::<T>().await?)
}

async fn extract_error_message(resp: reqwest::Response) -> String {
    match resp.text().await {
        Ok(text) => {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
                if let Some(msg) = v.get("error").and_then(|e| e.as_str()) {
                    return msg.to_string();
                }
            }
            text
        }
        Err(e) => e.to_string(),
    }
}
