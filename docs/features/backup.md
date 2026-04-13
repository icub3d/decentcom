# Feature: Backup & Restore

## Overview
Backup and restore allows server operators to export all server data as a portable archive and import it on a fresh instance. This is essential for migrating to new hardware, recovering from failure, or cloning a server for testing. The archive format is self-contained and works across storage backends (a backup from a SQLite instance can be restored to a PostgreSQL instance and vice versa).

## Background
The storage design doc (`docs/design/storage.md`) specifies full backup, incremental backup, and restore as requirements. It raises an open question about a canonical interchange format (JSON + media zip). The server-model doc (`docs/design/server-model.md`) lists backup and restore as an admin operation accessible via both the client UI and a CLI tool.

Depends on: `storage` (feature #3), `server-config` (feature #2), all Phase 1 and Phase 2 features.

## Requirements
- [ ] Full backup exports all server data: users, channels, categories, messages, roles, permissions, invites, media files, server configuration
- [ ] The backup is a single archive file (`.dcbackup` extension) containing JSON metadata and media blobs
- [ ] The archive format is versioned and backend-agnostic (not a raw SQLite or PostgreSQL dump)
- [ ] Restore imports a backup archive to a fresh server instance, recreating all data
- [ ] Restore works across storage backends (SQLite backup can restore to PostgreSQL and vice versa)
- [ ] Backup can be triggered via the admin REST API or the CLI
- [ ] Backup runs without stopping the server (online backup with consistent snapshot)
- [ ] Large backups stream to disk rather than buffering in memory
- [ ] The backup includes a manifest with metadata: server name, backup timestamp, record counts, format version
- [ ] Restore to a non-empty server is rejected (must be a fresh instance)
- [ ] Only users with admin permissions can trigger backup/restore

## Design

### API / Interface Changes

**REST endpoints:**

| Method | Path | Description |
|---|---|---|
| POST | `/api/v1/admin/backup` | Initiate a full backup; returns a download URL or streams the archive |
| POST | `/api/v1/admin/restore` | Upload a backup archive to restore (multipart upload) |
| GET | `/api/v1/admin/backup/status` | Check status of an in-progress backup |

**CLI commands:**

| Command | Description |
|---|---|
| `decentcom backup --output <path>` | Export a full backup to the specified file |
| `decentcom restore --input <path>` | Import a backup archive to the current server |

### Data Model Changes

No schema changes. The backup archive uses its own internal format:

**Archive structure (tar.zst — zstd-compressed tar):**

```
backup.dcbackup (tar.zst)
├── manifest.json          # version, server name, timestamp, record counts
├── server_config.json     # server configuration snapshot
├── data/
│   ├── users.jsonl        # one JSON object per line
│   ├── channels.jsonl
│   ├── categories.jsonl
│   ├── messages.jsonl
│   ├── roles.jsonl
│   ├── permissions.jsonl
│   ├── invites.jsonl
│   ├── reactions.jsonl
│   ├── threads.jsonl
│   └── device_keys.jsonl
└── media/
    ├── <content_hash_1>   # raw media files, named by content hash
    ├── <content_hash_2>
    └── ...
```

### Component Changes

**Server (`server/`):**

- `server/src/backup/` — new module directory
  - `server/src/backup/mod.rs` — backup orchestrator: coordinates export/import across all stores
  - `server/src/backup/export.rs` — reads from the storage backend, writes JSONL data files and media blobs into a tar.zst archive
  - `server/src/backup/import.rs` — reads from a tar.zst archive, writes to the storage backend
  - `server/src/backup/manifest.rs` — manifest struct, serialization, version checking
  - `server/src/backup/format.rs` — constants: magic bytes, version, file paths within the archive
- `server/src/routes/admin.rs` — add backup and restore endpoints (admin-only, permission check)
- `server/src/bin/backup.rs` — CLI entry point for backup/restore (or subcommands of the main binary)
- `server/src/storage/trait.rs` — add `iter_all_*()` methods to each store trait for streaming export (e.g., `iter_all_messages() -> impl Stream<Item = Message>`)

**Dependencies:**

- `tar` crate — tar archive creation/reading
- `zstd` crate — zstd compression/decompression
- `serde_json` — JSONL serialization

## Task List

### Phase A: Archive Format and Manifest
- [ ] Create `server/src/backup/format.rs` — define archive structure constants, file extension, magic bytes
- [ ] Create `server/src/backup/manifest.rs` — `BackupManifest` struct with version, server name, timestamp, record counts; serialization and validation

### Phase B: Export
- [ ] Add `iter_all_*()` streaming methods to storage trait for each entity type (users, channels, messages, etc.)
- [ ] Implement `iter_all_*()` for SQLite backend
- [ ] Implement `iter_all_*()` for PostgreSQL backend (if available)
- [ ] Create `server/src/backup/export.rs` — streaming export: open tar.zst writer, iterate each store, write JSONL files, copy media blobs
- [ ] Create `server/src/backup/mod.rs` — `BackupManager` that coordinates the export process
- [ ] Add `decentcom backup --output <path>` CLI command
- [ ] Add `POST /api/v1/admin/backup` endpoint with admin permission check

### Phase C: Import
- [ ] Create `server/src/backup/import.rs` — streaming import: open tar.zst reader, parse manifest, insert records in dependency order (users first, then channels, then messages, etc.)
- [ ] Handle media import: write media blobs to the appropriate media backend (local disk or S3)
- [ ] Validate manifest version compatibility before importing
- [ ] Reject restore if the server already has data (non-empty check)
- [ ] Add `decentcom restore --input <path>` CLI command
- [ ] Add `POST /api/v1/admin/restore` endpoint with admin permission check

### Phase D: Status and Progress
- [ ] Add backup status tracking (in-progress, completed, failed, record counts)
- [ ] Add `GET /api/v1/admin/backup/status` endpoint
- [ ] Add progress logging for CLI backup/restore (percentage, records processed)

## Test List
- [ ] Unit test: manifest serialization and deserialization round-trips correctly
- [ ] Unit test: manifest version check rejects archives from unsupported versions
- [ ] Unit test: export produces a valid tar.zst archive with the expected file structure
- [ ] Unit test: JSONL serialization of each entity type produces valid JSON per line
- [ ] Integration test: full export-import cycle on SQLite — backup, restore to fresh instance, verify all data matches
- [ ] Integration test: cross-backend restore — backup from SQLite, restore to PostgreSQL (if postgres feature is available)
- [ ] Integration test: restore to non-empty server is rejected
- [ ] Integration test: backup with media files includes all media blobs in the archive
- [ ] Integration test: backup endpoint requires admin permission
- [ ] Manual test: backup a server with real data, restore to a new instance, verify all channels, messages, and media are intact
- [ ] Manual test: verify backup archive can be inspected with standard tar tools (`tar -tf backup.dcbackup`)

## Open Questions
- **Incremental backups:** The storage doc mentions incremental backup (changes since last backup). Should this be part of the initial implementation, or a follow-up? Incremental backup requires tracking a "last backup" watermark per entity type.
- **Archive size limits:** For very large servers, the backup archive could be gigabytes. Should the backup support splitting into multiple files? Or is streaming to object storage (S3) a better approach?
- **Encryption:** Should the backup archive be optionally encrypted (passphrase-protected)? This would protect data at rest if the archive is stored on untrusted media.
- **Discord import:** The storage doc mentions importing from Discord data exports. Should the import tool support Discord's export format as an alternative input? This is a separate feature but could share the import infrastructure.
