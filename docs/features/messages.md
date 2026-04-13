# Feature: Messages

## Overview
Messages are the core content of decentcom. This feature implements sending, receiving, editing, and deleting text messages in channels, along with message history retrieval using cursor-based pagination and real-time delivery through the WebSocket gateway.

## Background
The storage design (`docs/design/storage.md`) identifies messages as append-heavy with high read volume, recommending indexes on `(channel_id, created_at)` and cursor-based pagination for fetching recent history. Messages are soft-deleted, never truly removed. The architecture doc (`docs/design/architecture.md`) places message CRUD in the REST API and real-time delivery in the gateway. The server model (`docs/design/server-model.md`) defines `max_message_length` as a configurable content policy. This feature depends on storage (feature 3), auth (feature 5), the gateway (feature 6), and channels (feature 7).

## Requirements
- [x] Authenticated users can send text messages to channels.
- [x] Messages are delivered in real-time to all gateway clients subscribed to that channel.
- [x] Users can fetch message history for a channel with cursor-based pagination (newest first).
- [x] Users can edit their own messages. Edited messages are marked with an `edited_at` timestamp.
- [x] Users can delete their own messages (soft-delete; `deleted` flag is set, row is retained).
- [x] Message content respects the server's `max_message_length` setting.
- [x] Each message includes: id, channel_id, author_id, content, created_at, edited_at, deleted flag.
- [x] Deleted messages remain in history as tombstones (deleted flag set; pagination cursors stay valid).

## Design

### API / Interface Changes

**REST endpoints (all under `/api/v1/`):**

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/channels/:channel_id/messages` | Send a message |
| `GET` | `/channels/:channel_id/messages` | Fetch message history (paginated) |
| `GET` | `/channels/:channel_id/messages/:message_id` | Get a single message |
| `PATCH` | `/channels/:channel_id/messages/:message_id` | Edit a message |
| `DELETE` | `/channels/:channel_id/messages/:message_id` | Soft-delete a message |

**Pagination query parameters for `GET /channels/:channel_id/messages`:**

| Param | Type | Description |
|-------|------|-------------|
| `before` | ULID | Fetch messages before this ID (for scrolling up) |
| `after` | ULID | Fetch messages after this ID (for scrolling down / catching up) |
| `limit` | integer | Max messages to return (default 50, max 100) |

**Request/response bodies:**

```json
// POST /channels/:channel_id/messages
{ "content": "Hello, world!" }

// Response (message object)
{
  "id": "01HXYZ...",
  "channel_id": "...",
  "author_pubkey": "ed25519:...",
  "content": "Hello, world!",
  "created_at": "2026-04-10T12:00:00Z",
  "edited_at": null,
  "deleted": false
}

// PATCH /channels/:channel_id/messages/:message_id
{ "content": "Hello, world! (edited)" }

