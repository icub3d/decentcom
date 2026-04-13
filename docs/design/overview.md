# Overview & Vision

## What is decentcom?

decentcom is a self-hostable, real-time communication platform inspired by Discord. It provides organized communities (servers) with text channels, voice/video chat, and direct messaging — but without a centralized operator. Anyone can run a server. Users own their identity across all servers they join.

## Problem Statement

Discord is the dominant platform for community communication, but it has fundamental structural issues:

- **Centralized control.** Discord Inc. can delete servers, ban users, or shut down the service at any time.
- **Credential storage.** Discord stores passwords (hashed, but still a central target).
- **No self-hosting.** There is no way to run your own Discord. Communities are entirely at Discord's discretion.
- **Opaque moderation.** Policy enforcement is inconsistent and non-appealable.

Matrix/Element addresses decentralization but is complex to deploy and has a steep UX curve. Revolt is self-hostable but still uses conventional credential auth and a fairly centralized default experience.

decentcom aims for the middle path: **Discord-level UX, Matrix-level decentralization, minimal operational complexity**.

## Core Principles

### 1. Decentralized by Design
There is no "decentcom cloud." A server is a binary you run. It communicates with clients directly. Servers can optionally federate (future work), but the base model is independent islands — similar to how email servers work, or how IRC networks are independent.

### 2. User-Owned Identity
Users are identified by a public key, not by a username+password on a central server. A user can take their identity to any server without asking permission from decentcom. Servers verify identity via cryptographic challenge-response; they never hold a credential that can be stolen.

### 3. Server Sovereignty
Server operators decide:
- Who can join (open, invite-only, or allowlist)
- What features are enabled (voice, video, screen share)
- Content and moderation policy
- How data is stored
- Whether the server is publicly listed or entirely private

Decentcom provides the tooling; operators make the choices.

### 4. Open Source, Sustainable Business
The server and client software are MIT-licensed. Managed hosting ("decentcom cloud") is the planned revenue model — we compete on operational convenience, not by locking down the software.

## Non-Goals

- **Peer-to-peer messaging without a server.** A server is always involved for persistence and presence. Pure P2P is a possible future extension, not the initial target.
- **Blockchain / token-based identity.** Public key identity does not require a blockchain. We use standard asymmetric cryptography.
- **Drop-in Discord API compatibility.** We are not building an API shim. Discord bots will not work without a port.
- **End-to-end encryption for server channels (initially).** Channel messages are stored on and readable by the server. DMs and private channels can be E2EE in a future iteration.

## Comparison with Alternatives

| | decentcom | Discord | Matrix/Element | Revolt |
|---|---|---|---|---|
| Self-hostable | Yes | No | Yes | Yes |
| No central credential store | Yes | No | Partially | No |
| User-owned identity | Yes | No | Partially (via homeserver) | No |
| Simple to deploy | Goal | N/A | No | Yes |
| Voice / Video | Planned | Yes | Yes (Jitsi) | Limited |
| Open source | MIT | No | Apache 2.0 | AGPL |
| Federation | Planned | No | Yes | No |

## Milestones (High-Level)

1. **Foundation** — server binary, client shell, pubkey auth, text channels
2. **Core UX** — DMs, roles, permissions, server settings, invites
3. **Voice & Video** — WebRTC voice channels, video, screen share
4. **Federation** — cross-server identity and messaging
5. **Managed Hosting** — one-click deploy, billing, support
