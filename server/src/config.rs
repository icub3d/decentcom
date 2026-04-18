use std::fmt;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};

use serde::Deserialize;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ServerConfig {
    pub server: ServerIdentity,
    pub network: NetworkConfig,
    pub storage: StorageConfig,
    pub features: FeatureFlags,
    pub membership: MembershipConfig,
    pub content: ContentPolicy,
    pub auth: AuthConfig,
    pub gateway: GatewayConfig,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ServerIdentity {
    pub name: String,
    pub description: Option<String>,
    pub icon_path: Option<PathBuf>,
    pub membership_mode: Option<MembershipMode>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct NetworkConfig {
    pub bind_address: SocketAddr,
    pub allowed_origins: Vec<String>,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StorageBackendType {
    Sqlite,
    Postgres,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct StorageConfig {
    pub backend: StorageBackendType,
    pub database_path: Option<PathBuf>,
    pub database_url: Option<String>,
    pub media_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct FeatureFlags {
    pub voice_channels: bool,
    pub video: bool,
    pub screen_share: bool,
    pub file_uploads: bool,
    pub emoji_reactions: bool,
    pub message_threads: bool,
    pub message_search: bool,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MembershipMode {
    Open,
    InviteOnly,
    Allowlist,
    Closed,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct MembershipConfig {
    pub mode: MembershipMode,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ContentPolicy {
    pub max_message_length: usize,
    pub max_file_size: u64,
    pub allowed_file_types: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct AuthConfig {
    pub session_ttl_seconds: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct GatewayConfig {
    pub heartbeat_interval_ms: u64,
    pub heartbeat_ack_timeout_ms: u64,
}

impl Default for ServerIdentity {
    fn default() -> Self {
        Self {
            name: "decentcom".to_string(),
            description: None,
            icon_path: None,
            membership_mode: None,
        }
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            bind_address: "127.0.0.1:8080".parse().expect("valid default addr"),
            allowed_origins: Vec::new(),
        }
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            backend: StorageBackendType::Sqlite,
            database_path: Some(PathBuf::from("decentcom.db")),
            database_url: None,
            media_path: Some(PathBuf::from("media")),
        }
    }
}

impl Default for FeatureFlags {
    fn default() -> Self {
        Self {
            voice_channels: true,
            video: true,
            screen_share: true,
            file_uploads: true,
            emoji_reactions: true,
            message_threads: true,
            message_search: true,
        }
    }
}

impl Default for MembershipConfig {
    fn default() -> Self {
        Self {
            mode: MembershipMode::Open,
        }
    }
}

impl Default for ContentPolicy {
    fn default() -> Self {
        Self {
            max_message_length: 4000,
            max_file_size: 25 * 1024 * 1024,
            allowed_file_types: Vec::new(),
        }
    }
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            session_ttl_seconds: 24 * 60 * 60,
        }
    }
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            heartbeat_interval_ms: 30_000,
            heartbeat_ack_timeout_ms: 10_000,
        }
    }
}

#[derive(Debug)]
pub enum ConfigError {
    Io(std::io::Error),
    Parse(toml::de::Error),
    Validation(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::Io(e) => write!(f, "failed to read config file: {e}"),
            ConfigError::Parse(e) => write!(f, "failed to parse config file: {e}"),
            ConfigError::Validation(msg) => write!(f, "invalid configuration: {msg}"),
        }
    }
}

impl std::error::Error for ConfigError {}

impl ServerConfig {
    pub fn from_toml_str(s: &str) -> Result<Self, ConfigError> {
        let mut cfg: ServerConfig = toml::from_str(s).map_err(ConfigError::Parse)?;
        if let Some(mode) = cfg.server.membership_mode {
            cfg.membership.mode = mode;
        }
        cfg.validate()?;
        Ok(cfg)
    }

    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        let contents = std::fs::read_to_string(path).map_err(ConfigError::Io)?;
        Self::from_toml_str(&contents)
    }

    pub fn load_or_default(path: &Path) -> Result<Self, ConfigError> {
        match std::fs::read_to_string(path) {
            Ok(contents) => Self::from_toml_str(&contents),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                let cfg = Self::default();
                cfg.validate()?;
                Ok(cfg)
            }
            Err(e) => Err(ConfigError::Io(e)),
        }
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.server.name.trim().is_empty() {
            return Err(ConfigError::Validation(
                "server.name must not be empty".into(),
            ));
        }
        match self.storage.backend {
            StorageBackendType::Sqlite => {
                if self.storage.database_path.is_none() {
                    return Err(ConfigError::Validation(
                        "storage.database_path is required when backend = \"sqlite\"".into(),
                    ));
                }
            }
            StorageBackendType::Postgres => {
                if self.storage.database_url.is_none() {
                    return Err(ConfigError::Validation(
                        "storage.database_url is required when backend = \"postgres\"".into(),
                    ));
                }
            }
        }
        if self.content.max_message_length == 0 {
            return Err(ConfigError::Validation(
                "content.max_message_length must be greater than 0".into(),
            ));
        }
        Ok(())
    }

    /// Return a human-readable summary of the config with sensitive fields redacted.
    pub fn summary(&self) -> String {
        let db_url = self
            .storage
            .database_url
            .as_ref()
            .map(|_| "<redacted>")
            .unwrap_or("<unset>");
        format!(
            "name={:?} bind={} backend={:?} db_path={:?} db_url={} media={:?} membership={:?} auth_ttl={}s hb={}ms ack={}ms",
            self.server.name,
            self.network.bind_address,
            self.storage.backend,
            self.storage.database_path,
            db_url,
            self.storage.media_path,
            self.membership.mode,
            self.auth.session_ttl_seconds,
            self.gateway.heartbeat_interval_ms,
            self.gateway.heartbeat_ack_timeout_ms,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_valid() {
        let cfg = ServerConfig::default();
        cfg.validate().expect("default should validate");
        assert_eq!(cfg.server.name, "decentcom");
        assert_eq!(cfg.storage.backend, StorageBackendType::Sqlite);
        assert_eq!(cfg.membership.mode, MembershipMode::Open);
        assert_eq!(cfg.auth.session_ttl_seconds, 24 * 60 * 60);
        assert_eq!(cfg.gateway.heartbeat_interval_ms, 30_000);
        assert_eq!(cfg.gateway.heartbeat_ack_timeout_ms, 10_000);
    }

    #[test]
    fn minimal_toml_uses_defaults() {
        let toml = r#"
            [server]
            name = "test"
        "#;
        let cfg = ServerConfig::from_toml_str(toml).unwrap();
        assert_eq!(cfg.server.name, "test");
        assert_eq!(cfg.network.bind_address.to_string(), "127.0.0.1:8080");
        assert_eq!(cfg.storage.backend, StorageBackendType::Sqlite);
        assert!(cfg.features.voice_channels);
        assert_eq!(cfg.auth.session_ttl_seconds, 24 * 60 * 60);
        assert_eq!(cfg.gateway.heartbeat_interval_ms, 30_000);
        assert_eq!(cfg.gateway.heartbeat_ack_timeout_ms, 10_000);
    }

    #[test]
    fn full_toml_roundtrip() {
        let toml = r#"
            [server]
            name = "My Community"
            description = "A test"

            [network]
            bind_address = "0.0.0.0:9000"

            [storage]
            backend = "postgres"
            database_url = "postgresql://localhost/decentcom"

            [features]
            voice_channels = false
            video = false
            screen_share = false
            file_uploads = true
            emoji_reactions = true
            message_threads = false
            message_search = false

            [membership]
            mode = "invite_only"

            [content]
            max_message_length = 2000
            max_file_size = 1048576
            allowed_file_types = ["image/png", "image/jpeg"]

            [auth]
            session_ttl_seconds = 3600

            [gateway]
            heartbeat_interval_ms = 20000
            heartbeat_ack_timeout_ms = 5000
        "#;
        let cfg = ServerConfig::from_toml_str(toml).unwrap();
        assert_eq!(cfg.server.name, "My Community");
        assert_eq!(cfg.server.description.as_deref(), Some("A test"));
        assert_eq!(cfg.network.bind_address.to_string(), "0.0.0.0:9000");
        assert_eq!(cfg.storage.backend, StorageBackendType::Postgres);
        assert!(!cfg.features.voice_channels);
        assert_eq!(cfg.membership.mode, MembershipMode::InviteOnly);
        assert_eq!(cfg.content.max_message_length, 2000);
        assert_eq!(cfg.content.allowed_file_types.len(), 2);
        assert_eq!(cfg.auth.session_ttl_seconds, 3600);
        assert_eq!(cfg.gateway.heartbeat_interval_ms, 20000);
        assert_eq!(cfg.gateway.heartbeat_ack_timeout_ms, 5000);
    }

    #[test]
    fn empty_server_name_fails() {
        let toml = r#"
            [server]
            name = ""
        "#;
        let err = ServerConfig::from_toml_str(toml).unwrap_err();
        assert!(matches!(err, ConfigError::Validation(_)));
        assert!(err.to_string().contains("server.name"));
    }

    #[test]
    fn invalid_membership_mode_fails() {
        let toml = r#"
            [server]
            name = "test"
            [membership]
            mode = "nonsense"
        "#;
        let err = ServerConfig::from_toml_str(toml).unwrap_err();
        assert!(matches!(err, ConfigError::Parse(_)));
    }

    #[test]
    fn sqlite_without_database_path_fails() {
        let toml = r#"
            [server]
            name = "test"
            [storage]
            backend = "sqlite"
            database_path = ""
        "#;
        // empty-path is still "set"; use explicit None via omitting w/ non-default:
        // Serde defaults to Some("decentcom.db") so we must override by null.
        // TOML has no null — omit the key entirely and override default via a manual struct test.
        let _ = toml;
        let mut cfg = ServerConfig::default();
        cfg.storage.database_path = None;
        let err = cfg.validate().unwrap_err();
        assert!(err.to_string().contains("database_path"));
    }

    #[test]
    fn postgres_without_database_url_fails() {
        let toml = r#"
            [server]
            name = "test"
            [storage]
            backend = "postgres"
        "#;
        let err = ServerConfig::from_toml_str(toml).unwrap_err();
        assert!(matches!(err, ConfigError::Validation(_)));
        assert!(err.to_string().contains("database_url"));
    }

    #[test]
    fn summary_redacts_database_url() {
        let mut cfg = ServerConfig::default();
        cfg.storage.database_url = Some("postgresql://user:secret@host/db".into());
        let s = cfg.summary();
        assert!(s.contains("<redacted>"));
        assert!(!s.contains("secret"));
    }
}
