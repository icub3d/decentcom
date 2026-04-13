# Feature: Message Reactions

## Overview
Allow users to add and remove emoji reactions on messages. Reactions are lightweight social signals that reduce the need for short reply messages. Each reaction shows the emoji, the count of users who reacted, and whether the current user has reacted. Reaction changes are broadcast in real time via the gateway.

## Background
The server model (`docs/design/server-model.md`) defines `emoji_reactions` as a feature flag (enabled by default). Reactions are a core UX feature in Discord-style platforms. The gateway (`docs/design/architecture.md`) already handles real-time event delivery, so reaction events follow the same event envelope pattern.

## Requirements
- [ ] Users can add a Unicode emoji reaction to any message they can view
- [ ] Users can remove their own reaction from a message
- [ ] Each message displays its reactions with emoji, count, and whether the current user reacted
- [ ] The same user can only react once with a given emoji per message (adding again is a no-op)
- [ ] A user can react with multiple different emoji on the same message
- [ ] Reaction add/remove is broadcast in real time via the gateway
- [ ] The `emoji_reactions` feature flag gates reactions; when disabled, reaction endpoints return 403
- [ ] Users need `ADD_REACTIONS` permission to add reactions (removing own reactions is always allowed)
- [ ] Users with `MANAGE_MESSAGES` permission can remove any user's reaction
- [ ] Deleting a message deletes all its reactions

## Design

### API / Interface Changes

**New REST endpoints:**

`PUT /api/v1/channels/{channel_id}/messages/{message_id}/reactions/{emoji}`
- Adds the current user's reaction. `emoji` is a URL-encoded Unicode emoji.
- Idempotent: re-adding the same reaction is a no-op (200 OK).
- Validates: session, `ADD_REACTIONS` permission, `emoji_reactions` feature flag.
- Response: `200 OK`

`DELETE /api/v1/channels/{channel_id}/messages/{message_id}/reactions/{emoji}`
- Removes the current user's reaction.
- Response: `204 No Content`

`DELETE /api/v1/channels/{channel_id}/messages/{message_id}/reactions/{emoji}/{user_id}`
- Admin/mod endpoint: removes another user's reaction.
- Requires `MANAGE_MESSAGES` permission.
- Response: `204 No Content`

`GET /api/v1/channels/{channel_id}/messages/{message_id}/reactions/{emoji}`
- Returns the list of users who reacted with this emoji (paginated).
- Response: `200 OK` with `{ users: [{ id, display_name }], total: N }`

**Modified endpoint:**

`GET /api/v1/channels/{channel_id}/messages` — Message objects in the response now include a `reactions` array:
```json
"reactions": [
  { "emoji": "👍", "count": 3, "me": true },
  { "emoji": "❤️", "count": 1, "me": false }
]
```

**New gateway events:**

```json
{ "op": "REACTION_ADD", "d": { "channel_id": "...", "message_id": "...", "user_id": "...", "emoji": "👍" } }
{ "op": "REACTION_REMOVE", "d": { "channel_id": "...", "message_id": "...", "user_id": "...", "emoji": "👍" } }
```

### Data Model Changes

**New table: `reactions`**

```sql
CREATE TABLE reactions (
    message_id TEXT NOT NULL REFERENCES messages(id),
    user_id    TEXT NOT NULL REFERENCES users(id),
    emoji      TEXT NOT NULL,           -- Unicode emoji string
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (message_id, user_id, emoji)
);

CREATE INDEX idx_reactions_message ON reactions(message_id);
```

The composite primary key enforces one reaction per emoji per user per message.

### Component Changes

**Server (`server/`):**
- `server/src/models/reaction.rs` — New module: `Reaction` struct, DB operations (add, remove, list by message, counts with `me` flag)
- `server/src/routes/reactions.rs` — New module: PUT, DELETE, GET endpoints
- `server/src/routes/messages.rs` — Modify message serialization to include aggregated reaction data
- `server/src/gateway/events.rs` — Add `REACTION_ADD` and `REACTION_REMOVE` event types
- `server/src/gateway/handler.rs` — Broadcast reaction events to channel subscribers

