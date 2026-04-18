mod attachments;
mod auth;
mod channels;
mod config;
mod gateway;
mod invites;
mod membership;
mod messages;
mod permissions;
mod profiles;
mod reactions;
mod roles;
mod server_info;
mod storage;
mod threads;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use axum::http::{header, Method};
use axum::{extract::State, routing::get, Json, Router};
use clap::Parser;
use shared::HealthStatus;
use tower_http::cors::{Any, CorsLayer};
use tracing::{info, warn};

use crate::config::{ServerConfig, StorageBackendType};
use crate::storage::{DynStorage, SqliteStorage};

#[derive(Parser, Debug)]
#[command(name = "decentcom-server", version, about = "decentcom server")]
struct Cli {
    /// Path to configuration file.
    #[arg(short, long, default_value = "decentcom.toml")]
    config: PathBuf,
}

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<ServerConfig>,
    pub storage: DynStorage,
    pub challenge_store: auth::challenge::SharedChallengeStore,
    pub gateway: gateway::GatewayHandle,
}

pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .nest("/api/v1", channels::router())
        .nest("/api/v1", messages::router())
        .nest("/api/v1", invites::router())
        .nest("/api/v1", membership::router())
        .nest("/api/v1", roles::router())
        .nest("/api/v1", server_info::router())
        .nest("/api/v1", profiles::profile_router())
        .nest("/api/v1", attachments::router())
        .nest("/api/v1", reactions::router())
        .nest("/api/v1", threads::router())
        .nest("/api/v1/auth", auth::router())
        .nest("/api/v1/gateway", gateway::router())
        .layer(cors_layer())
        .with_state(state)
}

fn cors_layer() -> CorsLayer {
    // Tauri WebView requests originate from non-http origins; allow REST calls
    // in development until we add a stricter per-origin server config.
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION])
}

async fn health(State(_state): State<AppState>) -> Json<HealthStatus> {
    Json(HealthStatus::ok())
}

async fn init_storage(config: &ServerConfig) -> Result<DynStorage, Box<dyn std::error::Error>> {
    match config.storage.backend {
        StorageBackendType::Sqlite => {
            let path = config
                .storage
                .database_path
                .as_ref()
                .ok_or("storage.database_path is required for sqlite")?;
            
            // Create media directory if configured
            if let Some(ref media_path) = config.storage.media_path {
                tokio::fs::create_dir_all(media_path).await?;
            }

            let sqlite = SqliteStorage::open(path, config.storage.media_path.clone()).await?;
            Ok(Arc::new(sqlite))
        }
        StorageBackendType::Postgres => Err("postgres backend is not yet implemented".into()),
    }
}

fn spawn_session_cleanup(storage: DynStorage) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(300));
        interval.tick().await; // skip immediate first tick
        loop {
            interval.tick().await;
            match storage.delete_expired_sessions().await {
                Ok(n) if n > 0 => info!(removed = n, "expired sessions cleaned up"),
                Ok(_) => {}
                Err(e) => warn!(error = %e, "session cleanup failed"),
            }
        }
    });
}

fn spawn_challenge_cleanup(challenge_store: auth::challenge::SharedChallengeStore) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        interval.tick().await; // skip immediate first tick
        loop {
            interval.tick().await;
            let removed = challenge_store.purge_expired();
            if removed > 0 {
                info!(removed, "expired challenges cleaned up");
            }
        }
    });
}

fn spawn_invite_cleanup(storage: DynStorage) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60 * 60));
        interval.tick().await;
        loop {
            interval.tick().await;
            match storage.delete_expired_invites(chrono::Utc::now()).await {
                Ok(n) if n > 0 => info!(removed = n, "expired invites cleaned up"),
                Ok(_) => {}
                Err(e) => warn!(error = %e, "invite cleanup failed"),
            }
        }
    });
}

