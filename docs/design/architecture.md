# Architecture

## System Overview

decentcom has three primary components:

```
┌─────────────────────────────────────────────────────────────┐
│  Client (Tauri + React)                                     │
│  ┌─────────────┐  ┌───────────────────────────────────────┐ │
│  │  Tauri host │  │  React app                            │ │
│  │  (Rust)     │  │  - UI / state                         │ │
│  │  - key mgmt │  │  - WS connection                      │ │
│  │  - signing  │  │  - WebRTC (voice/video)               │ │
│  └─────────────┘  └───────────────────────────────────────┘ │
└──────────────────────────┬──────────────────────────────────┘
                           │ HTTPS + WSS
                           ▼
┌─────────────────────────────────────────────────────────────┐
│  Server (Rust / axum)                                       │
│  ┌───────────┐  ┌──────────────┐  ┌──────────────────────┐ │
│  │  REST API │  │  WS Gateway  │  │  Media / WebRTC SFU  │ │
│  │           │  │  (presence,  │  │  (voice, video,      │ │
│  │           │  │   messages)  │  │   screen share)      │ │
│  └───────────┘  └──────────────┘  └──────────────────────┘ │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │  Storage layer (pluggable backend)                      │ │
│  └─────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

The client connects to exactly one server at a time (per community/server in the Discord sense). A user may be a member of multiple servers; the client maintains a separate WebSocket connection to each one that the user has open.

## Server Components

### REST API
Used for operations that are request/response in nature:
- Authentication (key challenge / response handshake)
- Fetching channel history
- Creating/editing/deleting messages, channels, roles
- Server settings and admin operations
- Invite management

### WebSocket Gateway
A persistent connection per authenticated client session. Used for:
- Real-time message delivery
- Presence (online/offline/away/busy)
- Typing indicators
- Live membership changes (role updates, joins, leaves)

The gateway is event-driven. The server pushes events; the client subscribes at connect time and can subscribe/unsubscribe to specific channels.

### Media / SFU (Selective Forwarding Unit)
Voice and video channels use WebRTC. The server acts as an SFU — it receives media from each participant and forwards it selectively to others, rather than requiring participants to send full mesh streams to each other. This is how Discord and most modern voice platforms work.

The SFU is an optional component. Servers that do not enable voice/video can omit it.

**Open question:** Build our own SFU in Rust, or integrate an existing one (e.g. mediasoup via a sidecar, or livekit as an optional dependency)?

## Client Architecture

The client is a Tauri v2 application. Tauri provides:
- A native OS window wrapping a WebView
- A Rust backend process (the "core") that handles privileged operations
- IPC between the React frontend and the Rust core

### Responsibilities split between Tauri core and React

| Tauri Core (Rust) | React App (TypeScript) |
|---|---|
| Key storage (OS keychain or encrypted file) | All UI rendering |
| Message signing | WebSocket management |
| File system access | State management |
| OS notifications | WebRTC (via browser APIs) |
| Auto-update | Theme / preferences |

The private key **never leaves the Tauri core process.** Signing operations are performed in Rust; the React app sends data to be signed and receives a signature — it never touches the raw private key.

### State Management

Each server connection is maintained as an independent state slice. Recommended approach: Zustand with one store per connected server, a top-level store for app-wide state (current server, theme, user preferences).

## Communication Protocol

### Authentication Flow

```
Client                                    Server
  |                                          |
  |-- POST /auth/challenge { pubkey } -----> |
  |                                          | generate nonce, store temporarily
  |<-- { challenge: nonce } --------------- |
  |                                          |
  | (Tauri core signs nonce with privkey)    |
  |                                          |
  |-- POST /auth/verify { pubkey, sig } ---> |
  |                                          | verify sig against pubkey
  |<-- { session_token, ... } ------------- |
  |                                          |
  |-- WS /gateway [Authorization: token] -> |
  |<======== persistent event stream ======>|
```

Session tokens are short-lived JWTs (or an opaque token stored server-side). The server never stores anything derivable from the private key.

### WebSocket Events

Events follow a typed envelope:

```json
{
  "op": "MESSAGE_CREATE",
  "d": { ... },
  "t": 1712345678
}
```

Operation types are defined in a shared schema (generated types for both Rust and TypeScript).

### API Versioning

The REST API is versioned via URL prefix (`/api/v1/`). The WebSocket protocol version is negotiated at connect time. This allows clients and servers to be independently updated without hard breakage.

## Security Boundaries

- The server trusts signed operations from authenticated public keys.
- The server does **not** trust the client for privilege escalation — role and permission checks happen server-side.
- Media streams pass through the SFU but are not stored unless the server explicitly enables recording (off by default, and a server setting).
- HTTPS/WSS are required in production. The server should reject plain HTTP connections (configurable for local dev).

## Federation (Future)

Federation is not in scope for the initial release but should be designed for. The identity model (public keys, not server-local usernames) is already federation-friendly: a user from server A can prove their identity to server B without server A being involved.

Likely approach: a protocol inspired by ActivityPub or ATProto, but scoped to the decentcom data model. This is a major design effort and will be its own document.

## Open Questions

1. **SFU strategy** — build vs. integrate (mediasoup, livekit)?
2. **Session tokens** — stateless JWT vs. server-stored opaque tokens?
3. **Message format** — Markdown subset? Full CommonMark? Custom?
4. **Push notifications** — For mobile (future), APNs/FCM require a relay. How does this interact with decentralization?
5. **API transport** — REST + WS is straightforward; gRPC or GraphQL subscriptions are alternatives worth evaluating for the WS layer.
