# Feature: Message Retention

## Overview
Message retention allows server operators to configure automatic cleanup policies for messages and media. Policies define how long messages are kept (forever, a number of days, or a maximum count per channel). A background purge job periodically identifies and removes expired messages and orphaned media files, freeing storage space.

## Background
The storage design doc (`docs/design/storage.md`) defines four retention policies: `forever`, `days: N`, `count: N`, and `manual`. Messages are soft-deleted first, then a background job purges soft-deleted records and orphaned media on a configurable schedule. The server-model doc (`docs/design/server-model.md`) lists `retention_policy` as a content policy configuration option. Media files are content-addressable (stored by hash), so media cleanup must check that no remaining message references a blob before deleting it.

Depends on: `storage` (feature #3), `messages` (feature #8), `file-uploads` (feature #17), all Phase 1 and Phase 2 features.

## Requirements
- [ ] Server operators can configure a global retention policy in the server config
- [ ] Per-channel retention policy overrides are supported (a channel can have a different policy than the server default)
- [ ] Supported policies: `forever` (no automatic deletion), `days: N` (delete messages older than N days), `count: N` (keep only the N most recent messages per channel)
- [ ] A background purge job runs on a configurable interval (default: hourly)
- [ ] The purge job soft-deletes messages that exceed the retention policy
- [ ] A separate cleanup pass permanently deletes soft-deleted messages older than a grace period (default: 7 days)
- [ ] Orphaned media files (not referenced by any non-deleted message) are deleted after the grace period
- [ ] The purge job logs its activity (messages purged, media cleaned, duration)
- [ ] Pinned messages are exempt from retention policies (they are never automatically deleted)
- [ ] The retention policy is configurable via the admin API and the server config file
- [ ] Changing the retention policy triggers an immediate purge pass (not just on the next scheduled interval)

## Design

### API / Interface Changes

**REST endpoints:**

| Method | Path | Description |
|---|---|---|
| GET | `/api/v1/admin/retention` | Get the current global retention policy |
| PUT | `/api/v1/admin/retention` | Update the global retention policy |
| GET | `/api/v1/channels/{channel_id}/retention` | Get the channel-specific retention override (if any) |
| PUT | `/api/v1/channels/{channel_id}/retention` | Set or clear a channel-specific retention override |

**Request body for PUT:**

```json
{
  "policy": "days",
  "value": 90,
  "grace_period_days": 7
}
```

Or:

```json
{
  "policy": "forever"
}
```

**Server config (TOML):**

```toml
[retention]
policy = "days"       # "forever", "days", "count"
value = 90            # N for days or count policies
grace_period_days = 7 # how long soft-deleted records are kept before hard delete
purge_interval = "1h" # how often the purge job runs
```

### Data Model Changes

**Modified `messages` table:**

| Column | Type | Description |
|---|---|---|
| `deleted_at` | TIMESTAMP (nullable) | When the message was soft-deleted (NULL = active) |

This column may already exist from the messages feature. If not, add it.

**New table: `channel_retention_overrides`**

| Column | Type | Description |
|---|---|---|
| `channel_id` | INTEGER/BIGINT | FK to channels, unique |
| `policy` | TEXT | `forever`, `days`, or `count` |
| `value` | INTEGER (nullable) | N for days or count; NULL for forever |
| `grace_period_days` | INTEGER | Grace period before hard delete |

**Modified `media` table:**

| Column | Type | Description |
|---|---|---|
| `orphaned_at` | TIMESTAMP (nullable) | When the media was detected as orphaned (NULL = referenced) |

### Component Changes

**Server (`server/`):**

- `server/src/retention/` — new module directory
  - `server/src/retention/mod.rs` — retention manager: starts the background job, provides methods for immediate purge
  - `server/src/retention/policy.rs` — `RetentionPolicy` enum and configuration parsing
  - `server/src/retention/purge.rs` — purge logic: identify expired messages by policy, soft-delete them, hard-delete after grace period, clean orphaned media
  - `server/src/retention/scheduler.rs` — tokio interval-based scheduler for the background purge job
- `server/src/storage/trait.rs` — extend `MessageStore` with `soft_delete_expired(policy, channel_id)`, `hard_delete_old_soft_deleted(grace_period)`, and `count_messages(channel_id)` methods
- `server/src/storage/trait.rs` — extend `MediaStore` with `mark_orphaned_media()` and `delete_orphaned_media(grace_period)` methods
- `server/src/storage/sqlite/messages.rs` — implement retention-related queries for SQLite
- `server/src/storage/sqlite/media.rs` — implement orphaned media detection and cleanup for SQLite
- `server/src/storage/postgres/messages.rs` — same for PostgreSQL (if available)
- `server/src/storage/postgres/media.rs` — same for PostgreSQL (if available)
- `server/src/routes/admin.rs` — add retention policy endpoints
- `server/src/routes/channels.rs` — add channel retention override endpoints
- `server/src/config.rs` — parse `[retention]` config section

**Client (`client/src/`):**

- `client/src/components/admin/RetentionSettings.tsx` — new component: global retention policy editor with policy type selector and value input
- `client/src/components/channel/ChannelRetention.tsx` — new component: per-channel retention override in channel settings
- `client/src/hooks/useRetention.ts` — new hook: fetch and update retention policies

## Task List

### Phase A: Retention Policy Model
- [ ] Create `server/src/retention/policy.rs` — `RetentionPolicy` enum (`Forever`, `Days(u32)`, `Count(u32)`) with serialization and config parsing
- [ ] Add `[retention]` section parsing to `server/src/config.rs`
- [ ] Create `channel_retention_overrides` table migration for SQLite (and PostgreSQL if available)
- [ ] Add `deleted_at` column to messages table if not already present
- [ ] Add `orphaned_at` column to media table

### Phase B: Purge Logic
- [ ] Extend `MessageStore` trait with `soft_delete_expired()` and `hard_delete_old_soft_deleted()` methods
- [ ] Implement `soft_delete_expired()` for SQLite — for `days` policy: `WHERE created_at < NOW() - N days AND pinned = false AND deleted_at IS NULL`; for `count` policy: soft-delete all but the N most recent per channel
- [ ] Implement `hard_delete_old_soft_deleted()` for SQLite — `WHERE deleted_at < NOW() - grace_period`
- [ ] Extend `MediaStore` trait with `mark_orphaned_media()` and `delete_orphaned_media()` methods
- [ ] Implement orphaned media detection for SQLite — find media hashes not referenced by any active (non-deleted) message
- [ ] Implement `delete_orphaned_media()` for SQLite — delete the metadata record and the file/S3 object
- [ ] Implement all the above for PostgreSQL (if available)
- [ ] Create `server/src/retention/purge.rs` — orchestrates the full purge cycle: resolve effective policy per channel, soft-delete expired, hard-delete after grace, clean orphaned media

### Phase C: Background Scheduler
- [ ] Create `server/src/retention/scheduler.rs` — tokio `interval` task that runs the purge cycle on the configured interval
- [ ] Create `server/src/retention/mod.rs` — `RetentionManager` that starts the scheduler and provides `trigger_immediate_purge()`
- [ ] Integrate `RetentionManager` into the server startup (spawn the background task)
- [ ] Add logging for each purge cycle (messages soft-deleted, hard-deleted, media cleaned, duration)

### Phase D: Admin API
- [ ] Add global retention policy endpoints to `server/src/routes/admin.rs`
- [ ] Add channel retention override endpoints to `server/src/routes/channels.rs`
- [ ] Trigger an immediate purge when the retention policy is changed
- [ ] Add admin permission check on retention endpoints

### Phase E: Client UI
- [ ] Create `client/src/hooks/useRetention.ts` — fetch and update retention policies
- [ ] Create `client/src/components/admin/RetentionSettings.tsx` — global retention policy editor
- [ ] Create `client/src/components/channel/ChannelRetention.tsx` — channel-specific retention override in channel settings
- [ ] Add retention settings to the admin panel and channel settings UI

## Test List
- [ ] Unit test: `RetentionPolicy` parsing from TOML config produces correct enum values
- [ ] Unit test: `soft_delete_expired()` with `days` policy soft-deletes messages older than N days
- [ ] Unit test: `soft_delete_expired()` with `count` policy soft-deletes all but the N most recent messages per channel
- [ ] Unit test: `soft_delete_expired()` does not soft-delete pinned messages
- [ ] Unit test: `hard_delete_old_soft_deleted()` permanently deletes messages soft-deleted longer than the grace period
- [ ] Unit test: `hard_delete_old_soft_deleted()` does not delete messages soft-deleted within the grace period
- [ ] Unit test: `mark_orphaned_media()` correctly identifies media not referenced by any active message
- [ ] Unit test: `delete_orphaned_media()` removes the metadata record and the backing file/object
- [ ] Unit test: channel retention override takes precedence over the global policy
- [ ] Unit test: channel with no override uses the global policy
- [ ] Integration test: full purge cycle — create messages, set a `days: 1` policy, advance time, run purge, verify messages are soft-deleted
- [ ] Integration test: grace period — soft-deleted messages are not hard-deleted until the grace period expires
- [ ] Integration test: media cleanup — upload media, delete the only message referencing it, run purge, verify media is removed
- [ ] Integration test: changing retention policy triggers an immediate purge
- [ ] Integration test: retention endpoints require admin permission
- [ ] Manual test: set a `count: 100` policy on a channel with 200 messages, verify the oldest 100 are purged after the next cycle

## Open Questions
- **DM retention:** Should retention policies apply to direct messages, or only to channel messages? DMs may have different privacy expectations.
- **Attachment-only cleanup:** Should there be a separate policy for media/attachments (e.g., keep messages forever but delete attachments after 30 days)?
- **User notification:** Should users be notified when messages are about to be purged? For example, a channel message saying "Messages older than 90 days will be deleted."
- **Export before purge:** Should the purge job optionally create a backup of messages before deleting them? This would be useful for compliance but adds complexity and storage cost.
- **Manual policy:** The storage doc defines a `manual` policy where admins manage retention by hand. Is this just `forever` with the expectation that admins delete messages manually, or does it need dedicated tooling?
