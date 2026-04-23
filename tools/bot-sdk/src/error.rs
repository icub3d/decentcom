use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("SDK error: {0}")]
    Sdk(#[from] decentcom_sdk::Error),

    #[error("configuration error: {0}")]
    Config(String),

    #[error("event handler error: {0}")]
    Handler(#[from] anyhow::Error),

    #[error("gateway disconnected")]
    Disconnected,

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}