/// Seed initial data if the server is freshly initialized (no channels exist).
async fn seed_channels(storage: &DynStorage) -> Result<(), Box<dyn std::error::Error>> {
    let channels = storage.list_channels().await?;
    if channels.is_empty() {
        storage.create_channel("general", None, 0).await?;
        info!("seeded default 'general' channel");
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();
    let config = ServerConfig::load_or_default(&cli.config)?;
    info!(config = %config.summary(), "loaded configuration");

    let storage = init_storage(&config).await?;
    seed_channels(&storage).await?;
    let challenge_store = auth::challenge_store();
    spawn_session_cleanup(storage.clone());
    spawn_challenge_cleanup(challenge_store.clone());
    spawn_invite_cleanup(storage.clone());

    let bind = config.network.bind_address;
    let state = AppState {
        config: Arc::new(config),
        storage,
        challenge_store,
        gateway: gateway::gateway_handle(),
    };
    let listener = tokio::net::TcpListener::bind(bind).await?;
    info!(%bind, "listening");
    axum::serve(listener, app(state)).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use base64::engine::general_purpose::STANDARD as BASE64;
    use base64::Engine;
    use ed25519_dalek::Signer;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    async fn test_state() -> AppState {
        let storage: DynStorage = Arc::new(SqliteStorage::in_memory().await.unwrap());
        AppState {
            config: Arc::new(ServerConfig::default()),
            storage,
            challenge_store: auth::challenge_store(),
            gateway: gateway::gateway_handle(),
        }
    }

    /// Perform challenge-response auth. Returns the full VerifyResponse.
    /// Set `is_bot = true` to send the bot claim in the challenge request.
    async fn do_auth(state: &AppState, seed: u8, is_bot: bool) -> shared::auth::VerifyResponse {
        let signing_key = ed25519_dalek::SigningKey::from_bytes(&[seed; 32]);
        let pubkey = bs58::encode(signing_key.verifying_key().as_bytes()).into_string();

        let challenge_resp = app(state.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/auth/challenge")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({ "pubkey": pubkey, "is_bot": is_bot }).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        let challenge: shared::auth::ChallengeResponse =
            serde_json::from_slice(&challenge_resp.into_body().collect().await.unwrap().to_bytes())
                .unwrap();

        let signature = BASE64.encode(signing_key.sign(challenge.challenge.as_bytes()).to_bytes());
        let verify_resp = app(state.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/auth/verify")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({ "pubkey": pubkey, "signature": signature }).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        serde_json::from_slice(&verify_resp.into_body().collect().await.unwrap().to_bytes())
            .unwrap()
    }

    /// POST a request with a Bearer token and JSON body. Returns (status, body bytes).
    async fn post(
        state: &AppState,
        uri: &str,
        token: &str,
        body: &str,
    ) -> (StatusCode, Vec<u8>) {
        let resp = app(state.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(uri)
                    .header("authorization", format!("Bearer {token}"))
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = resp.status();
        let body = resp.into_body().collect().await.unwrap().to_bytes().to_vec();
        (status, body)
    }

    /// GET with a Bearer token. Returns (status, body bytes).
    async fn get(state: &AppState, uri: &str, token: &str) -> (StatusCode, Vec<u8>) {
        let resp = app(state.clone())
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(uri)
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = resp.status();
        let body = resp.into_body().collect().await.unwrap().to_bytes().to_vec();
        (status, body)
    }

    /// Full bot lifecycle:
    ///   1. Admin authenticates (first user → auto-granted admin).
    ///   2. Bot authenticates with is_bot=true → read-only session, queued in bot_approvals.
    ///   3. Bot joins the server (Open mode allows it).
    ///   4. Bot tries to send a message → blocked (read-only).
    ///   5. Admin lists pending bots → bot appears.
    ///   6. Admin approves the bot.
    ///   7. Bot re-authenticates → non-read-only session.
    ///   8. Bot sends a message → succeeds.
    ///   9. Verify DB state: approval recorded, message authored by bot.
    #[tokio::test]
    async fn bot_approval_flow() {
        let state = test_state().await;

        // Create a channel for message tests.
        let channel_id = state.storage.create_channel("general", None, 0).await.unwrap().id;

        // Step 1: Admin authenticates. First user is auto-granted admin in Open mode.
        let admin = do_auth(&state, 0x01, false).await;
        assert!(!admin.is_read_only, "admin should not be read-only");

        // Step 2: Bot authenticates with is_bot=true.
        let bot_pubkey = {
            let sk = ed25519_dalek::SigningKey::from_bytes(&[0x10u8; 32]);
            bs58::encode(sk.verifying_key().as_bytes()).into_string()
        };
        let bot_auth1 = do_auth(&state, 0x10, true).await;
        assert!(bot_auth1.is_read_only, "unapproved bot must receive a read-only session");

        // bot_approvals should have a pending row with no approved_at.
        let pending = state.storage.list_pending_bots().await.unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].pubkey, bot_pubkey);
        assert!(pending[0].approved_at.is_none(), "should still be pending");

        // Step 3: Bot joins the server (Open mode).
        let (join_status, join_body) = post(&state, "/api/v1/members/join", &bot_auth1.token, "").await;
        assert_eq!(
            join_status,
            StatusCode::OK,
            "bot should be able to join an open server: {}",
            String::from_utf8_lossy(&join_body)
        );

        // Step 4: Bot tries to send a message — must be blocked.
        let (send_status, send_body) = post(
            &state,
            &format!("/api/v1/channels/{channel_id}/messages"),
            &bot_auth1.token,
            r#"{"content":"hello from unapproved bot"}"#,
        )
        .await;
        assert_eq!(send_status, StatusCode::FORBIDDEN);
        let body_str = String::from_utf8_lossy(&send_body);
        assert!(
            body_str.contains("read-only"),
            "error should mention read-only: {body_str}"
        );

        // Step 5: Admin lists pending bots — bot should appear.
        let (list_status, list_body) = get(&state, "/api/v1/admin/bots/pending", &admin.token).await;
        assert_eq!(list_status, StatusCode::OK);
        let pending_json: serde_json::Value = serde_json::from_slice(&list_body).unwrap();
        assert_eq!(
            pending_json["bots"].as_array().unwrap().len(),
            1,
            "one bot awaiting approval"
        );

        // Step 6: Admin approves the bot.
        let (approve_status, approve_body) = post(
            &state,
            &format!("/api/v1/admin/bots/{bot_pubkey}/approve"),
            &admin.token,
            r#"{}"#,
        )
        .await;
        assert_eq!(
            approve_status,
            StatusCode::OK,
            "approve should succeed: {}",
            String::from_utf8_lossy(&approve_body)
        );

        // bot_approvals row should now have approved_at set.
        let approval = state.storage.get_bot_approval(&bot_pubkey).await.unwrap().unwrap();
        assert!(approval.approved_at.is_some(), "approved_at must be set after approval");
        assert!(approval.revoked_at.is_none(), "revoked_at must be empty");

        // Pending list should now be empty.
        let pending_after = state.storage.list_pending_bots().await.unwrap();
        assert!(pending_after.is_empty(), "no bots pending after approval");

        // Step 7: Bot re-authenticates — should now get a non-read-only session.
        let bot_auth2 = do_auth(&state, 0x10, true).await;
        assert!(
            !bot_auth2.is_read_only,
            "approved bot should get a normal (non-read-only) session"
        );
        assert_eq!(bot_auth2.user_id, bot_auth1.user_id, "same user_id on re-auth");

        // Step 8: Bot sends a message with the new session.
        let (msg_status, msg_body) = post(
            &state,
            &format!("/api/v1/channels/{channel_id}/messages"),
            &bot_auth2.token,
            r#"{"content":"hello from approved bot"}"#,
        )
        .await;
        assert_eq!(
            msg_status,
            StatusCode::OK,
            "approved bot should be able to send messages: {}",
            String::from_utf8_lossy(&msg_body)
        );

        // Step 9: Verify DB state — message is authored by the bot.
        let msg_json: serde_json::Value = serde_json::from_slice(&msg_body).unwrap();
        assert_eq!(msg_json["author_id"], bot_auth1.user_id);
        assert_eq!(msg_json["content"], "hello from approved bot");

        let messages = state.storage.list_messages(&channel_id, None, 50).await.unwrap();
        assert!(
            messages.iter().any(|m| m.author_id == bot_auth1.user_id),
            "message authored by bot must be in DB"
        );
    }

    #[tokio::test]
    async fn health_endpoint_returns_ok() {
        let response = app(test_state().await)
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let status: HealthStatus = serde_json::from_slice(&body).unwrap();
        assert_eq!(status, HealthStatus::ok());
    }

    #[tokio::test]
    async fn server_starts_with_custom_config_toml() {
        let toml = r#"
            [server]
            name = "integration-test"
            [network]
            bind_address = "127.0.0.1:0"
        "#;
        let cfg = ServerConfig::from_toml_str(toml).unwrap();
        let storage: DynStorage = Arc::new(SqliteStorage::in_memory().await.unwrap());
        let state = AppState {
            config: Arc::new(cfg),
            storage,
            challenge_store: auth::challenge_store(),
            gateway: gateway::gateway_handle(),
        };
        let response = app(state)
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn storage_error_variants_display() {
        use crate::storage::StorageError;
        assert!(StorageError::NotFound.to_string().contains("not found"));
        assert!(StorageError::Conflict("dup".into())
            .to_string()
            .contains("dup"));
        assert!(StorageError::Internal("boom".into())
            .to_string()
            .contains("boom"));
    }
}
