use std::path::Path;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub server_url: String,
    pub mnemonic: String,
    #[serde(default)]
    pub display_name: Option<String>,
    /// Raw 32-byte Ed25519 seed as 64 hex chars (alternative to mnemonic).
    #[serde(default)]
    pub seed_hex: Option<String>,

    /// Channel name *or* channel ID where welcome messages are sent.
    /// The bot resolves names to IDs at first use via the REST API.
    pub welcome_channel: String,

    /// Template for the welcome message.
    /// Supports `{display_name}` and `{pubkey}` placeholders.
    #[serde(default = "default_welcome_template")]
    pub welcome_template: String,

    /// Optional additional message posted after the welcome (e.g. server rules).
    #[serde(default)]
    pub rules_message: Option<String>,
}

fn default_welcome_template() -> String {
    "Welcome, **{display_name}**! Glad to have you here.".to_string()
}

impl Config {
    /// Load from `path`, with env var overrides.
    /// The config path itself is read from the `WELCOMEBOT_CONFIG` env var,
    /// defaulting to `welcomebot.toml`.
    pub fn load(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path.as_ref())?;
        let mut cfg: Config = toml::from_str(&content)?;

        if let Ok(v) = std::env::var("BOT_SERVER_URL") {
            cfg.server_url = v;
        }
        if let Ok(v) = std::env::var("BOT_MNEMONIC") {
            cfg.mnemonic = v;
        }
        if let Ok(v) = std::env::var("BOT_SEED_HEX") {
            cfg.seed_hex = Some(v);
        }
        if let Ok(v) = std::env::var("BOT_DISPLAY_NAME") {
            cfg.display_name = Some(v);
        }
        if let Ok(v) = std::env::var("WELCOME_CHANNEL") {
            cfg.welcome_channel = v;
        }
        if let Ok(v) = std::env::var("WELCOME_TEMPLATE") {
            cfg.welcome_template = v;
        }

        Ok(cfg)
    }

    /// Returns the path to load config from: `$WELCOMEBOT_CONFIG` or `welcomebot.toml`.
    pub fn default_path() -> String {
        std::env::var("WELCOMEBOT_CONFIG").unwrap_or_else(|_| "welcomebot.toml".to_string())
    }
}
