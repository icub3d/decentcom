# Feature: Message Threads

## Overview
Allow users to start threaded conversations on any message. Threads provide focused sub-conversations without cluttering the main channel timeline. Thread replies are visible in a dedicated thread view panel, with unread tracking and notifications so users can follow threads they care about.

## Background
The server model (`docs/design/server-model.md`) defines `message_threads` as a feature flag (enabled by default). The existing message model supports flat channel-based messaging; threads add a parent-child relationship between messages. The gateway (`docs/design/architecture.md`) already handles real-time message delivery — thread messages are delivered as events scoped to the thread. Unread tracking builds on the message delivery infrastructure.

## Requirements
- [ ] Any message in a text channel can become the root of a thread
- [ ] Thread replies are messages linked to a parent message by `thread_id`
- [ ] The main channel timeline shows a "N replies" indicator on messages that have threads
- [ ] Clicking a thread indicator opens a thread view panel alongside the channel
- [ ] Thread messages are delivered in real time to users who have the thread open or are following it
- [ ] Users who reply to a thread automatically follow it
- [ ] Users can manually follow/unfollow a thread
- [ ] Unread thread replies are tracked per user, with visual indicators in the channel
- [ ] The `message_threads` feature flag gates thread creation; when disabled, thread endpoints return 403
- [ ] Thread replies support all features of regular messages: attachments, reactions, editing, deletion
- [ ] Thread messages appear in search results with thread context
- [ ] Threads can be created in both text channels and DM channels

## Design

### API / Interface Changes

**New REST endpoints:**

`POST /api/v1/channels/{channel_id}/messages/{message_id}/threads`
- Creates (or retrieves existing) a thread on the target message.
- The parent message becomes the thread root.
- Response: `201 Created` with `{ thread_id, parent_message_id, channel_id, created_at, reply_count, follower_count }`

`GET /api/v1/threads/{thread_id}/messages`
- Returns messages in the thread with cursor-based pagination (same format as channel message history).
- Query params: `limit`, `before`, `after` (cursor-based).
- Response: `200 OK` with `{ messages: [...], cursor }`

`POST /api/v1/threads/{thread_id}/messages`
- Send a message to the thread. Same body as channel message creation (`content`, `attachment_ids`).
- Automatically follows the thread for the sender.
- Response: `201 Created` with the message object.

`PUT /api/v1/threads/{thread_id}/follow`
- Follow a thread (receive notifications for new replies).
- Response: `200 OK`

`DELETE /api/v1/threads/{thread_id}/follow`
- Unfollow a thread.
- Response: `204 No Content`

`GET /api/v1/threads/{thread_id}`
- Get thread metadata: reply count, follower count, last reply timestamp, participants.
- Response: `200 OK`

**Modified endpoints:**

`GET /api/v1/channels/{channel_id}/messages` — Message objects now include optional thread metadata:
```json
{
  "id": "...",
  "content": "...",
  "thread": {
    "thread_id": "...",
    "reply_count": 5,
    "last_reply_at": "2026-04-10T12:00:00Z",
    "last_reply_by": { "id": "...", "display_name": "Bob" }
  }
}
```

**New gateway events:**

```json
{ "op": "THREAD_CREATE", "d": { "thread_id": "...", "parent_message_id": "...", "channel_id": "...", "creator_id": "..." } }
{ "op": "THREAD_MESSAGE_CREATE", "d": { "thread_id": "...", "message": { /* message object */ } } }
{ "op": "THREAD_UPDATE", "d": { "thread_id": "...", "reply_count": 6, "last_reply_at": "..." } }
```

`THREAD_MESSAGE_CREATE` is sent to users who are subscribed to the thread (have it open or are following it). `THREAD_UPDATE` is sent to all users in the parent channel so the reply count indicator stays current.

### Data Model Changes

**New table: `threads`**

