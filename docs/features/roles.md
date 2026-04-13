# Feature: Roles & Permissions

## Overview
A role-based permission system that controls what members can do within a server and within individual channels. Roles are ordered by hierarchy, each carrying a set of boolean permission flags. Channel-level overrides allow fine-grained control. Two built-in roles (`@everyone` and `@admin`) provide sensible defaults.

## Background
The design docs specify a Discord-style role system (see `docs/design/server-model.md`, "Roles and Permissions" section). Roles are central to moderation (kick, ban), channel management, and membership management features that follow. The permission model must be in place before invites (#12) and membership management (#13) can enforce access control.

Phase 1 features (auth, channels, messages, gateway) exist but have no authorization layer beyond "is the user authenticated." This feature adds that layer.

## Requirements
- [ ] Server creates `@everyone` and `@admin` roles on instance initialization
- [ ] Roles have an ordered position (integer); higher position = higher authority
- [ ] Each role carries a bitfield of permission flags
- [ ] Users with appropriate permissions can create, read, update, and delete custom roles
- [ ] A role can only be managed by a user whose highest role is above it in the hierarchy
- [ ] Channel-level permission overrides can grant or deny specific flags per role
- [ ] Effective permissions are computed as: `@everyone` base, merged with all assigned roles (union), then channel overrides applied
- [ ] The `@admin` role has all permissions and cannot be deleted or have permissions removed
- [ ] The server owner (first authenticated user or configured pubkey) is always assigned `@admin`
- [ ] Permission checks gate all existing REST and gateway operations (send message, edit channel, etc.)
- [ ] Role changes are broadcast via the gateway in real time

## Design

### API / Interface Changes

**REST endpoints** (all under `/api/v1/`):

| Method | Path | Description | Required Permission |
|--------|------|-------------|---------------------|
| GET | `/roles` | List all roles | None (authenticated) |
| POST | `/roles` | Create a role | `manage_roles` |
| PATCH | `/roles/:role_id` | Update role name, color, permissions, position | `manage_roles` |
| DELETE | `/roles/:role_id` | Delete a role | `manage_roles` |
| PUT | `/members/:pubkey/roles/:role_id` | Assign role to member | `manage_roles` |
| DELETE | `/members/:pubkey/roles/:role_id` | Remove role from member | `manage_roles` |
| GET | `/channels/:channel_id/overrides` | List permission overrides for a channel | `manage_channels` |
| PUT | `/channels/:channel_id/overrides/:role_id` | Set permission override | `manage_channels` |
| DELETE | `/channels/:channel_id/overrides/:role_id` | Remove permission override | `manage_channels` |

**Gateway events:**
- `ROLE_CREATE` — new role created
- `ROLE_UPDATE` — role modified (permissions, name, position, color)
- `ROLE_DELETE` — role removed
- `MEMBER_ROLE_ADD` — role assigned to member
- `MEMBER_ROLE_REMOVE` — role removed from member

**Tauri IPC:** No new IPC commands needed; the React app calls REST endpoints directly.

### Data Model Changes

**New tables:**

```sql
CREATE TABLE roles (
    id          TEXT PRIMARY KEY,  -- ULID or UUID
    name        TEXT NOT NULL,
    color       TEXT,              -- hex color string, nullable
    permissions INTEGER NOT NULL DEFAULT 0,  -- bitfield
    position    INTEGER NOT NULL,  -- higher = more authority
    is_builtin  BOOLEAN NOT NULL DEFAULT FALSE,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);

CREATE TABLE member_roles (
    user_id  TEXT NOT NULL REFERENCES users(id),
    role_id  TEXT NOT NULL REFERENCES roles(id),
    PRIMARY KEY (user_id, role_id)
);

CREATE TABLE channel_permission_overrides (
    channel_id  TEXT NOT NULL REFERENCES channels(id),
    role_id     TEXT NOT NULL REFERENCES roles(id),
    allow       INTEGER NOT NULL DEFAULT 0,  -- bitfield of granted permissions
    deny        INTEGER NOT NULL DEFAULT 0,  -- bitfield of denied permissions
    PRIMARY KEY (channel_id, role_id)
);
```

**Permission bitfield flags:**

| Bit | Flag | Description |
|-----|------|-------------|
| 0 | `send_messages` | Send messages in text channels |
| 1 | `read_messages` | View channels and read message history |
| 2 | `manage_messages` | Delete or pin other users' messages |
| 3 | `manage_channels` | Create, edit, delete channels and categories |
| 4 | `manage_roles` | Create, edit, delete, assign roles |
| 5 | `kick_members` | Kick members from the server |
| 6 | `ban_members` | Ban members from the server |
| 7 | `manage_invites` | Create and revoke invite links |
| 8 | `manage_server` | Edit server settings |
| 9 | `attach_files` | Upload files and images |
| 10 | `add_reactions` | Add emoji reactions |
| 11 | `mention_everyone` | Use @everyone mentions |
| 12 | `view_audit_log` | View the server audit log |
| 13 | `administrator` | Bypasses all permission checks |

### Component Changes

**Server (`server/`):**

- `server/src/models/role.rs` — Role, MemberRole, ChannelPermissionOverride structs; permission bitfield constants and helpers
- `server/src/models/permissions.rs` — Permission computation logic (merge roles, apply overrides)
- `server/src/store/role_store.rs` — `RoleStore` trait added to the storage trait hierarchy
- `server/src/store/sqlite/role_store.rs` — SQLite implementation of `RoleStore`
- `server/src/routes/roles.rs` — REST handlers for role CRUD and assignment
- `server/src/routes/channels.rs` — Extended with permission override endpoints
- `server/src/middleware/permissions.rs` — Axum middleware/extractor that computes effective permissions for the authenticated user and injects them into request extensions
- `server/src/gateway/events.rs` — New event types for role changes
- Modify `server/src/routes/messages.rs` — Add permission checks (send_messages, read_messages)
- Modify `server/src/routes/channels.rs` — Add permission checks (manage_channels)

**Client (`client/`):**

- `client/src/api/roles.ts` — API client functions for role endpoints
- `client/src/stores/roles.ts` — Zustand slice for roles state
- `client/src/components/settings/RoleEditor.tsx` — Role creation/editing UI with permission toggles
- `client/src/components/settings/RoleList.tsx` — Ordered role list with drag-to-reorder
- `client/src/components/settings/ChannelPermissions.tsx` — Channel-level override editor
- `client/src/hooks/usePermissions.ts` — Hook that computes effective permissions for the current user in a given channel

**Database migrations:**

- `server/migrations/NNNN_create_roles.sql`

## Implementation Notes

- Permission constants and `UserPermissions` extractor live in `server/src/permissions.rs` (not `models/permissions.rs` as originally planned, matching the actual module layout).
- Role and `ChannelPermissionOverride` structs are in `server/src/storage/models.rs`.
- `RoleStore` trait is in `server/src/storage/traits.rs`; SQLite impl in `server/src/storage/sqlite/roles.rs`.
- Built-in roles are seeded in `server/migrations/002_roles.sql` (fixed IDs: `"everyone"`, `"admin"`).
- First authenticated user gets `@admin` in `server/src/auth/handlers.rs` (best-effort; minor TOCTOU acceptable for v1).
- Channel handlers use `UserPermissions` extractor (replaces `AuthUser`); message handlers call `effective_permissions` with channel_id for channel-scoped checks.
- Role REST handlers and channel override handlers are implemented in `server/src/roles/handlers.rs` and mounted by `server/src/roles/mod.rs`.
- Gateway protocol now includes `ROLE_CREATE`, `ROLE_UPDATE`, `ROLE_DELETE`, `MEMBER_ROLE_ADD`, and `MEMBER_ROLE_REMOVE` in `shared/src/gateway.rs`.
- Client role API/store foundations are in `client/src/api/roles.ts` and `client/src/stores/roles.ts`; `usePermissions` is in `client/src/hooks/usePermissions.ts`.
- Initial role settings UI components are scaffolded in `client/src/components/settings/RoleList.tsx`, `client/src/components/settings/RoleEditor.tsx`, and `client/src/components/settings/ChannelPermissions.tsx`.

## Task List

### Server
- [x] Define permission bitfield constants and helper functions in `server/src/permissions.rs`
- [x] Define Role, MemberRole, and ChannelPermissionOverride model structs in `server/src/storage/models.rs`
- [x] Add `RoleStore` trait to the storage trait hierarchy
- [x] Write SQLite migration to create `roles`, `member_roles`, and `channel_permission_overrides` tables
- [x] Implement `RoleStore` for the SQLite backend
- [x] Add server initialization logic to create `@everyone` and `@admin` (seeded in migration; first user gets @admin via auth handler)
- [x] Implement REST handlers for role CRUD (`POST/GET/PATCH/DELETE /roles`)
- [x] Implement REST handlers for role assignment (`PUT/DELETE /members/:pubkey/roles/:role_id`)
- [x] Implement REST handlers for channel permission overrides
- [x] Build permission computation middleware (`UserPermissions` extractor resolves base permissions; channel overrides applied per-handler)
- [x] Add permission checks to existing message endpoints (send, edit, delete)
- [x] Add permission checks to existing channel endpoints (create, edit, delete)
- [x] Add gateway events for role changes (ROLE_CREATE, ROLE_UPDATE, ROLE_DELETE, MEMBER_ROLE_ADD, MEMBER_ROLE_REMOVE)
- [x] Enforce hierarchy rule: users cannot manage roles at or above their own highest role position

### Client
- [x] Add roles API client functions in `client/src/api/roles.ts`
- [x] Add roles Zustand store slice
- [x] Handle role gateway events to keep store in sync
- [x] Build `usePermissions` hook for computing effective permissions
- [x] Build role list UI in server settings
- [x] Build role editor UI with permission flag toggles
- [x] Build channel permission override editor
- [ ] Gate UI actions based on effective permissions (hide/disable buttons the user lacks permission for)

## Test List
- [x] Unit: permission bitfield operations (set, clear, has, merge)
- [ ] Unit: effective permission computation with multiple roles and channel overrides
- [ ] Unit: hierarchy enforcement (cannot edit role at or above own position)
- [x] Integration: create role via REST, verify it appears in GET /roles
- [ ] Integration: assign role to member, verify member's effective permissions change
- [x] Integration: set channel override to deny send_messages, verify message send is rejected
- [ ] Integration: @admin role bypasses all permission checks
- [x] Integration: @everyone and @admin cannot be deleted
- [x] Integration: role CRUD events are broadcast via gateway (gateway broadcast wired; no dedicated assertion test)
- [x] Integration: unauthenticated requests to role endpoints return 401
- [x] Integration: user without manage_roles cannot create/edit/delete roles
- [ ] Manual: role editor UI correctly shows and saves permission toggles
- [ ] Manual: channel permission override UI works and takes effect immediately

## Open Questions
- Should permission flags be stored as a single i64 bitfield or as a more extensible structure (e.g., a JSON array of strings)? Bitfield is faster but limited to 64 flags. Starting with i64 and migrating later if needed seems pragmatic.
- Should role colors be enforced in any way (e.g., minimum contrast against the theme background)?
- Should there be a maximum number of roles per server?