**Client (`client/`):**
- `client/src/components/ReactionBar.tsx` — New component: horizontal row of reaction pills below a message, each showing emoji + count, highlighted if the current user reacted
- `client/src/components/ReactionPicker.tsx` — New component: emoji picker popover for adding reactions (can reuse a lightweight emoji picker library or a curated grid of common emoji)
- `client/src/components/Message.tsx` — Integrate `ReactionBar` below message content
- `client/src/api/reactions.ts` — API client functions for add/remove/list reactions
- `client/src/stores/messageStore.ts` — Update message reaction state on `REACTION_ADD` / `REACTION_REMOVE` events
- `client/src/gateway/handlers.ts` — Add handlers for reaction gateway events

## Task List

### Server
- [ ] Add `reactions` table to SQLite migrations (`server/migrations/008_reactions.sql`)
- [ ] Implement `ReactionStore` trait and SQLite impl in `server/src/storage/sqlite/reactions.rs`
- [ ] `REACTION_ADD` and `REACTION_REMOVE` added to gateway Op enum in `shared/src/lib.rs`
- [ ] Implement `PUT /api/v1/channels/{channel_id}/messages/{message_id}/reactions/{emoji}` with permission and feature flag checks
- [ ] Implement `DELETE` endpoint for self-removal and admin removal (`server/src/reactions/handlers.rs`)
- [ ] Implement `GET` endpoint to list users who reacted with a given emoji
- [ ] Message responses include aggregated `reactions` array with `me` field via `enrich_messages`
- [ ] `REACTION_ADD` / `REACTION_REMOVE` events broadcast via gateway on reaction changes
- [ ] `ADD_REACTIONS` permission flag added
- [ ] Reactions cascade-deleted when message is deleted (FK constraint in migration)

### Client
- [ ] Add reaction API client functions in `client/src/api/reactions.ts`
- [ ] Add `REACTION_ADD` / `REACTION_REMOVE` handlers in `client/src/gateway/handlers.ts`
- [ ] Update message store to track and update reaction state from gateway events
- [ ] Create `ReactionBar.tsx` component: render reaction pills, toggle own reaction on click
- [ ] Create `ReactionPicker.tsx` component: emoji selection popover triggered by a "+" button on message hover
- [ ] Integrate `ReactionBar` into `Message.tsx`
- [ ] Style reaction pills with Catppuccin theme colors (highlighted state for "me" reactions)

## Test List
- [ ] Unit test: adding a reaction creates the record, adding again is idempotent (`storage/sqlite/reactions.rs`)
- [ ] Unit test: removing a reaction deletes the record
- [ ] Unit test: aggregate query returns correct counts and `me` flag per emoji
- [ ] Integration test: add reaction via REST, verify it appears in message fetch (`routes/reaction_tests.rs`)
- [ ] Integration test: reaction idempotency (add twice, count stays 1)
- [ ] Integration test: remove reaction via REST, verify it disappears
- [ ] Integration test: reaction endpoints return 403 when `emoji_reactions` feature flag is disabled
- [ ] Integration test: add reaction triggers `REACTION_ADD` gateway event to other subscribers
- [ ] Integration test: user with `MANAGE_MESSAGES` can remove another user's reaction
- [ ] Manual: hover over a message, click the reaction button, pick an emoji, verify it appears
- [ ] Manual: click an existing reaction pill to toggle your own reaction on/off
- [ ] Manual: verify real-time reaction updates between two clients

## Open Questions
- Should we support custom emoji (server-uploaded images) in addition to Unicode emoji? This is a significant feature on its own and may warrant a separate feature doc.
- Should there be a limit on the number of distinct emoji reactions per message? Discord caps at 20.
- Should there be a limit on the number of users who can react with a single emoji per message? Probably not needed for initial release.
- Should the emoji picker be a full library (e.g., `emoji-mart`) or a minimal curated set for the initial release?
