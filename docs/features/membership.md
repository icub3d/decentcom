# Feature: Membership Management

## Overview
Membership management controls who is part of a server and how they got there. It implements the four membership modes (open, invite_only, allowlist, closed), provides kick and ban operations, and exposes a member list with role information. This is essential for server moderation and community management.

## Background
The server model design doc (`docs/design/server-model.md`) defines four membership modes and describes kick/ban operations. Bans are per-public-key. Since users are identified by Ed25519 keys, a banned user who generates a new key appears as a new account with no history, which is by design — operators can use `allowlist` or `invite_only` mode to mitigate ban evasion.

This feature depends on roles (#11) for permission checks (`kick_members`, `ban_members`) and invites (#12) for the `invite_only` join path.

## Requirements
- [x] Server configuration includes a `membership_mode` setting: `open`, `invite_only`, `allowlist`, `closed`
- [x] In `open` mode, any authenticated user can join by calling a join endpoint
- [x] In `invite_only` mode, joining requires a valid invite (handled by invites feature)
- [x] In `allowlist` mode, only pre-approved public keys can join; admins manage the allowlist
- [x] In `closed` mode, no new members can join
- [x] Users with `kick_members` permission can kick a member (removes membership, user can rejoin)
- [x] Users with `ban_members` permission can ban a member by pubkey (removes membership, blocks rejoin)
- [x] Users with `ban_members` permission can unban a pubkey
- [x] Kick and ban respect role hierarchy: cannot kick/ban a user with an equal or higher role
- [x] A member list endpoint returns all members with their roles and join date
- [x] Member join, leave, kick, and ban events are broadcast via the gateway
- [x] Banned pubkeys are persisted and checked on every join attempt

## Design

### API / Interface Changes

**REST endpoints** (all under `/api/v1/`):

| Method | Path | Description | Required Permission |
|--------|------|-------------|---------------------|
| POST | `/members/join` | Join the server (open mode) | Authenticated |
| DELETE | `/members/me` | Leave the server voluntarily | Authenticated (member) |
| GET | `/members` | List all members with roles | Authenticated (member) |
| GET | `/members/:pubkey` | Get a specific member's info | Authenticated (member) |
| POST | `/members/:pubkey/kick` | Kick a member | `kick_members` |
| POST | `/members/:pubkey/ban` | Ban a member by pubkey | `ban_members` |
| DELETE | `/bans/:pubkey` | Unban a pubkey | `ban_members` |
| GET | `/bans` | List all banned pubkeys | `ban_members` |
| GET | `/allowlist` | List allowed pubkeys (allowlist mode) | `manage_server` |
| POST | `/allowlist` | Add pubkey to allowlist | `manage_server` |
| DELETE | `/allowlist/:pubkey` | Remove pubkey from allowlist | `manage_server` |

**POST `/members/:pubkey/ban` request body:**
```json
{
  "reason": "optional reason string"
}
```

**Gateway events:**
- `MEMBER_JOIN` — user joined the server
- `MEMBER_LEAVE` — user left voluntarily
- `MEMBER_KICK` — user was kicked
- `MEMBER_BAN` — user was banned

### Data Model Changes

**Modified table:** The existing `users` table already tracks membership. A `members` concept may be a view or a separate table depending on Phase 1 implementation. Assuming a `members` table:

```sql
CREATE TABLE members (
    user_id    TEXT NOT NULL REFERENCES users(id),
    joined_at  TEXT NOT NULL,
    PRIMARY KEY (user_id)
);

CREATE TABLE bans (
    pubkey     TEXT PRIMARY KEY,
    banned_by  TEXT NOT NULL REFERENCES users(id),
    reason     TEXT,
    banned_at  TEXT NOT NULL
);

CREATE TABLE allowlist (
    pubkey     TEXT PRIMARY KEY,
    added_by   TEXT NOT NULL REFERENCES users(id),
    added_at   TEXT NOT NULL
);
```

**Server config addition** (in TOML):
```toml
[server]
membership_mode = "invite_only"  # open | invite_only | allowlist | closed
```

### Component Changes

**Server (`server/`):**

- `server/src/models/member.rs` — Member, Ban, AllowlistEntry structs
- `server/src/store/member_store.rs` — `MemberStore` trait (list members, add/remove membership, bans, allowlist)
- `server/src/store/sqlite/member_store.rs` — SQLite implementation
- `server/src/routes/members.rs` — REST handlers for membership operations
- `server/src/routes/bans.rs` — REST handlers for ban/unban/list bans
- `server/src/routes/allowlist.rs` — REST handlers for allowlist management
- Modify `server/src/config.rs` — Add `membership_mode` to server config
- Modify `server/src/middleware/permissions.rs` — Add membership check (user must be a member for most endpoints)
- Modify `server/src/gateway/events.rs` — New event types for membership changes

**Client (`client/`):**

- `client/src/api/members.ts` — API client functions for membership endpoints
- `client/src/stores/members.ts` — Zustand slice for member list and bans
- `client/src/components/members/MemberList.tsx` — Sidebar member list with role badges and online status
- `client/src/components/members/MemberContextMenu.tsx` — Right-click menu with kick/ban options (permission-gated)
- `client/src/components/settings/BanList.tsx` — View and manage banned users
- `client/src/components/settings/AllowlistEditor.tsx` — Manage allowlist entries (shown only in allowlist mode)
- `client/src/components/settings/MembershipSettings.tsx` — Toggle membership mode

**Database migrations:**

- `server/migrations/NNNN_create_members.sql`
- `server/migrations/NNNN_create_bans.sql`
- `server/migrations/NNNN_create_allowlist.sql`

## Task List

### Server
- [x] Add `membership_mode` to server config struct and TOML parsing
- [x] Define Member, Ban, and AllowlistEntry model structs
- [x] Add `MemberStore` trait to the storage trait hierarchy
- [x] Write SQLite migrations for `members`, `bans`, and `allowlist` tables
- [x] Implement `MemberStore` for the SQLite backend
- [x] Implement POST `/members/join` with membership mode enforcement (open: allow; invite_only: reject without invite; allowlist: check list; closed: reject)
- [x] Implement DELETE `/members/me` (voluntary leave)
- [x] Implement GET `/members` and GET `/members/:pubkey`
- [x] Implement POST `/members/:pubkey/kick` with permission and hierarchy checks
- [x] Implement POST `/members/:pubkey/ban` with permission and hierarchy checks; remove membership and add to bans table
- [x] Implement DELETE `/bans/:pubkey` (unban) and GET `/bans`
- [x] Implement allowlist CRUD endpoints (GET/POST/DELETE `/allowlist`)
- [x] Add ban check to all join paths (direct join, invite join): reject if pubkey is in bans table
- [x] Add membership middleware: most API endpoints require the user to be a current member (`MemberUser` extractor)
- [x] Broadcast MEMBER_JOIN, MEMBER_LEAVE, MEMBER_KICK, MEMBER_BAN events via gateway

### Client
- [x] Add members API client functions
- [x] Add members Zustand store slice (member list, bans, allowlist)
- [x] Handle membership gateway events to keep member list in sync
- [ ] Build MemberList sidebar component showing members grouped by role and online status
- [x] Build MemberContextMenu with kick/ban actions (gated by permissions)
- [x] Build BanList view in server settings
- [x] Build AllowlistEditor in server settings (visible only when mode is allowlist)
- [x] Build MembershipSettings to change membership mode

## Test List
- [ ] Unit: membership mode enforcement logic correctly allows/rejects joins
- [x] Unit: hierarchy check prevents kicking/banning users with equal or higher roles
- [x] Integration: join in open mode succeeds
- [x] Integration: join in invite_only mode without invite is rejected
- [x] Integration: join in allowlist mode with unlisted pubkey is rejected
- [x] Integration: join in allowlist mode with listed pubkey succeeds
- [x] Integration: join in closed mode is rejected (tested at auth layer via `registration_restricted_in_closed_mode`)
- [x] Integration: kick removes membership, user can rejoin
- [x] Integration: ban removes membership, rejoin is blocked
- [x] Integration: unban allows previously banned pubkey to rejoin
- [x] Integration: user without kick_members permission cannot kick
- [x] Integration: user without ban_members permission cannot ban
- [x] Integration: GET /members returns all members with roles
- [ ] Integration: membership change events are broadcast via gateway (events broadcast; WebSocket delivery tested in gateway tests)
- [ ] Manual: member list displays correctly with role badges
- [ ] Manual: right-click context menu shows kick/ban only for permitted users

## Implementation Notes

- The `allowlist` membership mode allows users to create accounts (authenticate) but gates membership at the join endpoint. Only `closed` mode fully blocks new account creation. This differs slightly from the original design which blocked registration for both `allowlist` and `closed` — the revised approach is simpler and lets users get a session token before they've been approved, which is needed for the allowlist flow.
- The first registered user (server owner) is always auto-joined as a member regardless of membership mode, so they can manage the server.
- The `MemberUser` extractor lives in `server/src/permissions.rs` and wraps `UserPermissions` with a membership check.
- `GET /members/:pubkey` looks up by public key (not user ID) since pubkeys are the user-visible identity.
- Runtime API support for changing `membership_mode` is not implemented yet; mode changes still come from server config and restart.

## Open Questions
- Should kicked users receive a notification with a reason, or just be disconnected?
- Should there be a "timeout" (temporary mute) feature as part of this feature, or as a separate feature?
- When membership mode changes (e.g., from open to closed), should existing members be affected?
