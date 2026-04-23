use std::path::Path;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Warn,
    Kick,
    Ban,
}

impl Action {
    pub fn as_str(&self) -> &str {
        match self {
            Action::Warn => "warn",
            Action::Kick => "kick",
            Action::Ban => "ban",
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Detection {
    /// Number of identical messages within `duplicate_window_secs` to trigger action.
    #[serde(default = "default_duplicate_threshold")]
    pub duplicate_threshold: usize,

    /// Sliding window in seconds for duplicate detection.
    #[serde(default = "default_duplicate_window_secs")]
    pub duplicate_window_secs: u64,

    /// Number of links (http/https) within `link_window_secs` to trigger action.
    #[serde(default = "default_link_threshold")]
    pub link_threshold: usize,

    /// Sliding window in seconds for link flood detection.
    #[serde(default = "default_link_window_secs")]
    pub link_window_secs: u64,
}

fn default_duplicate_threshold() -> usize { 5 }
fn default_duplicate_window_secs() -> u64 { 30 }
fn default_link_threshold() -> usize { 5 }
fn default_link_window_secs() -> u64 { 60 }

impl Default for Detection {
    fn default() -> Self {
        Self {
            duplicate_threshold: default_duplicate_threshold(),
            duplicate_window_secs: default_duplicate_window_secs(),
            link_threshold: default_link_threshold(),
            link_window_secs: default_link_window_secs(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ActionPolicy {
    /// Action taken when a user hits the duplicate message threshold.
    #[serde(default = "default_on_duplicate")]
    pub on_duplicate: Action,

    /// Action taken when a user hits the link flood threshold.
    #[serde(default = "default_on_link_flood")]
    pub on_link_flood: Action,

    /// Channel ID where warning messages are posted (for `Action::Warn`).
    pub warn_channel_id: Option<String>,
}

fn default_on_duplicate() -> Action { Action::Warn }
fn default_on_link_flood() -> Action { Action::Kick }

impl Default for ActionPolicy {
    fn default() -> Self {
        Self {
            on_duplicate: default_on_duplicate(),
            on_link_flood: default_on_link_flood(),
            warn_channel_id: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub server_url: String,
    pub mnemonic: String,
    #[serde(default)]
    pub display_name: Option<String>,

    #[serde(default)]
    pub detection: Detection,

    #[serde(default)]
    pub action: ActionPolicy,
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

        Ok(cfg)
    }
}
