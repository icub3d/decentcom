# Feature: Storage Layer

## Overview
Define the storage trait hierarchy that abstracts all persistence operations, then implement the SQLite backend as the default. This gives the server a working database for users, messages, channels, and sessions while keeping the door open for PostgreSQL and other backends later.

## Background
The storage design doc ([storage.md](../design/storage.md)) specifies a pluggable backend architecture with traits for `UserStore`, `MessageStore`, `ChannelStore`, `MediaStore`, and `SessionStore`. SQLite + local disk is the default backend requiring zero external dependencies. The architecture doc ([architecture.md](../design/architecture.md)) notes that session tokens may be kept in-memory rather than in the primary database. The server-config feature provides the `StorageConfig` struct that selects which backend to initialize.

## Requirements
- [x] Storage trait hierarchy defined in the `shared` crate (or a dedicated `storage` module in `server`) with async methods
- [x] Traits: `UserStore`, `MessageStore`, `ChannelStore`, `SessionStore`
- [x] `MediaStore` trait is defined but the implementation is deferred to the file-uploads feature
- [x] SQLite backend implements all four active traits using sqlx
- [x] Database schema is managed via sqlx migrations
- [x] SQLite database file is created automatically on first run at the path specified in config
- [x] WAL mode is enabled for SQLite for better concurrent read performance
- [x] Session tokens are stored in SQLite for simplicity (in-memory caching is a future optimization)
- [x] The storage layer is injected into the axum app state as a trait object (or generic) so handlers are backend-agnostic
- [x] All store methods return `Result<T, StorageError>` with a unified error type

## Design

### API / Interface Changes
No new HTTP endpoints. This feature provides internal infrastructure used by other features.

### Data Model Changes

**SQLite schema (managed via sqlx migrations):**

```sql
-- migrations/001_initial.sql

CREATE TABLE users (
    id          TEXT PRIMARY KEY,  -- ULID or UUID
    pubkey      TEXT NOT NULL UNIQUE,
    display_name TEXT,
    avatar_hash TEXT,
    created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE channels (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    topic       TEXT,
    category_id TEXT,
    position    INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE categories (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    position    INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE messages (
    id          TEXT PRIMARY KEY,
    channel_id  TEXT NOT NULL REFERENCES channels(id),
    author_id   TEXT NOT NULL REFERENCES users(id),
    content     TEXT NOT NULL,
    edited_at   TEXT,
    deleted     INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX idx_messages_channel_created ON messages(channel_id, created_at);

CREATE TABLE sessions (
    token       TEXT PRIMARY KEY,
    user_id     TEXT NOT NULL REFERENCES users(id),
    created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    expires_at  TEXT NOT NULL
);

CREATE INDEX idx_sessions_user ON sessions(user_id);
CREATE INDEX idx_sessions_expires ON sessions(expires_at);
```

### Component Changes

**New files:**
```
server/src/storage/
  mod.rs                    # Re-exports, StorageError type, Storage composite trait
  traits.rs                 # UserStore, MessageStore, ChannelStore, SessionStore, MediaStore trait definitions
  models.rs                 # Shared model structs (User, Message, Channel, Session, etc.)
  sqlite/
    mod.rs                  # SqliteStorage struct, initialization, migration runner
    users.rs                # UserStore impl for SqliteStorage
    messages.rs             # MessageStore impl for SqliteStorage
    channels.rs             # ChannelStore impl for SqliteStorage
    sessions.rs             # SessionStore impl for SqliteStorage
server/migrations/
  001_initial.sql           # Schema above
```

**Modified files:**
```
server/Cargo.toml           # Add: sqlx (sqlite, runtime-tokio), ulid or uuid
server/src/main.rs          # Initialize storage from config, inject into app state
shared/src/lib.rs           # Add shared model types if used across server and client
```

**Key trait definitions (in `server/src/storage/traits.rs`):**
```rust
#[async_trait]
pub trait UserStore: Send + Sync {
    async fn create_user(&self, pubkey: &str, display_name: Option<&str>) -> Result<User, StorageError>;
    async fn get_user_by_id(&self, id: &str) -> Result<Option<User>, StorageError>;
    async fn get_user_by_pubkey(&self, pubkey: &str) -> Result<Option<User>, StorageError>;
    async fn update_user(&self, id: &str, display_name: Option<&str>, avatar_hash: Option<&str>) -> Result<User, StorageError>;
    async fn list_users(&self) -> Result<Vec<User>, StorageError>;
}

#[async_trait]
pub trait MessageStore: Send + Sync {
    async fn create_message(&self, channel_id: &str, author_id: &str, content: &str) -> Result<Message, StorageError>;
    async fn get_message(&self, id: &str) -> Result<Option<Message>, StorageError>;
    async fn list_messages(&self, channel_id: &str, before: Option<&str>, limit: u32) -> Result<Vec<Message>, StorageError>;
    async fn update_message(&self, id: &str, content: &str) -> Result<Message, StorageError>;
    async fn delete_message(&self, id: &str) -> Result<(), StorageError>;
}

#[async_trait]
pub trait ChannelStore: Send + Sync {
    async fn create_channel(&self, name: &str, category_id: Option<&str>, position: i32) -> Result<Channel, StorageError>;
    async fn get_channel(&self, id: &str) -> Result<Option<Channel>, StorageError>;
    async fn list_channels(&self) -> Result<Vec<Channel>, StorageError>;
    async fn update_channel(&self, id: &str, name: Option<&str>, topic: Option<&str>, position: Option<i32>) -> Result<Channel, StorageError>;
    async fn delete_channel(&self, id: &str) -> Result<(), StorageError>;
}

#[async_trait]
pub trait SessionStore: Send + Sync {
    async fn create_session(&self, user_id: &str, duration: Duration) -> Result<Session, StorageError>;
    async fn get_session(&self, token: &str) -> Result<Option<Session>, StorageError>;
    async fn delete_session(&self, token: &str) -> Result<(), StorageError>;
    async fn delete_expired_sessions(&self) -> Result<u64, StorageError>;
}
```

