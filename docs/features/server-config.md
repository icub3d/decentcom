# Feature: Server Configuration

## Overview
Implement TOML-based configuration file loading for the decentcom server. This includes server identity (name, description, icon), network settings (bind address, TLS), feature flags, membership mode, content policy limits, and storage backend selection. The config is the primary way operators customize their instance.

## Background
The server model design doc ([server-model.md](../design/server-model.md)) defines membership modes, feature flags, and content policy settings. The storage design doc ([storage.md](../design/storage.md)) specifies backend selection via TOML config blocks. The architecture doc ([architecture.md](../design/architecture.md)) notes that one instance hosts exactly one community, so all config is instance-level.

## Requirements
- [x] Server loads configuration from a TOML file specified via `--config <path>` CLI argument
- [x] If no config file is specified, the server looks for `decentcom.toml` in the current directory
- [x] If no config file is found, the server starts with sensible defaults (bind `127.0.0.1:8080`, SQLite storage, open membership)
- [x] Server identity fields: `name` (required, string), `description` (optional, string), `icon_path` (optional, file path)
- [x] Network settings: `bind_address` (default `127.0.0.1:8080`)
- [x] Storage backend selection: `[storage]` section with `backend` field (`sqlite` or `postgres`), plus backend-specific fields
- [x] Feature flags: `[features]` section with boolean toggles matching the flags in the server-model design doc
- [x] Membership mode: `membership_mode` field (`open`, `invite_only`, `allowlist`, `closed`)
- [x] Content policy: `max_message_length`, `max_file_size`, `allowed_file_types`
- [x] Config validation at startup: invalid values produce clear error messages and prevent the server from starting
- [x] Config values are accessible throughout the server via a shared, read-only config struct

## Design

### API / Interface Changes
No new HTTP endpoints. Configuration is file-based, not API-based.

CLI interface for the server binary:
```
decentcom-server [OPTIONS]

Options:
  -c, --config <PATH>    Path to configuration file [default: decentcom.toml]
  -h, --help             Print help
  -V, --version          Print version
```

### Data Model Changes
None. Configuration is not stored in the database.

### Component Changes

**New files:**
```
server/src/config.rs        # Config structs, TOML deserialization, validation, defaults
```

**Modified files:**
```
server/Cargo.toml           # Add: toml, clap, tracing, tracing-subscriber
server/src/main.rs          # Parse CLI args, load config, pass to app state
```

**Config structs (in `server/src/config.rs`):**
```rust
pub struct ServerConfig {
    pub server: ServerIdentity,
    pub network: NetworkConfig,
    pub storage: StorageConfig,
    pub features: FeatureFlags,
    pub membership: MembershipConfig,
    pub content: ContentPolicy,
}

pub struct ServerIdentity {
    pub name: String,
    pub description: Option<String>,
    pub icon_path: Option<PathBuf>,
}

pub struct NetworkConfig {
    pub bind_address: SocketAddr,
}

pub struct StorageConfig {
    pub backend: StorageBackendType,  // sqlite | postgres
    pub database_path: Option<PathBuf>,
    pub database_url: Option<String>,
    pub media_path: Option<PathBuf>,
}

pub struct FeatureFlags {
    pub voice_channels: bool,
    pub video: bool,
    pub screen_share: bool,
    pub file_uploads: bool,
    pub emoji_reactions: bool,
    pub message_threads: bool,
    pub message_search: bool,
}

pub struct MembershipConfig {
    pub mode: MembershipMode,  // open | invite_only | allowlist | closed
}

pub struct ContentPolicy {
    pub max_message_length: usize,
    pub max_file_size: u64,
    pub allowed_file_types: Vec<String>,
}
```

## Task List
- [x] Add `toml`, `serde`, `clap`, `tracing`, `tracing-subscriber` dependencies to `server/Cargo.toml`
- [x] Define config structs with serde `Deserialize` in `server/src/config.rs`
- [x] Implement `Default` for all config structs with sensible values
- [x] Implement config file loading: read file, deserialize TOML, merge with defaults
- [x] Implement validation logic (e.g., `name` is non-empty, `database_path` is set when backend is `sqlite`, `database_url` is set when backend is `postgres`)
- [x] Add CLI argument parsing with clap in `server/src/main.rs` (`--config` flag)
- [x] Initialize tracing/logging subscriber at startup
- [x] Store the loaded `ServerConfig` in axum's app state (via `Arc<ServerConfig>` in router state)
- [x] Log the loaded configuration summary at startup (redacting sensitive fields like `database_url`)
- [x] Write an example `decentcom.toml` at `server/decentcom.example.toml`

## Test List
- [x] Unit test: default config is valid and passes validation
- [x] Unit test: a minimal TOML file (just `[server] name = "test"`) deserializes correctly with defaults for everything else
- [x] Unit test: a full TOML file with all sections deserializes correctly
- [x] Unit test: missing required field (`server.name` absent) returns a clear error
- [x] Unit test: invalid `membership_mode` value returns a clear error
- [x] Unit test: `backend = "sqlite"` without `database_path` returns a validation error
- [x] Unit test: `backend = "postgres"` without `database_url` returns a validation error
- [x] Integration test: server starts with default config and `/health` still responds
- [x] Integration test: server starts with a valid TOML config file
- [ ] Manual: run server with `--config path/to/test.toml` and verify logged config summary (**not verified in this session — covered by integration test that exercises the same config-loading path**)

## Open Questions
- Should the server support environment variable overrides for config values (e.g., `DECENTCOM_BIND_ADDRESS`)? This is useful for container deployments but adds complexity.
- Should the config be hot-reloadable (watch file for changes), or require a restart? A restart is simpler and safer for the initial implementation.
- The design doc mentions TLS configuration. Should TLS termination be handled by the server directly, or always delegated to a reverse proxy? For simplicity, the initial implementation assumes a reverse proxy handles TLS.