```sql
CREATE TABLE threads (
    id                TEXT PRIMARY KEY,    -- UUID
    channel_id        TEXT NOT NULL REFERENCES channels(id),
    parent_message_id TEXT NOT NULL UNIQUE REFERENCES messages(id),
    creator_id        TEXT NOT NULL REFERENCES users(id),
    created_at        TEXT NOT NULL DEFAULT (datetime('now')),
    last_reply_at     TEXT,
    reply_count       INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_threads_channel ON threads(channel_id);
CREATE INDEX idx_threads_parent ON threads(parent_message_id);
```

**New table: `thread_followers`**

```sql
CREATE TABLE thread_followers (
    thread_id       TEXT NOT NULL REFERENCES threads(id),
    user_id         TEXT NOT NULL REFERENCES users(id),
    last_read_at    TEXT,                 -- timestamp of last read message in thread
    PRIMARY KEY (thread_id, user_id)
);
```

**Modified table: `messages`**

Add a nullable `thread_id` column:

```sql
ALTER TABLE messages ADD COLUMN thread_id TEXT REFERENCES threads(id);
CREATE INDEX idx_messages_thread ON messages(thread_id);
```

Messages with a non-null `thread_id` are thread replies. Messages with a null `thread_id` are regular channel messages.

### Component Changes

**Server (`server/`):**
- `server/src/models/thread.rs` — New module: `Thread` struct, `ThreadFollower` struct, CRUD operations, reply count updates, follower management
- `server/src/routes/threads.rs` — New module: thread creation, message listing, message posting, follow/unfollow, thread metadata endpoints
- `server/src/routes/messages.rs` — Modify message serialization to include thread summary (reply count, last reply) when a thread exists
- `server/src/gateway/events.rs` — Add `THREAD_CREATE`, `THREAD_MESSAGE_CREATE`, `THREAD_UPDATE` event types
- `server/src/gateway/handler.rs` — Manage thread subscriptions: users who open a thread subscribe to its events; thread followers receive `THREAD_MESSAGE_CREATE` even if they don't have the thread panel open
- `server/src/storage/sqlite/migrations/` — Migrations for `threads`, `thread_followers` tables and `messages.thread_id` column

**Client (`client/`):**
- `client/src/components/ThreadPanel.tsx` — New component: slide-in panel showing the parent message at the top, thread replies below, message input at the bottom
- `client/src/components/ThreadIndicator.tsx` — New component: shows "N replies" badge on messages that have threads, with participant avatars and last reply time
- `client/src/components/Message.tsx` — Add `ThreadIndicator` rendering, "Reply in thread" button on message hover
- `client/src/stores/threadStore.ts` — New store: active thread state, thread message list, follow state, unread counts per thread
- `client/src/api/threads.ts` — API client functions for thread CRUD, follow/unfollow, message fetching
- `client/src/gateway/handlers.ts` — Add handlers for `THREAD_CREATE`, `THREAD_MESSAGE_CREATE`, `THREAD_UPDATE` events
- `client/src/components/ChannelView.tsx` — Layout adjustment: when a thread is open, render `ThreadPanel` alongside the main message list (split view)

## Task List

### Server
- [ ] Add `threads` and `thread_followers` tables and `messages.thread_id` column to SQLite migrations (`server/migrations/010_threads.sql`)
- [ ] Implement `Thread`, `ThreadFollower`, `ThreadSummary` models in `server/src/storage/models.rs`
- [ ] `ThreadStore` trait and SQLite impl in `server/src/storage/sqlite/threads.rs`
- [ ] `POST /api/v1/channels/{channel_id}/messages/{message_id}/threads` — create thread, idempotent, broadcasts `THREAD_CREATE`
- [ ] `GET /api/v1/threads/{thread_id}/messages` with cursor-based pagination (oldest-first)
- [ ] `POST /api/v1/threads/{thread_id}/messages` — create reply, increments reply_count, auto-follows sender, broadcasts `THREAD_MESSAGE_CREATE` and `THREAD_UPDATE`
- [ ] `PUT/DELETE /api/v1/threads/{thread_id}/follow` for follow/unfollow
- [ ] `GET /api/v1/threads/{thread_id}` for thread metadata
- [ ] `THREAD_CREATE`, `THREAD_MESSAGE_CREATE`, `THREAD_UPDATE` added to gateway Op enum
- [ ] `THREAD_UPDATE` broadcast to channel subscribers when reply count changes
- [ ] Thread summaries enriched onto root messages in channel message list via `enrich_messages`
- [ ] Thread creation gated behind `message_threads` feature flag

