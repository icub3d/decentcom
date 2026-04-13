# Feature: Audit Log

## Overview
The audit log is an append-only record of administrative and moderator actions on the server. Every privileged action (role changes, kicks, bans, channel modifications, permission updates, server setting changes) is logged with the actor, action, target, timestamp, and details. The log is queryable via a REST API and viewable in the client's admin panel.

## Background
The server-model doc (`docs/design/server-model.md`) lists an audit log for admin actions as a moderation feature and includes "View audit log" as an admin operation. The storage doc (`docs/design/storage.md`) categorizes audit logs as append-only data. The log should be tamper-resistant — entries are never modified or deleted through normal operations.

Depends on: `roles` (feature #11), `membership` (feature #13), all Phase 1 and Phase 2 features.

## Requirements
- [ ] All admin and moderator actions are logged automatically (no opt-in required)
- [ ] Each log entry records: actor (pubkey), action type, target (user/channel/role/etc.), timestamp, and a details payload
- [ ] The audit log is append-only — entries cannot be edited or deleted via the API
- [ ] The audit log API supports filtering by action type, actor, target, and time range
- [ ] The audit log API supports cursor-based pagination
- [ ] Only users with the `view_audit_log` permission can access the audit log
- [ ] The audit log viewer in the client displays entries in reverse chronological order with filtering controls
- [ ] Log entries are retained indefinitely by default (not subject to message retention policies)

## Design

### API / Interface Changes

**REST endpoints:**

| Method | Path | Description |
|---|---|---|
| GET | `/api/v1/admin/audit-log` | Query audit log entries with filters and pagination |
| GET | `/api/v1/admin/audit-log/{entry_id}` | Get a single audit log entry with full details |

**Query parameters for `GET /api/v1/admin/audit-log`:**

| Parameter | Type | Description |
|---|---|---|
| `action_type` | string | Filter by action type (e.g., `member_kick`, `role_update`) |
| `actor_id` | integer | Filter by the user who performed the action |
| `target_id` | string | Filter by the target entity ID |
| `before` | string (cursor) | Cursor for backward pagination |
| `after` | string (cursor) | Cursor for forward pagination |
| `limit` | integer | Max entries to return (default 50, max 100) |

**Response:**

```json
{
  "entries": [
    {
      "id": "12345",
      "action_type": "member_ban",
      "actor": { "id": 1, "pubkey": "...", "display_name": "Admin" },
      "target": { "type": "user", "id": 42, "pubkey": "...", "display_name": "BadUser" },
      "details": { "reason": "spam" },
      "created_at": "2026-04-10T12:00:00Z"
    }
  ],
  "cursor": { "before": "...", "after": "..." }
}
```

**Action types:**

| Action Type | Trigger |
|---|---|
| `member_kick` | User kicked from server |
| `member_ban` | User banned from server |
| `member_unban` | User unbanned |
| `role_create` | New role created |
| `role_update` | Role permissions or properties changed |
| `role_delete` | Role deleted |
| `role_assign` | Role assigned to a user |
| `role_revoke` | Role removed from a user |
| `channel_create` | Channel created |
| `channel_update` | Channel name, topic, or settings changed |
| `channel_delete` | Channel deleted |
| `channel_permission_update` | Channel permission override changed |
| `message_delete` | Message deleted by a moderator (not the author) |
| `message_pin` | Message pinned |
| `message_unpin` | Message unpinned |
| `invite_create` | Invite link created |
| `invite_delete` | Invite link revoked |
| `server_update` | Server settings changed |
| `key_rotation` | User's key was rotated |

### Data Model Changes

**New table: `audit_log`**

| Column | Type | Description |
|---|---|---|
| `id` | INTEGER/BIGSERIAL | Primary key, monotonically increasing |
| `action_type` | TEXT | Action type enum value |
| `actor_id` | INTEGER/BIGINT | FK to users (the user who performed the action) |
| `target_type` | TEXT | Type of target entity (`user`, `channel`, `role`, `message`, `invite`, `server`) |
| `target_id` | TEXT | ID of the target entity (TEXT to support different ID types) |
| `details` | TEXT (JSON) | JSON blob with action-specific details |
| `created_at` | TIMESTAMP | When the action occurred |

**Indexes:**
- `(action_type, created_at)` — for filtering by action type
- `(actor_id, created_at)` — for filtering by actor
- `(target_type, target_id, created_at)` — for filtering by target
- `(created_at)` — for pagination

### Component Changes

**Server (`server/`):**

- `server/src/models/audit.rs` — new model: `AuditLogEntry` struct, `ActionType` enum, serialization
- `server/src/storage/trait.rs` — extend with `AuditStore` trait: `append(entry)`, `query(filters, pagination)`, `get(id)`
- `server/src/storage/sqlite/audit.rs` — SQLite implementation of `AuditStore`
- `server/src/storage/postgres/audit.rs` — PostgreSQL implementation of `AuditStore` (if postgres feature is available)
- `server/src/audit.rs` — new module: `AuditLogger` service that is injected into route handlers; provides `log_action()` method
- `server/src/routes/admin.rs` — add audit log query endpoints
- Modify existing route handlers to call `AuditLogger::log_action()`:
  - `server/src/routes/members.rs` — kick, ban, unban
  - `server/src/routes/roles.rs` — create, update, delete, assign, revoke
  - `server/src/routes/channels.rs` — create, update, delete, permission overrides
  - `server/src/routes/messages.rs` — moderator delete, pin, unpin
  - `server/src/routes/invites.rs` — create, delete
  - `server/src/routes/settings.rs` — server config changes

**Client (`client/src/`):**

- `client/src/components/admin/AuditLog.tsx` — new component: audit log viewer with table layout, action type icons, actor/target links
- `client/src/components/admin/AuditLogFilters.tsx` — new component: filter controls (action type dropdown, actor search, date range)
- `client/src/components/admin/AuditLogEntry.tsx` — new component: expanded view of a single audit log entry with full details
- `client/src/hooks/useAuditLog.ts` — new hook: fetches audit log entries with filters and pagination
- `client/src/stores/adminStore.ts` — extend with audit log state (or new dedicated store)

## Task List

### Phase A: Data Model and Storage
- [ ] Create `audit_log` table migration for SQLite (`server/migrations/012_audit_log.sql`)
- [ ] Create `audit_log` table migration for PostgreSQL *(deferred)*
- [ ] `AuditLogEntry` struct added to `server/src/storage/models.rs`
- [ ] `AuditStore` trait + `AuditLogQuery` defined in `server/src/storage/traits.rs`
- [ ] SQLite implementation: `server/src/storage/sqlite/audit.rs`

### Phase B: Audit Logger Service
- [ ] `server/src/audit/mod.rs` — `log_action()` free function; errors are logged but never propagate
- [ ] Audit calls in membership routes: kick, ban, unban
- [ ] Audit calls in role routes: create, update, delete, assign, revoke
- [ ] Audit calls in channel routes: create, update, delete
- [ ] Audit calls for moderator message delete / pin / unpin *(message pinning not yet implemented)*
- [ ] Audit calls in invite routes: create, delete

### Phase C: Query API
- [ ] `GET /api/v1/admin/audit-log` with filters (`action_type`, `actor_id`, `target_id`, `before` cursor, `limit`)
- [ ] `GET /api/v1/admin/audit-log/{entry_id}` — single entry
- [ ] `VIEW_AUDIT_LOG` permission (bit 12) enforced on both endpoints

### Phase D: Client UI
- [ ] Create `client/src/hooks/useAuditLog.ts` — fetch, filter, and paginate audit log entries
- [ ] Create `client/src/components/admin/AuditLog.tsx` — main audit log viewer with infinite scroll or pagination
- [ ] Create `client/src/components/admin/AuditLogFilters.tsx` — filter controls
- [ ] Create `client/src/components/admin/AuditLogEntry.tsx` — detailed entry view
- [ ] Add audit log section to the admin panel navigation

## Test List
- [ ] Unit test: `AuditLogEntry` serialization/deserialization round-trips correctly
- [ ] Unit test: `append()` inserts an entry and it is retrievable via `get()`
- [ ] Unit test: `query()` with no filters returns entries in reverse chronological order
- [ ] Unit test: `query()` with `action_type` filter returns only matching entries
- [ ] Unit test: `query()` with `actor_id` filter returns only matching entries
- [ ] Unit test: `query()` with `target_id` filter returns only matching entries
- [ ] Unit test: `query()` pagination (cursor-based) returns correct pages
- [ ] Integration test: kicking a user creates a `member_kick` audit log entry with correct actor and target
- [ ] Integration test: banning a user creates a `member_ban` audit log entry
- [ ] Integration test: creating a channel creates a `channel_create` audit log entry
- [ ] Integration test: changing a role creates a `role_update` audit log entry
- [ ] Integration test: audit log endpoint returns 403 for users without `view_audit_log` permission
- [ ] Integration test: audit log entries cannot be deleted via any API endpoint
- [ ] Manual test: perform various admin actions and verify they appear in the audit log viewer with correct details

## Open Questions
- **Log retention:** Should audit log entries ever be pruned, or are they truly permanent? For very active servers, the audit log could grow indefinitely. Consider an optional max-age config.
- **Webhook integration:** The server-model doc mentions webhook support for logging mod actions to external services. Should the audit log emit webhooks on new entries? This could be a follow-up feature.
- **Bulk actions:** If an admin performs a bulk action (e.g., deleting all messages from a banned user), should that be one audit log entry or one per message? One entry is more practical but less granular.
- **Tamper resistance:** Should audit log entries be hash-chained (each entry includes the hash of the previous entry) for stronger tamper evidence? This adds complexity but improves trust for operators reviewing logs.
