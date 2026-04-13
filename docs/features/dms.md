# Feature: Direct Messages

## Overview
Direct messages (DMs) allow two users on the same server to have a private conversation outside of channels. DMs are delivered in real time via the WebSocket gateway, stored server-side, and appear in a dedicated DM list in the client. DM notifications alert users to new messages.

## Background
The overview design doc (`docs/design/overview.md`) lists DMs as a core milestone 2 feature. The server model doc (`docs/design/server-model.md`) mentions a `require_invite_for_dm` config option, indicating that DMs are a server-mediated feature — they flow through the server, not peer-to-peer.

DMs reuse the existing message infrastructure (storage, real-time delivery, pagination) but target a private two-user channel rather than a server channel. The channel type distinction (text channel vs. DM channel) must be introduced.

This feature depends on messages (#8), gateway (#6), membership (#13), and profiles (#14) from earlier phases.

## Requirements
- [ ] A user can initiate a DM with any other member of the same server
- [ ] A DM channel is a private channel between exactly two users; it is not visible to other members or admins
- [ ] DM channels are created on first message (lazy creation) or by explicit open action
- [ ] Messages in DM channels use the same data model and storage as channel messages
- [ ] DMs are delivered in real time via the gateway to both participants
- [ ] Each user has a DM list showing all their active DM conversations, ordered by most recent message
- [ ] Users can close (hide) a DM conversation from their list without deleting messages
- [ ] Closed DMs reappear if the other user sends a new message
- [ ] Unread DM indicators show in the client sidebar
- [ ] Server config option `require_invite_for_dm` controls whether DMs require mutual server membership (default: true)
- [ ] DM history supports the same cursor-based pagination as channel messages
- [ ] Users can delete their own messages in DMs

## Design

### API / Interface Changes

**REST endpoints** (all under `/api/v1/`):

| Method | Path | Description | Required Permission |
|--------|------|-------------|---------------------|
| POST | `/dms` | Open or create a DM channel with a user | Authenticated (member) |
| GET | `/dms` | List the authenticated user's DM channels | Authenticated (member) |
| GET | `/dms/:channel_id/messages` | Get DM message history (paginated) | Authenticated (DM participant) |
| POST | `/dms/:channel_id/messages` | Send a message in a DM | Authenticated (DM participant) |
| DELETE | `/dms/:channel_id/messages/:message_id` | Delete own message in a DM | Authenticated (message author) |
| DELETE | `/dms/:channel_id` | Close (hide) a DM from the user's list | Authenticated (DM participant) |

**POST `/dms` request body:**
```json
{
  "recipient_pubkey": "<base58>"
}
```

**POST `/dms` response:**
```json
{
  "channel_id": "...",
  "recipient": { "pubkey": "...", "display_name": "...", "avatar_hash": "..." }
}
```

**Gateway events:**
- `DM_CREATE` — a new DM channel was opened (sent to both participants)
- `DM_MESSAGE_CREATE` — new message in a DM channel
- `DM_MESSAGE_DELETE` — message deleted in a DM channel

**Gateway subscription:** On connect, the client automatically receives events for all DM channels the user participates in. No explicit subscribe needed.

### Data Model Changes

**Modified table:** Add a `channel_type` column to the existing `channels` table (or introduce it if not already present):

```sql
ALTER TABLE channels ADD COLUMN channel_type TEXT NOT NULL DEFAULT 'text';
-- channel_type: 'text' | 'category' | 'dm'
```

**New table for DM channel membership and visibility:**

```sql
CREATE TABLE dm_channels (
    channel_id   TEXT NOT NULL REFERENCES channels(id),
    user_id      TEXT NOT NULL REFERENCES users(id),
    peer_user_id TEXT NOT NULL REFERENCES users(id),
    is_open      BOOLEAN NOT NULL DEFAULT TRUE,  -- FALSE = hidden from DM list
    last_read_at TEXT,                            -- for unread tracking
    PRIMARY KEY (channel_id, user_id)
);

CREATE UNIQUE INDEX idx_dm_pair ON dm_channels (user_id, peer_user_id);
```

Each DM channel has two rows in `dm_channels` (one per participant), allowing independent `is_open` and `last_read_at` tracking.

### Component Changes

**Server (`server/`):**

- `server/src/models/dm.rs` — DmChannel struct, DM-specific types
- `server/src/store/dm_store.rs` — `DmStore` trait (create DM channel, list user's DMs, get DM by participants, update visibility, update last_read)
- `server/src/store/sqlite/dm_store.rs` — SQLite implementation
- `server/src/routes/dms.rs` — REST handlers for DM operations
- Modify `server/src/models/channel.rs` — Add `channel_type` field if not present
- Modify `server/src/gateway/mod.rs` — Auto-subscribe users to their DM channels on connect; route DM message events to correct participants
- Modify `server/src/config.rs` — Add `require_invite_for_dm` option

**Client (`client/`):**

- `client/src/api/dms.ts` — API client functions for DM endpoints
- `client/src/stores/dms.ts` — Zustand slice for DM channels, messages, and unread counts
- `client/src/components/dms/DmList.tsx` — Sidebar list of DM conversations with unread indicators, ordered by recency
- `client/src/components/dms/DmView.tsx` — Message view for a DM conversation (reuses message components)
- `client/src/components/dms/StartDm.tsx` — UI to initiate a DM (e.g., from member list context menu or a search dialog)
- Modify `client/src/components/layout/Sidebar.tsx` — Add DM section above or below channel list
- Modify `client/src/components/members/MemberContextMenu.tsx` — Add "Send Message" option to start a DM
- Modify `client/src/stores/gateway.ts` — Handle DM gateway events

**Database migrations:**

- `server/migrations/NNNN_add_channel_type.sql`
- `server/migrations/NNNN_create_dm_channels.sql`

## Task List

### Server
- [ ] Add `channel_type` to the channels model and migration (if not already present)
- [ ] Add `require_invite_for_dm` to server config
- [ ] Define DmChannel model struct in `server/src/storage/models.rs`
- [ ] Add `DmStore` trait to the storage trait hierarchy
- [ ] Write SQLite migrations for `dm_channels` table and unique index
- [ ] Implement `DmStore` for the SQLite backend
- [ ] Implement POST `/dms` handler: find existing DM channel between the two users or create one (create a channel with type 'dm' + two dm_channels rows); enforce `require_invite_for_dm` config
- [ ] Implement GET `/dms` handler: list user's open DM channels
- [ ] Implement GET `/dms/:channel_id/messages` handler: reuse message pagination, verify requester is a DM participant
- [ ] Implement POST `/dms/:channel_id/messages` handler: reuse message creation, verify requester is a participant, broadcast DM_MESSAGE_CREATE
- [ ] Implement DELETE `/dms/:channel_id/messages/:message_id` handler: verify author, soft-delete
- [ ] Implement DELETE `/dms/:channel_id` handler: set is_open=false for the requester (does not delete messages)
- [ ] Re-open closed DM when a new message arrives: set is_open=true for the recipient
- [ ] Auto-subscribe authenticated users to their DM channels on gateway connect
- [ ] Route DM_MESSAGE_CREATE events only to the two DM participants (via send_to_user)
- [ ] Broadcast DM_CREATE event when a new DM channel is created

### Client
- [ ] Add DMs API client functions
- [ ] Add DMs Zustand store slice (DM list, per-DM message cache, unread counts)
- [ ] Handle DM gateway events (DM_CREATE, DM_MESSAGE_CREATE, DM_MESSAGE_DELETE)
- [ ] Build DmList sidebar component with unread indicators and most-recent ordering
- [ ] Build DmView component reusing existing message rendering components
- [ ] Build StartDm UI (search/select user dialog)
- [ ] Add "Send Message" to MemberContextMenu to start a DM
- [ ] Add DM section to the main sidebar layout
- [ ] Track unread state: update last_read_at when user views a DM, show unread badge when new messages arrive

## Test List
- [ ] Unit: DM channel creation finds existing channel for the same pair of users
- [ ] Unit: DM channel visibility toggling (close/reopen)
- [ ] Integration: create DM via POST /dms, verify channel is returned
- [ ] Integration: send message in DM, verify it appears in GET /dms/:channel_id/messages
- [ ] Integration: DM_MESSAGE_CREATE event is delivered only to the two participants
- [ ] Integration: close DM, verify it no longer appears in GET /dms; send new message, verify it reappears
- [ ] Integration: non-participant cannot access DM messages (403)
- [ ] Integration: with require_invite_for_dm=true, non-members cannot initiate DMs (404 for unknown pubkey)
- [ ] Integration: DM message deletion works and broadcasts DM_MESSAGE_DELETE
- [ ] Integration: DM message history supports cursor-based pagination
- [ ] Manual: DM list shows conversations ordered by recency
- [ ] Manual: unread indicator appears when a new DM arrives
- [ ] Manual: "Send Message" from member context menu opens a DM
- [ ] Manual: DM conversation view looks and works like a channel message view

## Implementation Notes
- DmChannel model placed in `server/src/storage/models.rs` (not a separate file) to align with existing patterns.
- DM delivery uses `send_to_user` on the gateway rather than channel subscriptions, ensuring events reach both participants regardless of subscription state.
- Gateway auto-subscribes users to their DM channels at connect (via `list_all_dms`) for future channel-based subscription support.

## Open Questions
- Should DMs be end-to-end encrypted? The overview design doc lists E2EE for DMs as a future iteration. For now, DMs are stored in plaintext on the server, same as channel messages. This should be clearly communicated to users.
- Should group DMs (3+ users) be supported? Discord supports them. Deferring to a separate feature keeps this one focused.
- When a user is banned, should their DM history be preserved for the other participant, or removed entirely?
- Should DMs work across servers (for users who share multiple servers)? Currently scoped to per-server DMs. Cross-server DMs relate to federation and are out of scope.
