mod auth;
mod channels;
mod config;
mod gateway;
mod invites;
mod membership;
mod messages;
mod permissions;
mod profiles;
mod roles;
mod storage;

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
        .nest("/api/v1", profiles::profile_router())
        .nest("/api/v1", profiles::media_router())
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
