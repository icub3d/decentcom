use std::path::Path;

use serde::Deserialize;

use crate::error::{Error, Result};

/// Bot configuration loaded from a TOML file and/or environment variables.
///
/// Environment variables take precedence over TOML file values.
/// Required fields: `server_url`, `mnemonic`.
///
/// Env var names: `BOT_SERVER_URL`, `BOT_MNEMONIC`, `BOT_DISPLAY_NAME`.
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// Base URL of the decentcom server (e.g. `https://decentcom.example.com`).
    pub server_url: String,

    /// BIP39 mnemonic phrase for the bot's Ed25519 identity.
    pub mnemonic: String,

    /// Optional display name shown for the bot in the member list.
    #[serde(default)]
    pub display_name: Option<String>,
}

impl Config {
    /// Load from environment variables only (`BOT_SERVER_URL`, `BOT_MNEMONIC`).
    pub fn from_env() -> Result<Self> {
        let server_url = std::env::var("BOT_SERVER_URL")
            .map_err(|_| Error::Config("BOT_SERVER_URL not set".into()))?;
        let mnemonic = std::env::var("BOT_MNEMONIC")
            .map_err(|_| Error::Config("BOT_MNEMONIC not set".into()))?;
        let display_name = std::env::var("BOT_DISPLAY_NAME").ok();

        Ok(Self { server_url, mnemonic, display_name })
    }

    /// Load from a TOML file, then override with any env vars that are present.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let content = std::fs::read_to_string(path.as_ref())
            .map_err(|e| Error::Config(format!("cannot read config file: {e}")))?;
        let mut cfg: Config = toml::from_str(&content)
            .map_err(|e| Error::Config(format!("invalid TOML: {e}")))?;

        // env vars override
        if let Ok(v) = std::env::var("BOT_SERVER_URL") {
            cfg.server_url = v;
        }
        if let Ok(v) = std::env::var("BOT_MNEMONIC") {
            cfg.mnemonic = v;
        }
        if let Ok(v) = std::env::var("BOT_DISPLAY_NAME") {
            cfg.display_name = Some(v);
        }

        Ok(cfg)
    }

    /// Try loading from a TOML file if it exists, otherwise fall back to env vars.
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        if path.as_ref().exists() {
            Self::from_file(path)
        } else {
            Self::from_env()
        }
    }
}
