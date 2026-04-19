use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid mnemonic: {0}")]
    Mnemonic(String),

    #[error("invalid identity input: {0}")]
    Identity(String),

    #[error("HTTP transport error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("invalid URL: {0}")]
    Url(#[from] url::ParseError),

    #[error("API error ({status}): {message}")]
    Api { status: u16, message: String },

    #[error("not authenticated: call Client::authenticate() first")]
    Unauthenticated,

    #[error("JSON encoding/decoding: {0}")]
    Json(#[from] serde_json::Error),

    #[error("gateway transport error: {0}")]
    Gateway(String),

    #[error("protocol error: {0}")]
    Protocol(String),
}
