use std::path::Path;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SinkType {
    File,
    Webhook,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EventFilter {
    #[serde(default = "yes")]
    pub member_kick: bool,
    #[serde(default = "yes")]
    pub member_ban: bool,
    #[serde(default = "yes")]
    pub message_delete: bool,
    #[serde(default = "no")]
    pub member_join: bool,
    #[serde(default = "no")]
    pub member_leave: bool,
}

fn yes() -> bool { true }
fn no() -> bool { false }

impl Default for EventFilter {
    fn default() -> Self {
        Self {
            member_kick: true,
            member_ban: true,
            message_delete: true,
            member_join: false,
            member_leave: false,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub server_url: String,
    pub mnemonic: String,
    #[serde(default)]
    pub display_name: Option<String>,

    /// Sink type: `"file"` or `"webhook"`.
    pub sink_type: SinkType,

    /// For `sink_type = "file"`: path to the log file (NDJSON).
    pub sink_path: Option<String>,

    /// For `sink_type = "webhook"`: URL to POST events to.
    pub sink_url: Option<String>,

    #[serde(default)]
    pub filter: EventFilter,
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
        if let Ok(v) = std::env::var("AUDIT_SINK_PATH") {
            cfg.sink_path = Some(v);
        }
        if let Ok(v) = std::env::var("AUDIT_SINK_URL") {
            cfg.sink_url = Some(v);
        }

        Ok(cfg)
    }
}