### Client
- [ ] Add thread API client in `client/src/api/threads.ts`
- [ ] Create `threadStore.ts` with active thread ID, thread messages, follow state, and per-thread unread counts
- [ ] Add gateway handlers for `THREAD_CREATE`, `THREAD_MESSAGE_CREATE`, `THREAD_UPDATE`
- [ ] Create `ThreadIndicator.tsx` — reply count badge with participant avatars, clickable to open thread
- [ ] Create `ThreadPanel.tsx` — parent message header, scrollable reply list, message input, follow/unfollow toggle
- [ ] Add "Reply in thread" action to message hover menu in `Message.tsx`
- [ ] Integrate `ThreadIndicator` into `Message.tsx` for messages with threads
- [ ] Adjust `ChannelView.tsx` layout for side-by-side thread panel (resizable split)
- [ ] Implement unread thread indicators: bold thread indicator when there are unread replies
- [ ] Update `last_read_at` on thread follower record when user views the thread
- [ ] Style thread panel with Catppuccin theme, visually distinct from main channel

## Test List
- [ ] Unit test: creating a thread sets the parent message and initializes reply count to 0
- [ ] Unit test: posting to a thread increments reply count and updates `last_reply_at`
- [ ] Unit test: create_thread is idempotent
- [ ] Unit test: follow/unfollow correctly adds/removes follower records; idempotent follow
- [ ] Unit test: `get_thread_summaries` returns summaries for parent message IDs
- [ ] Integration test: create thread via REST, verify thread metadata in response (`routes/thread_tests.rs`)
- [ ] Integration test: create thread is idempotent (same ID returned)
- [ ] Integration test: post messages to thread, fetch thread messages with pagination
- [ ] Integration test: reply increments reply_count visible in GET thread
- [ ] Integration test: thread summary appears in channel message fetch
- [ ] Integration test: follow/unfollow via REST returns correct status codes
- [ ] Integration test: reply auto-follows sender (unfollow returns 204)
- [ ] Integration test: thread endpoints return 403 when `message_threads` feature flag is disabled
- [ ] Integration test: thread message pagination (has_more = true)
- [ ] Integration test: `THREAD_CREATE` event is broadcast to channel subscribers
- [ ] Integration test: `THREAD_MESSAGE_CREATE` event is sent to thread followers
- [ ] Integration test: thread messages appear in search results
- [ ] Manual: click "Reply in thread" on a message, verify thread panel opens with parent message
- [ ] Manual: send a reply in the thread, verify it appears in real time for other thread viewers
- [ ] Manual: verify "N replies" indicator updates in the main channel after a thread reply
- [ ] Manual: follow a thread, close the panel, verify notification when new reply arrives

## Open Questions
- Should thread replies be visible in the main channel timeline as well (like Slack's "Also send to channel" option), or strictly only in the thread view?
- Should threads have a maximum depth (flat replies only, no nested threads), or allow threads on thread messages? Flat is simpler and matches Discord/Slack behavior.
- How should thread notifications interact with the overall unread count for a channel? Should unread thread replies count toward the channel's unread badge?
- Should there be a limit on the number of active threads per channel?
- How should deleted parent messages affect their threads? Options: delete the thread, orphan it, or show "[deleted message]" as the thread root.