## Task List
- [x] Add `sqlx` (with `sqlite` and `runtime-tokio` features), `async-trait`, and `ulid` (or `uuid`) to `server/Cargo.toml`
- [x] Create `server/src/storage/mod.rs` with `StorageError` enum and a composite `Storage` trait (or struct wrapping all sub-traits)
- [x] Define model structs in `server/src/storage/models.rs`: `User`, `Message`, `Channel`, `Category`, `Session`
- [x] Define `UserStore`, `MessageStore`, `ChannelStore`, `SessionStore`, and `MediaStore` traits in `server/src/storage/traits.rs`
- [x] Create `server/migrations/001_initial.sql` with the schema
- [x] Implement `SqliteStorage` struct in `server/src/storage/sqlite/mod.rs` with pool initialization and migration running
- [x] Enable WAL mode on the SQLite connection pool
- [x] Implement `UserStore` for `SqliteStorage` in `server/src/storage/sqlite/users.rs`
- [x] Implement `ChannelStore` for `SqliteStorage` in `server/src/storage/sqlite/channels.rs`
- [x] Implement `MessageStore` for `SqliteStorage` in `server/src/storage/sqlite/messages.rs` with cursor-based pagination
- [x] Implement `SessionStore` for `SqliteStorage` in `server/src/storage/sqlite/sessions.rs`
- [x] Wire up storage initialization in `server/src/main.rs`: read config, create `SqliteStorage`, inject into axum state
- [x] Add a session expiry cleanup task (tokio background task that calls `delete_expired_sessions` periodically)

## Test List
- [x] Unit test: `SqliteStorage` initializes with an in-memory database and runs migrations successfully
- [x] Unit test: `UserStore` — create user, get by id, get by pubkey, update, list
- [x] Unit test: `UserStore` — duplicate pubkey returns appropriate error
- [x] Unit test: `ChannelStore` — create, get, list, update, delete
- [x] Unit test: `MessageStore` — create message, get by id, list by channel with ordering
- [x] Unit test: `MessageStore` — cursor-based pagination returns correct pages
- [x] Unit test: `MessageStore` — soft delete sets `deleted` flag, message no longer appears in list
- [x] Unit test: `SessionStore` — create session, retrieve by token, verify expiry time
- [x] Unit test: `SessionStore` — `delete_expired_sessions` removes only expired sessions
- [x] Unit test: `StorageError` variants cover not-found, conflict, and internal error cases
- [x] Integration test: server starts, initializes SQLite storage, `/health` still responds

## Implementation Notes

- All SQLite queries use runtime `sqlx::query()` with `.bind()` and `Row::get()` rather than the `query!()` compile-time macros. The macros infer column nullability inconsistently for SQLite depending on whether a WHERE clause is present, causing type mismatches between `T` and `Option<T>` for the same NOT-NULL column. Runtime queries sidestep this entirely.
- `SqliteStorage` holds a `Mutex<ulid::Generator>` so IDs are monotonically increasing even when multiple records are created within the same millisecond. This ensures consistent ordering in message pagination tests and production.
- The in-memory pool for tests uses `max_connections(1)` because each SQLite `:memory:` connection is a separate isolated database; without this, migrations run on one connection but queries land on another.
- Open questions from the design doc are resolved: ULIDs are used (sortable by time, ideal for cursor pagination); model types stay in `server/src/storage/models.rs`; the `media` table is included in `001_initial.sql`.

## Open Questions
- Should model types live in `shared` (so the client Tauri core can also use them) or in `server/src/storage/models.rs`? If they live in `shared`, the client gets the same `User`, `Message`, etc. structs for deserialization. This is likely the right call but means `shared` gains a dependency on `serde` (which it probably already has).
- Should IDs be ULIDs (sortable, encode creation time) or UUIDs (more widely supported)? ULIDs are a better fit for message ordering.
- The `MediaStore` trait is defined here but not implemented. Should it include just the trait definition, or also the `media` table migration? Including the migration keeps all schema in one place; deferring it avoids unused tables.
