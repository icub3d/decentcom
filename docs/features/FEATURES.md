# Feature List

Features are organized into phases. Each phase builds on the previous one. Within a phase, features are ordered by dependency — earlier features are prerequisites for later ones.

Use `/feature <name>` to create a feature document, then `/implement <name>` to build it.

## Phase 1: Foundation

The minimum to have a working server and client that can exchange messages.

| # | Feature | Name | Description | Status |
|---|---|---|---|---|
| 1 | Project Scaffolding | `scaffolding` | Cargo workspace for server, Tauri v2 + React + TypeScript client, shared types crate, build tooling | ✅ |
| 2 | Server Configuration | `server-config` | TOML config file loading, server identity, feature flags, bind address, storage backend selection | ✅ |
| 3 | Storage Layer | `storage` | Storage trait hierarchy (UserStore, MessageStore, ChannelStore, SessionStore), SQLite backend implementation | ✅ |
| 4 | Identity & Key Generation | `identity` | Ed25519 master key pair generation, seed phrase (BIP39), secure key storage via Tauri (OS keychain), public key display format | ✅ |
| 5 | Authentication | `auth` | Challenge-response protocol (POST /auth/challenge, POST /auth/verify), session token issuance and validation, server-side user registration on first auth | ✅ |
| 6 | WebSocket Gateway | `gateway` | Persistent WSS connection, event envelope format, subscribe/unsubscribe to channels, session auth on connect, heartbeat/keepalive | ✅ |
| 7 | Channels | `channels` | Create/list/edit/delete text channels and categories, channel ordering, REST API endpoints, real-time channel events via gateway | ✅ |
| 8 | Messages | `messages` | Send/receive/edit/delete messages in channels, message history with cursor-based pagination, real-time delivery via gateway | ✅ |

| 9 | Client Shell | `client-shell` | React app layout: server sidebar, channel list, message view, message input. Zustand state management. Connect to one server, authenticate, send and receive messages | ✅ |
 10 | Theming | `theming` | Catppuccin theme integration (Mocha default), theme switcher (Latte, Frappe, Macchiato, Mocha), user preference persistence, Tailwind CSS config |

## Phase 2: Core UX

The features that make it feel like a real communication platform.

| # | Feature | Name | Description | Status |
|---|---|---|---|---|
| 11 | Roles & Permissions | `roles` | Role CRUD, ordered role hierarchy, permission flags (send messages, manage channels, kick, ban, etc.), channel-level permission overrides, @everyone and @admin built-in roles |
| 12 | Invites | `invites` | Generate invite links (single/multi-use, time-limited, role-granting), join via invite link, invite management API |
| 13 | Membership Management | `membership` | Membership modes (open, invite_only, allowlist, closed), kick, ban (by pubkey), unban, member list with roles |
| 14 | User Profiles | `profiles` | Per-server display name and avatar, avatar upload and storage, profile view UI |
| 15 | Device Sub-Keys | `device-keys` | Device key generation, delegation certificates signed by master key, server-side delegation chain verification, device listing and revocation |
| 16 | Direct Messages | `dms` | DM channels between two users, DM list in client, real-time delivery, DM notifications |
| 17 | File Uploads | `file-uploads` | Upload files/images to channels, content-addressable media storage, size/type limits from server config, inline image preview in client | ✅ server |
| 18 | Typing Indicators | `typing` | Typing event broadcast via gateway, client-side typing UI, debounce/timeout logic | ✅ server |
| 19 | Message Reactions | `reactions` | Add/remove emoji reactions to messages, reaction counts, real-time reaction events | ✅ server |
| 20 | Message Search | `search` | Full-text message search (FTS5 for SQLite), search API endpoint, search UI in client, configurable per server feature flag | ✅ server |
| 21 | Message Threads | `threads` | Reply threads on messages, thread view in client, thread notification, unread tracking | ✅ server |

## Phase 3: Media & Voice

Real-time audio/video communication.

| # | Feature | Name | Description | Status |
|---|---|---|---|---|
| 22 | Voice Channels | `voice` | WebRTC SFU integration, voice channel join/leave, audio streaming, mute/deafen, participant list | ✅ server (SFU/media deferred) |
| 23 | Video Chat | `video` | Camera stream alongside voice, video toggle, participant video grid | ✅ server (SFU/media deferred) |
| 24 | Screen Sharing | `screen-share` | Screen/window share stream, presenter mode, screen share as an additional stream in voice channels | ✅ server (SFU/media deferred) |

## Phase 4: Operations (Future)

Features for production deployment and scale.

| # | Feature | Name | Description | Status |
|---|---|---|---|---|
| 25 | PostgreSQL Backend | `postgres` | PostgreSQL storage backend implementation, S3-compatible media storage, migration tooling between SQLite and PostgreSQL |
| 26 | Key Recovery | `key-recovery` | Encrypted key backup (passphrase-protected export to file/cloud), key import on new device |
| 27 | Key Rotation | `key-rotation` | Rotation declaration signed by old key, server-side identity migration, revocation list |
| 28 | Backup & Restore | `backup` | Full server data export/import, portable archive format |
| 29 | Audit Log | `audit-log` | Append-only log of admin/mod actions, audit log API, audit log viewer in client | ✅ server |
| 30 | Message Retention | `retention` | Configurable retention policies (forever, days, count), background purge job, media cleanup |

## Notes

- Features marked "Future" in the design docs (federation, managed hosting, Kubernetes) are not listed here. They will get their own planning phase.
- Each feature should be small enough to implement in a single session. If a feature grows too large during `/feature` planning, split it.
- Server-side features should be buildable and testable before the client-side UI for the same feature.
