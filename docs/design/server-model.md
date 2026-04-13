# Server Model

## What is a "Server"?

In decentcom, a "server" (borrowing Discord terminology) is a community space. It has:
- A name, icon, and description
- One or more text channels organized into categories
- Optional voice/video channels
- A role and permission system
- A membership list

A decentcom **server** (the community) runs on a decentcom **instance** (the software). One instance can host exactly one community. This 1:1 mapping keeps the software and operational model simple — no multi-tenancy at the instance level.

## Deployment Models

### Home Server
A server running on a home machine or local network. It may only be accessible via LAN or through a VPN. Suitable for small friend groups.

### Self-Hosted VPS / Cloud
A server running on a VPS, cloud VM, or container platform. Accessible from the internet. The operator is responsible for infrastructure, DNS, and TLS.

### Managed Hosting (Future)
decentcom will offer a managed hosting product where we provision and operate the infrastructure. Users get a subdomain, managed TLS, backups, and monitoring. The software is the same open-source binary; the difference is who is responsible for keeping it running.

### Kubernetes / Cloud-Native (Future)
For large deployments, the server should be able to run in a Kubernetes-native configuration: stateless app pods backed by managed storage (see [Storage Backends](storage.md)).

## Discovery and Joining

There is no central server directory. Servers are found by:
1. **Invite link** — the server generates a link (`https://myserver.example.com/invite/abc123`). The link encodes the server address and an invite token. The client connects to that address to join.
2. **Direct address entry** — users can enter a server address manually.
3. **(Future) Opt-in listing** — servers can choose to register with a public directory service. This is purely opt-in.

Invite links can be:
- Single-use or multi-use
- Time-limited or permanent
- Channel-specific (join and land in a specific channel)
- Role-granting (automatically assign a role on join)

## Membership Models

Server operators configure membership policy:

| Mode | Description |
|---|---|
| `open` | Anyone with the server address can join |
| `invite_only` | Joining requires a valid invite link |
| `allowlist` | Only specific public keys can join (admin must pre-approve) |
| `closed` | No new members (useful for archiving) |

## Roles and Permissions

A role-based permission system similar to Discord's:

- Roles are ordered (higher roles override lower roles)
- Each role has a set of boolean permission flags (send messages, manage channels, kick members, etc.)
- Permissions can be overridden at the channel level
- One built-in `@everyone` role (applied to all members) and one `@admin` role (for server owners/admins)

The full permission flag set is defined in the server schema. This is a significant design surface — it should be iterated on before implementation.

## Server Configuration

Server operators have a configuration file (TOML) and/or an admin UI. Key configuration areas:

### Identity & Access
- `membership_mode` — open | invite_only | allowlist | closed
- `require_invite_for_dm` — whether DMs require being server members
- Default role assigned to new members

### Feature Flags
Features can be enabled or disabled per server. This lets operators:
- Run a text-only server with no media upload support
- Disable voice/video to reduce infrastructure requirements
- Run a read-only announcement server

| Feature | Default | Notes |
|---|---|---|
| `text_channels` | enabled | Cannot be disabled (core feature) |
| `voice_channels` | enabled | Requires SFU component |
| `video` | enabled | Requires voice channels |
| `screen_share` | enabled | Requires voice channels |
| `file_uploads` | enabled | Configurable size limits |
| `emoji_reactions` | enabled | |
| `message_threads` | enabled | |
| `message_search` | enabled | May be expensive for large servers |

### Content Policy
- `max_message_length` — character limit
- `max_file_size` — per-upload size limit
- `allowed_file_types` — allowlist of MIME types (empty = all allowed)
- `retention_policy` — how long messages and media are retained (see [Storage](storage.md))
- `require_content_scan` — (future) integration with content moderation service

### Moderation
- Configurable list of banned public keys
- Timeout / slow mode per channel
- Audit log for admin actions
- Webhook support for logging mod actions to external services (future)

## Moderation

Server admins and users with appropriate roles can:
- Kick or ban a user (ban = block their public key from joining)
- Delete messages
- Set channel slow mode
- Pin messages
- Manage roles

Since users are identified by public keys, bans are per-key. An evaded ban (user generates a new key) is visible as a new account with no history. The server can choose to require invite-only or allowlist mode to prevent ban evasion.

**Open question:** Should servers be able to share ban lists with each other (an opt-in "trust network" for moderation)? This has benefits for spam/abuse control but raises privacy concerns.

## Voice and Video Channels

Voice channels operate differently from text channels:
- Users "join" a voice channel (establish a WebRTC connection to the server SFU)
- All connected users in the channel receive each other's media streams
- Users can mute/deafen locally
- Camera and screen share are optional streams alongside audio

Server configuration controls:
- Maximum participants per voice channel
- Whether video is permitted
- Whether screen share is permitted
- Noise suppression / echo cancellation (handled client-side but server may declare requirements)
- Recording (off by default; if enabled, all participants are notified)

## Admin Operations

The server exposes an admin API (accessible only to admin-role users):
- Manage channels and categories
- Manage roles and permissions
- View and manage membership
- View audit log
- Configure server settings
- Generate invite links
- Backup and restore (export/import server data)

The admin API should be accessible via both the client UI and a CLI tool, so operators can script server management without a GUI.

## Server-to-Server (Federation) — Future

The initial release treats each server as an island. Future federation would allow:
- Users from server A to join server B without creating a new identity
- Cross-server DMs between users on different servers
- Shared channels between federated servers

The identity model (public keys) is designed to support this without changes. The protocol layer for federation will be a separate design document.
