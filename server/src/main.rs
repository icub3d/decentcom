mod config;

use std::path::PathBuf;
use std::sync::Arc;

use axum::{extract::State, routing::get, Json, Router};
use clap::Parser;
use shared::HealthStatus;
use tracing::info;

use crate::config::ServerConfig;

#[derive(Parser, Debug)]
#[command(name = "decentcom-server", version, about = "decentcom server")]
struct Cli {
    /// Path to configuration file.
    #[arg(short, long, default_value = "decentcom.toml")]
    config: PathBuf,
}

pub type AppState = Arc<ServerConfig>;

pub fn app(state: AppState) -> Router {
    Router::new().route("/health", get(health)).with_state(state)
}

async fn health(State(_cfg): State<AppState>) -> Json<HealthStatus> {
    Json(HealthStatus::ok())
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

    let bind = config.network.bind_address;
    let state: AppState = Arc::new(config);
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

    fn test_state() -> AppState {
        Arc::new(ServerConfig::default())
    }

    #[tokio::test]
    async fn health_endpoint_returns_ok() {
        let response = app(test_state())
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
        let state: AppState = Arc::new(cfg);
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
}
