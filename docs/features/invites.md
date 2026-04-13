# Feature: Invites

## Overview
Invite links allow server members to bring new users into the community. Invites can be single-use or multi-use, time-limited or permanent, and can optionally grant a role on join. This is the primary mechanism for growing a server's membership and is required by the `invite_only` membership mode.

## Background
The server model design doc (`docs/design/server-model.md`) defines invite links as the primary discovery and joining mechanism. Invite links encode the server address and an invite token. They can be single-use, multi-use, time-limited, channel-specific, or role-granting. This feature depends on roles (#11) being implemented so that role-granting invites work and the `manage_invites` permission flag is available.

## Requirements
- [ ] Authenticated users with the `manage_invites` permission can generate invite links
- [ ] Invite links contain the server's public address and a unique invite code
- [ ] Invites support configuration: max uses (0 = unlimited), expiry duration, and an optional role to grant on join
- [ ] Joining via invite creates the user's membership record and assigns the default role (+ any invite-granted role)
- [ ] Single-use invites are consumed after one successful join
- [ ] Expired invites return a clear error when used
- [ ] Users with `manage_invites` can list all active invites and see usage counts
- [ ] Users with `manage_invites` can revoke (delete) any invite
- [ ] The invite code is a short, URL-safe string
- [ ] Joining via invite is the only join path when the server is in `invite_only` mode

## Design

### API / Interface Changes

**REST endpoints** (all under `/api/v1/`):

| Method | Path | Description | Required Permission |
|--------|------|-------------|---------------------|
| POST | `/invites` | Create a new invite | `manage_invites` |
| GET | `/invites` | List all active invites | `manage_invites` |
| GET | `/invites/:code` | Get invite details (public, no auth required) | None |
| DELETE | `/invites/:code` | Revoke an invite | `manage_invites` |
| POST | `/invites/:code/join` | Join the server using this invite | Authenticated (no membership required) |

**POST `/invites` request body:**
```json
{
  "max_uses": 10,
  "expires_in_seconds": 86400,
  "grant_role_id": "optional-role-id"
}
```

**POST `/invites/:code/join` response:**
```json
{
  "member": { "pubkey": "...", "roles": [...], "joined_at": "..." }
}
```

**GET `/invites/:code` response (public):**
```json
{
  "code": "abc123",
  "server_name": "My Server",
  "server_icon": "https://...",
  "member_count": 42,
  "expires_at": "2026-05-01T00:00:00Z"
}
```

**Gateway events:**
- `MEMBER_JOIN` — broadcast when a user joins via invite (includes member info and roles)

**Invite link format:**
```
https://<server-address>/invite/<code>
```
The client intercepts this URL scheme and initiates the join flow.

### Data Model Changes

**New table:**

```sql
CREATE TABLE invites (
    code           TEXT PRIMARY KEY,     -- short URL-safe string (e.g., 8 chars)
    created_by     TEXT NOT NULL REFERENCES users(id),
    grant_role_id  TEXT REFERENCES roles(id),
    max_uses       INTEGER NOT NULL DEFAULT 0,  -- 0 = unlimited
    use_count      INTEGER NOT NULL DEFAULT 0,
    expires_at     TEXT,                 -- nullable, ISO 8601
    created_at     TEXT NOT NULL
);
```

### Component Changes

**Server (`server/`):**

- `server/src/models/invite.rs` — Invite struct and creation parameters
- `server/src/store/invite_store.rs` — `InviteStore` trait added to the storage trait hierarchy
- `server/src/store/sqlite/invite_store.rs` — SQLite implementation
- `server/src/routes/invites.rs` — REST handlers for invite CRUD and join
- `server/src/utils/code.rs` — Utility to generate short, URL-safe invite codes (e.g., `nanoid` or `rand` with a Base62 alphabet)
- Modify `server/src/routes/mod.rs` — Register invite routes
- Modify `server/src/store/mod.rs` — Add `InviteStore` to storage trait

**Client (`client/`):**

- `client/src/api/invites.ts` — API client functions for invite endpoints
- `client/src/stores/invites.ts` — Zustand slice for invite management state
- `client/src/components/invites/CreateInviteDialog.tsx` — Modal for generating invites with options (max uses, expiry, role grant)
- `client/src/components/invites/InviteList.tsx` — Table of active invites with usage stats and revoke button
- `client/src/components/invites/JoinByInvite.tsx` — UI for joining a server via an invite link (shows server preview, join button)
- `client/src/hooks/useInviteLink.ts` — Hook to parse invite URLs and extract code + server address

**Database migrations:**

- `server/migrations/NNNN_create_invites.sql`

## Implementation Notes

- Invite model and `InviteStore` trait follow the existing module layout (`storage/models.rs`, `storage/traits.rs`, `storage/sqlite/invites.rs`).
- Code generation is in `server/src/invites/mod.rs` (`generate_code()` — 8 Base62 chars via `rand`).
- `consume_invite` is atomic: UPDATE with all validity conditions in the WHERE clause; a follow-up SELECT disambiguates NotFound vs exhausted/expired.
- In `invite_only` mode, user account creation is now permitted (so users can auth before using an invite); full membership gating is deferred to Feature 13.
- Expired invite cleanup runs as a background task every hour.

## Task List

### Server
- [ ] Define Invite model struct in `server/src/storage/models.rs`
- [ ] Implement invite code generation utility (8-char Base62 strings)
- [ ] Add `InviteStore` trait to the storage trait hierarchy
- [ ] Write SQLite migration to create `invites` table
- [ ] Implement `InviteStore` for the SQLite backend
- [ ] Implement POST `/invites` handler (create invite, permission check)
- [ ] Implement GET `/invites` handler (list all invites, permission check)
- [ ] Implement GET `/invites/:code` handler (public invite preview, returns server name and member count)
- [ ] Implement DELETE `/invites/:code` handler (revoke invite, permission check)
- [ ] Implement POST `/invites/:code/join` handler: validate invite (not expired, not exhausted), assign grant role, increment use_count, broadcast MEMBER_JOIN via gateway
- [ ] Enforce membership mode: in `invite_only` mode user creation is allowed; access gating deferred to Feature 13
- [ ] Add background cleanup for expired invites (hourly background task)

### Client
- [ ] Add invites API client functions
- [ ] Add invites Zustand store slice
- [ ] Build CreateInviteDialog with options for max uses, expiry, and role grant
- [ ] Build InviteList table with usage stats and revoke action
- [ ] Build JoinByInvite flow: parse invite URL, fetch preview, show join button
- [ ] Handle `decentcom://` or `https://` invite URL scheme in the Tauri app (deep link registration)
- [ ] Copy-to-clipboard button for generated invite links

## Test List
- [ ] Unit: invite code generation produces URL-safe strings of correct length
- [ ] Unit: invite expiry check correctly identifies expired invites (storage unit test)
- [ ] Integration: create invite, verify it appears in GET /invites
- [ ] Integration: join via valid invite, verify membership is created and roles assigned
- [ ] Integration: join via single-use invite, verify second join attempt fails
- [ ] Integration: join via expired invite returns appropriate error (storage unit test)
- [ ] Integration: join via invite with grant_role_id assigns the specified role
- [ ] Integration: revoke invite, verify it can no longer be used
- [ ] Integration: user without manage_invites permission cannot create or list invites
- [ ] Integration: GET /invites/:code returns server preview without auth
- [ ] Integration: in invite_only mode, direct join (without invite) is rejected (deferred to Feature 13)
- [ ] Manual: create invite dialog works, link is copyable, and join flow completes

## Open Questions
- Should invite codes be case-sensitive? Lowercase-only is easier to communicate verbally.
- Should there be a limit on the number of active invites per server?
- Should invites optionally target a specific channel (user lands in that channel after joining)?
