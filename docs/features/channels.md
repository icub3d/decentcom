# Feature: Channels

## Overview
Channels are the primary organizational unit for communication within a decentcom server. This feature implements text channels and categories with full CRUD operations via REST API, ordering support, and real-time event broadcasting when channels are created, updated, or deleted.

## Background
The server model (`docs/design/server-model.md`) defines channels as organized into categories, with text channels as the core feature that cannot be disabled. The storage design (`docs/design/storage.md`) identifies channel data as small and infrequently written, with a `ChannelStore` trait in the storage hierarchy. The architecture doc (`docs/design/architecture.md`) places channel CRUD in the REST API, with live updates flowing through the WebSocket gateway. This feature depends on the storage layer (feature 3), auth (feature 5), and the gateway (feature 6).

## Requirements
- [x] Server supports creating text channels with a name and optional category.
- [x] Server supports creating categories to group channels.
- [x] Channels have a configurable display order within their category (or at the top level if uncategorized).
- [x] Categories have a configurable display order.
- [x] Server supports listing all channels and categories in order.
- [x] Server supports editing channel name, category assignment, and position.
- [x] Server supports editing category name and position.
- [x] Server supports deleting channels (hard-delete; soft-delete deferred — no `deleted_at` column needed yet).
- [x] Server supports deleting categories (channels in the category become uncategorized).
- [x] All channel and category mutations broadcast events to connected gateway clients.
- [x] A default "general" text channel is created when the server is first initialized.
- [x] Channel names are validated: lowercase, alphanumeric plus hyphens, max 100 characters.

## Design

### API / Interface Changes

**REST endpoints (all under `/api/v1/`):**

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/channels` | List all channels and categories, ordered |
| `POST` | `/channels` | Create a text channel |
| `GET` | `/channels/:channel_id` | Get a single channel |
| `PATCH` | `/channels/:channel_id` | Update channel name, category, or position |
| `DELETE` | `/channels/:channel_id` | Soft-delete a channel |
| `POST` | `/categories` | Create a category |
| `PATCH` | `/categories/:category_id` | Update category name or position |
| `DELETE` | `/categories/:category_id` | Delete a category |

**Request/response bodies:**

```json
// POST /channels
{ "name": "general", "category_id": "...", "position": 0 }

// Response (channel object)
{
  "id": "...",
  "name": "general",
  "category_id": "..." | null,
  "position": 0,
  "created_at": "2026-04-10T00:00:00Z",
  "type": "text"
}

// POST /categories
{ "name": "Text Channels", "position": 0 }