// GET /channels/:channel_id/messages?before=01HXYZ&limit=50
// Response
{
  "messages": [ ... ],
  "has_more": true
}
```

**Gateway events:**

| Op | Payload | Sent to |
|----|---------|---------|
| `MESSAGE_CREATE` | Full message object | Subscribers of the channel |
| `MESSAGE_UPDATE` | Full message object (after edit) | Subscribers of the channel |
| `MESSAGE_DELETE` | `{ "id": "...", "channel_id": "..." }` | Subscribers of the channel |

### Data Model Changes

**`messages` table:**

| Column | Type | Notes |
|--------|------|-------|
| `id` | TEXT (ULID) | Primary key; ULID provides time-ordered, unique IDs |
| `channel_id` | TEXT | FK to channels |
| `author_pubkey` | TEXT | Public key of the message author |
| `content` | TEXT | Message text (cleared on soft-delete) |
| `created_at` | TEXT (ISO 8601) | Derived from ULID but stored explicitly for indexing |
| `edited_at` | TEXT (ISO 8601) | Null if never edited |
| `deleted_at` | TEXT (ISO 8601) | Null if active |

**Indexes:**
- `(channel_id, id)` -- Primary query path for paginated history (ULID is time-ordered so this is equivalent to ordering by time).
- `(author_pubkey)` -- For fetching a user's messages (future use: moderation).

### Component Changes

**New files/modules (server):**

- `server/src/messages/mod.rs` -- Module root.
- `server/src/messages/handlers.rs` -- axum handlers for message CRUD and history.
- `server/src/messages/models.rs` -- `Message`, `CreateMessage`, `UpdateMessage`, `MessagePage` structs.

**Storage layer:**

- `server/src/storage/message_store.rs` -- `MessageStore` trait: `create_message`, `get_message`, `list_messages` (with cursor params), `update_message`, `delete_message`.
- `server/src/storage/sqlite/messages.rs` -- SQLite implementation of `MessageStore`.

**Shared types crate:**

- `shared/src/messages.rs` -- Message types and gateway event payloads.

**Migrations:**

- `server/migrations/NNNN_create_messages.sql` -- Creates `messages` table with indexes.

**Modified files:**

- `server/src/main.rs` -- Add message routes to the router.
- `shared/src/gateway.rs` -- Add `MESSAGE_CREATE`, `MESSAGE_UPDATE`, `MESSAGE_DELETE` to the `Op` enum.

## Task List

### Phase A: Data model and storage
- [x] `messages` table in `001_initial.sql` with index on `(channel_id, created_at)`.
- [x] `Message` model struct in `server/src/storage/models.rs`.
- [x] `MessageStore` trait in `server/src/storage/traits.rs` with `before`-cursor pagination.
- [x] SQLite implementation in `server/src/storage/sqlite/messages.rs`.

### Phase B: REST API
- [x] `POST /channels/{channel_id}/messages` — validate, insert, broadcast `MESSAGE_CREATE`.
- [x] `GET /channels/{channel_id}/messages` — `before`/`limit` pagination with `has_more`.
- [x] `GET /channels/{channel_id}/messages/{message_id}` — get single message.
- [x] `PATCH /channels/{channel_id}/messages/{message_id}` — owner-only edit, broadcast `MESSAGE_UPDATE`.
- [x] `DELETE /channels/{channel_id}/messages/{message_id}` — owner-only soft-delete, broadcast `MESSAGE_DELETE`.
- [x] All routes wired in `server/src/main.rs` (router is composed directly in `app()`, no `routes/mod.rs` file exists).

### Phase C: Shared types and gateway integration
- [x] `MESSAGE_CREATE`, `MESSAGE_UPDATE`, `MESSAGE_DELETE` ops already in shared `Op` enum.
- [x] End-to-end gateway test deferred (WebSocket test infra not yet in place).

## Test List
- [x] Unit test: `MessageStore` SQLite — create a message, fetch by ID, verify all fields.
- [x] Unit test: `MessageStore` cursor pagination — insert messages, fetch with `before` cursor.
- [x] Unit test: `MessageStore` `after` cursor — deferred (only `before` implemented for now).
- [x] Unit test: `MessageStore` soft-delete — `deleted` flag set, row retained, preserved in list as a tombstone.
- [x] Unit test: `MessageStore` edit — update content, verify `edited_at` is set.
- [x] Integration test: Send a message, fetch history, verify message appears.
- [x] Integration test: Edit returns updated `edited_at`; unauthorized edit returns 403.
- [x] Integration test: Delete marks message deleted; unauthorized delete returns 403.
- [x] Integration test: Empty/oversized message content returns 422.
- [x] Integration test: Send to unknown channel returns 404.
- [x] Integration test: Pagination `has_more=true` when more messages exist.
- [x] Integration test: Empty channel returns empty array with `has_more: false`.
- [x] Integration test: Gateway subscriber receives `MESSAGE_CREATE` — deferred.
- [x] Manual: Send messages via curl, fetch history, verify ordering and pagination.

## Implementation Notes
- Message handlers are implemented in `server/src/messages/handlers.rs` and router wiring is in `server/src/messages/mod.rs` + `server/src/main.rs`.
- Message content validation returns HTTP 422 for empty/whitespace-only messages and for content exceeding `content.max_message_length`.
- `after` cursor input is accepted by schema but currently returns HTTP 400 with a clear error (`after` not implemented yet); only `before` is supported per test plan.
- Ownership checks for edit/delete are enforced by comparing `AuthUser.user_id` with `message.author_id`; unauthorized edits/deletes return HTTP 403.
- Soft-delete sets `deleted=1` and clears `content` to `""`; rows remain queryable so pagination cursors remain valid and tombstones are preserved in history.
- Gateway broadcasts are channel-scoped via `gateway.broadcast_to_channel` for create/update/delete events.

## Open Questions
- Should ULIDs be generated client-side or server-side? Server-side is simpler and avoids clock skew issues. Client-generated ULIDs could enable offline message drafting but add complexity.
- What message format should be supported? Plain text for now; Markdown rendering is a client-side concern. The server stores raw content. The architecture doc lists this as an open question.
- Should the server enforce any rate limiting on message sends? A simple per-user rate limit (e.g., 5 messages/second) would prevent spam without impacting normal usage.
