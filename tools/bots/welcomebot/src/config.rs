use std::path::Path;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub server_url: String,
    pub mnemonic: String,
    #[serde(default)]
    pub display_name: Option<String>,

    /// Channel ID where welcome messages are sent.
    pub welcome_channel_id: String,

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
    pub fn load(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path.as_ref())?;
        let mut cfg: Config = toml::from_str(&content)?;

        if let Ok(v) = std::env::var("BOT_SERVER_URL") {
            cfg.server_url = v;
        }
        if let Ok(v) = std::env::var("BOT_MNEMONIC") {
            cfg.mnemonic = v;
        }
        if let Ok(v) = std::env::var("BOT_DISPLAY_NAME") {
            cfg.display_name = Some(v);
        }
        if let Ok(v) = std::env::var("WELCOME_CHANNEL_ID") {
            cfg.welcome_channel_id = v;
        }
        if let Ok(v) = std::env::var("WELCOME_TEMPLATE") {
            cfg.welcome_template = v;
        }

        Ok(cfg)
    }
}