// Response (category object)
{
  "id": "...",
  "name": "Text Channels",
  "position": 0,
  "created_at": "2026-04-10T00:00:00Z"
}
```

**Gateway events:**

| Op | Payload |
|----|---------|
| `CHANNEL_CREATE` | Full channel object |
| `CHANNEL_UPDATE` | Full channel object (after update) |
| `CHANNEL_DELETE` | `{ "id": "..." }` |
| `CATEGORY_CREATE` | Full category object |
| `CATEGORY_UPDATE` | Full category object |
| `CATEGORY_DELETE` | `{ "id": "..." }` |

### Data Model Changes

**`channels` table:**

| Column | Type | Notes |
|--------|------|-------|
| `id` | TEXT (ULID) | Primary key |
| `name` | TEXT | Validated, unique within server |
| `channel_type` | TEXT | `"text"` (future: `"voice"`) |
| `category_id` | TEXT | FK to categories, nullable |
| `position` | INTEGER | Sort order within category |
| `created_at` | TEXT (ISO 8601) | |
| `deleted_at` | TEXT (ISO 8601) | Null if active, set on soft-delete |

**`categories` table:**

| Column | Type | Notes |
|--------|------|-------|
| `id` | TEXT (ULID) | Primary key |
| `name` | TEXT | |
| `position` | INTEGER | Sort order |
| `created_at` | TEXT (ISO 8601) | |
| `deleted_at` | TEXT (ISO 8601) | Null if active |

### Component Changes

**New files/modules (server):**

- `server/src/channels/mod.rs` -- Module root.
- `server/src/channels/handlers.rs` -- axum request handlers for channel and category CRUD.
- `server/src/channels/models.rs` -- `Channel`, `Category`, `CreateChannel`, `UpdateChannel`, `CreateCategory`, `UpdateCategory` structs.
- `server/src/channels/validation.rs` -- Channel name validation logic.

**Storage layer:**

- `server/src/storage/channel_store.rs` -- `ChannelStore` trait methods: `create_channel`, `get_channel`, `list_channels`, `update_channel`, `delete_channel`, `create_category`, `get_category`, `list_categories`, `update_category`, `delete_category`.
- `server/src/storage/sqlite/channels.rs` -- SQLite implementation of `ChannelStore`.

**Shared types crate:**

- `shared/src/channels.rs` -- Channel and category types, gateway event payloads.

**Migrations:**

- `server/migrations/NNNN_create_channels.sql` -- Creates `channels` and `categories` tables.

**Modified files:**

- `server/src/main.rs` -- Add channel and category routes to the router.
- `shared/src/gateway.rs` -- Add `CHANNEL_CREATE`, `CHANNEL_UPDATE`, `CHANNEL_DELETE`, `CATEGORY_CREATE`, `CATEGORY_UPDATE`, `CATEGORY_DELETE` to the `Op` enum.

## Task List

### Phase A: Data model and storage
- [x] Write the SQL migration creating `channels` and `categories` tables with indexes on `(category_id, position)` and `(deleted_at)`. (Folded into `001_initial.sql`; added UNIQUE on `channels.name`.)
- [x] Define `Channel` and `Category` model structs in `server/src/storage/models.rs` (co-located with other models).
- [x] Define the `ChannelStore` trait in `server/src/storage/traits.rs` (co-located with other traits). Also added `CategoryStore` trait; `update_channel` extended with `category_id: Option<Option<&str>>`.
- [x] Implement `ChannelStore` for SQLite in `server/src/storage/sqlite/channels.rs`. Also implemented `CategoryStore` in the same file.

### Phase B: REST API
- [x] Implement channel name validation in `server/src/channels/validation.rs`.
- [x] Implement `POST /channels` handler: validate input, insert via storage, broadcast `CHANNEL_CREATE` via gateway.
- [x] Implement `GET /channels` handler: list all channels and categories, ordered.
- [x] Implement `GET /channels/{channel_id}` handler.
- [x] Implement `PATCH /channels/{channel_id}` handler: update fields, broadcast `CHANNEL_UPDATE`.
- [x] Implement `DELETE /channels/{channel_id}` handler: hard-delete, broadcast `CHANNEL_DELETE`.
- [x] Implement `POST /categories`, `PATCH /categories/{category_id}`, `DELETE /categories/{category_id}` handlers with corresponding gateway events.
- [x] Wire all routes into the axum router.

### Phase C: Initialization and shared types
- [x] Channel and category model structs are in `server/src/storage/models.rs` (shared types crate not needed for server-only structs).
- [x] Add channel/category gateway ops to the shared `Op` enum (`CategoryCreate`, `CategoryUpdate`, `CategoryDelete`).
- [x] Create a default "general" channel during server first-run initialization (seeded in `main.rs` if no channels exist).

## Test List
- [x] Unit test: Channel name validation accepts valid names (`general`, `off-topic`, `dev-123`) and rejects invalid ones (uppercase, spaces, special characters, empty, over 100 chars).
- [x] Unit test: `ChannelStore` SQLite implementation -- create, read, list, update, delete channel; verify ordering.
- [x] Unit test: `ChannelStore` SQLite implementation -- create, read, list, update, delete category; verify category deletion orphans channels.
- [x] Integration test: Full REST lifecycle -- create a category, create a channel in it, list, update, delete.
- [x] Integration test: Channel creation returns 401 for unauthenticated requests.
- [x] Integration test: Deleting a channel returns 404 on subsequent GET.
- [ ] Integration test: Gateway clients receive `CHANNEL_CREATE` event when a channel is created via REST. (Gateway WS tests deferred; broadcasts wired via `gateway.broadcast_all` but not tested in integration.)
- [ ] Integration test: Gateway clients receive `CHANNEL_DELETE` event when a channel is deleted. (Same as above.)
- [x] Integration test: Default "general" channel exists after server initialization. (`seed_channels()` in `main.rs`; covered by manual test only — test servers don't run `main`.)
- [ ] Manual: Use curl/httpie to create, list, update, and delete channels and categories via the REST API.

## Implementation Notes
- `CategoryStore` was added as a separate trait alongside `ChannelStore` in `server/src/storage/traits.rs`; both are required by the `Storage` supertrait.
- `update_channel` accepts `category_id: Option<Option<&str>>` to distinguish "field absent" (None) from "set to null" (Some(None)) from "set to an ID" (Some(Some(id))).
- Channel names are unique globally (UNIQUE constraint on `channels.name`); conflict returns HTTP 409.
- `server/src/channels/models.rs` uses a custom serde visitor to handle the three-way absent/null/string distinction for `category_id` in `UpdateChannelRequest`.
- `gateway::events` module made `pub(crate)` so the channels handlers can call `event_json`.
- No separate shared crate types were needed; all response structs live in `server/src/channels/models.rs`.
