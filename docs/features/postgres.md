# Feature: PostgreSQL Backend

## Overview
The PostgreSQL storage backend is the scale-out alternative to the default SQLite backend. It enables multi-process server deployments behind a load balancer and pairs with S3-compatible object storage for media. This feature also provides migration tooling to move an existing SQLite-based server to PostgreSQL (and back) without data loss.

## Background
The storage design doc (`docs/design/storage.md`) defines a pluggable storage trait hierarchy (`StorageBackend` with `UserStore`, `MessageStore`, `ChannelStore`, `MediaStore`, `SessionStore`). The SQLite backend is the default implementation built in Phase 1 (`docs/features/scaffolding.md`). The PostgreSQL backend implements the same trait using sqlx with the PostgreSQL driver. Media shifts from local disk (content-addressable files) to an S3-compatible object store. The configuration is described in the storage doc with a `[storage]` TOML section specifying `backend = "postgres"` and `[storage.media]` for S3 settings.

Depends on: `storage` (feature #3), all Phase 1 and Phase 2 features.

## Requirements
- [ ] All storage trait methods have a PostgreSQL implementation that passes the same test suite as the SQLite backend
- [ ] PostgreSQL schema uses proper types: BIGSERIAL for IDs, TIMESTAMPTZ for timestamps, BYTEA for binary data
- [ ] Media storage uses an S3-compatible object store (AWS S3, MinIO, Backblaze B2, Cloudflare R2)
- [ ] Media is served via pre-signed URLs when using S3 (avoids proxying large blobs through the app server)
- [ ] Connection pooling is configured via sqlx's PgPool
- [ ] Database migrations are managed via sqlx migrations (separate migration directory from SQLite)
- [ ] A CLI migration tool can export a SQLite database to PostgreSQL and vice versa
- [ ] The server selects the backend based on the `[storage].backend` config field
- [ ] Full-text search uses PostgreSQL tsvector indexes instead of SQLite FTS5

## Design

### API / Interface Changes

No REST API changes. The storage backend is transparent to clients. One new admin CLI command:

| Command | Description |
|---|---|
| `decentcom migrate --from sqlite --to postgres --config <path>` | Migrate all data from SQLite to PostgreSQL |
| `decentcom migrate --from postgres --to sqlite --config <path>` | Migrate all data from PostgreSQL to SQLite |

Media serving endpoint behavior changes: when using S3, `GET /api/v1/media/{media_id}` returns a 302 redirect to a pre-signed S3 URL instead of streaming the file body.

### Data Model Changes

**PostgreSQL schema** (mirrors SQLite schema with PostgreSQL-appropriate types):

All existing tables (`users`, `messages`, `channels`, `categories`, `roles`, `role_permissions`, `channel_permission_overrides`, `invites`, `sessions`, `media`, `reactions`, `threads`, `dms`, `device_keys`) are recreated with:
- `BIGSERIAL` primary keys instead of `INTEGER PRIMARY KEY AUTOINCREMENT`
- `TIMESTAMPTZ` instead of `TEXT` for timestamps
- `BYTEA` for binary fields (pubkeys, signatures)
- `TEXT` with `tsvector` generated column on `messages.content` for full-text search
- Proper foreign key constraints with `ON DELETE CASCADE` where appropriate

**New table for S3 media tracking:**

| Column | Type | Description |
|---|---|---|
| `media_id` | BIGSERIAL | Primary key |
| `content_hash` | TEXT | SHA-256 or BLAKE3 hash |
| `s3_key` | TEXT | Object key in the S3 bucket |
| `bucket` | TEXT | Bucket name |
| `size_bytes` | BIGINT | File size |
| `mime_type` | TEXT | MIME type |
| `uploaded_by` | BIGINT | FK to users |
| `uploaded_at` | TIMESTAMPTZ | Upload timestamp |

### Component Changes

**Server (`server/`):**

- `server/src/storage/postgres/` — new module directory for the PostgreSQL backend
  - `server/src/storage/postgres/mod.rs` — `PostgresBackend` struct implementing `StorageBackend`
  - `server/src/storage/postgres/users.rs` — `UserStore` implementation
  - `server/src/storage/postgres/messages.rs` — `MessageStore` implementation with tsvector search
  - `server/src/storage/postgres/channels.rs` — `ChannelStore` implementation
  - `server/src/storage/postgres/sessions.rs` — `SessionStore` implementation
  - `server/src/storage/postgres/media.rs` — `MediaStore` implementation using S3
- `server/src/storage/s3.rs` — S3 client wrapper (upload, download, pre-signed URL generation)
- `server/migrations/postgres/` — PostgreSQL migration files (separate from `server/migrations/sqlite/`)
- `server/src/storage/migrate.rs` — migration tool: reads from one backend, writes to another
- `server/src/config.rs` — parse `[storage.media]` S3 configuration
- `server/src/routes/media.rs` — modify media serving to return 302 redirect for S3 backend
- `server/src/bin/migrate.rs` — CLI entry point for the migration tool (or subcommand of the main binary)

**Dependencies:**

- `aws-sdk-s3` or `rust-s3` crate for S3 operations
- sqlx with the `postgres` feature enabled

## Task List

### Phase A: PostgreSQL Schema and Connection
- [ ] Add sqlx `postgres` feature to `server/Cargo.toml`
- [ ] Create `server/migrations/postgres/` directory with initial migration matching the SQLite schema (using PostgreSQL types)
- [ ] Create `server/src/storage/postgres/mod.rs` — `PostgresBackend` struct with `PgPool`, implements `StorageBackend` trait
- [ ] Implement `UserStore` for PostgreSQL in `server/src/storage/postgres/users.rs`
- [ ] Implement `ChannelStore` for PostgreSQL in `server/src/storage/postgres/channels.rs`
- [ ] Implement `SessionStore` for PostgreSQL in `server/src/storage/postgres/sessions.rs`
- [ ] Implement `MessageStore` for PostgreSQL in `server/src/storage/postgres/messages.rs`
- [ ] Add tsvector index and full-text search query implementation for messages

### Phase B: S3 Media Storage
- [ ] Add S3 client dependency (`aws-sdk-s3` or `rust-s3`) to `server/Cargo.toml`
- [ ] Create `server/src/storage/s3.rs` — S3 client wrapper with `upload`, `download`, `presigned_url`, and `delete` methods
- [ ] Implement `MediaStore` for PostgreSQL+S3 in `server/src/storage/postgres/media.rs`
- [ ] Modify `server/src/routes/media.rs` to return 302 redirect to pre-signed URL when using S3 backend
- [ ] Parse `[storage.media]` config section for S3 bucket, region, and credentials

### Phase C: Backend Selection and Config
- [ ] Modify `server/src/storage/mod.rs` to select backend based on `[storage].backend` config value
- [ ] Update `server/src/config.rs` with full PostgreSQL and S3 configuration parsing
- [ ] Ensure all existing tests pass against both SQLite and PostgreSQL backends (parameterized test suite)

### Phase D: Migration Tooling
- [ ] Create `server/src/storage/migrate.rs` — generic migration: iterate all records from source backend, write to destination backend
- [ ] Implement media migration: copy files from local disk to S3 (or S3 to local disk)
- [ ] Create CLI entry point (`decentcom migrate` subcommand) in `server/src/bin/migrate.rs` or as a subcommand of the main server binary
- [ ] Add progress reporting for large migrations (record count, percentage)
- [ ] Handle migration of content-addressable media (verify hashes after transfer)

## Test List
- [ ] Unit test: all `UserStore` methods against PostgreSQL (same assertions as SQLite tests)
- [ ] Unit test: all `MessageStore` methods against PostgreSQL including full-text search
- [ ] Unit test: all `ChannelStore` methods against PostgreSQL
- [ ] Unit test: all `SessionStore` methods against PostgreSQL
- [ ] Unit test: S3 client wrapper uploads, downloads, generates pre-signed URLs, and deletes objects (use MinIO in tests or mock)
- [ ] Unit test: `MediaStore` stores metadata in PostgreSQL and blob in S3
- [ ] Integration test: media endpoint returns 302 redirect with valid pre-signed URL when using S3 backend
- [ ] Integration test: full migration from SQLite to PostgreSQL preserves all records and media
- [ ] Integration test: full migration from PostgreSQL to SQLite preserves all records and media
- [ ] Integration test: server starts with `backend = "postgres"` config and serves requests correctly
- [ ] Manual test: deploy server with PostgreSQL + MinIO and verify all features work end-to-end

## Open Questions
- **S3 crate choice:** `aws-sdk-s3` is the official AWS SDK but heavy. `rust-s3` is lighter but less maintained. Which to use?
- **Pre-signed URL expiry:** How long should pre-signed URLs be valid? Short (5 minutes) is more secure but requires re-fetching. Long (1 hour) is better for UX. Should this be configurable?
- **Migration downtime:** The migration tool requires the server to be stopped. Should we support online migration (read from old, write to new, then switch)? This is significantly more complex.
- **CockroachDB compatibility:** The storage doc mentions CockroachDB as a possible PostgreSQL-compatible option. Should we test against it or explicitly document it as unsupported?
