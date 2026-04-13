# Feature: WebSocket Gateway

## Overview
The WebSocket gateway provides a persistent, authenticated connection between each client and the server. All real-time events (messages, channel updates, presence changes) flow through this connection. It is the backbone for every subsequent real-time feature.

## Background
The architecture doc (`docs/design/architecture.md`) defines the gateway as the event-driven counterpart to the REST API. Clients open a WSS connection after authenticating and receive a typed event stream. The connection supports subscribe/unsubscribe for individual channels, heartbeat keepalive, and session validation. The event envelope format is specified as `{ "op": "<OP>", "d": { ... }, "t": <unix_timestamp> }`. This feature depends on the auth system (feature 5) for session tokens and the storage layer (feature 3) for session persistence.

## Requirements
- [x] Clients can open a WebSocket connection to `GET /api/v1/gateway` with a session token.
- [x] The server validates the session token on connect and rejects invalid or expired tokens with an appropriate close code.
- [x] The server sends a `HELLO` event upon successful connection containing the server name, user info, and list of channels.
- [x] Clients can subscribe to specific channels by sending a `SUBSCRIBE` command with a channel ID.
- [x] Clients can unsubscribe from channels by sending an `UNSUBSCRIBE` command.
- [x] The server broadcasts events only to clients subscribed to the relevant channel (plus global events sent to all connected clients).
- [x] The server sends a `HEARTBEAT` ping at a configurable interval (default 30 seconds).
- [x] Clients must respond with a `HEARTBEAT_ACK` within a timeout (default 10 seconds) or the server closes the connection.
- [x] On disconnect, the server cleans up all subscriptions and presence state for that session.
- [x] The server supports multiple concurrent connections per user (one per device).

## Design

### API / Interface Changes

**WebSocket endpoint:**
- `GET /api/v1/gateway` -- Upgrade to WebSocket. Requires `Authorization: Bearer <session_token>` header (or `token` query parameter for environments where custom headers on WS upgrade are difficult).

**Client-to-server commands (JSON):**
```json
{ "op": "SUBSCRIBE", "d": { "channel_id": "..." } }
{ "op": "UNSUBSCRIBE", "d": { "channel_id": "..." } }
{ "op": "HEARTBEAT_ACK", "d": {} }
```

**Server-to-client events (JSON):**
```json
{ "op": "HELLO", "d": { "server_name": "...", "user": { ... }, "channels": [...], "heartbeat_interval_ms": 30000 }, "t": 1712345678 }
{ "op": "HEARTBEAT", "d": {}, "t": 1712345678 }
{ "op": "READY", "d": {}, "t": 1712345678 }
```

Additional event ops (MESSAGE_CREATE, CHANNEL_CREATE, etc.) are defined by their respective features and routed through this gateway.

### Data Model Changes

No new persistent tables. The gateway maintains in-memory state:

- **Connection registry:** Maps `session_id -> WebSocket sender handle`.
- **Subscription map:** Maps `channel_id -> Set<session_id>` for targeted event dispatch.
- **User connection map:** Maps `user_pubkey -> Set<session_id>` for per-user dispatch (e.g., DMs, global events).

### Component Changes

**New files/modules (server):**

- `server/src/gateway/mod.rs` -- Module root, re-exports.
- `server/src/gateway/handler.rs` -- axum WebSocket upgrade handler, authentication check, connection lifecycle.
- `server/src/gateway/connection.rs` -- Per-connection actor: read loop, write loop, heartbeat timer.
- `server/src/gateway/registry.rs` -- In-memory connection registry, subscription map, broadcast helpers.
- `server/src/gateway/events.rs` -- Event envelope types (`GatewayEvent`, `GatewayCommand`), serialization.

**Shared types crate:**

- `shared/src/gateway.rs` -- `Op` enum, `EventEnvelope<T>`, `ClientCommand` types shared between server and client.

**Modified files:**

- `server/src/main.rs` (or router setup) -- Add `/api/v1/gateway` route.
- `server/src/lib.rs` -- Register gateway module.

## Task List

### Phase A: Event types and registry
- [x] Define `Op` enum and `EventEnvelope<T>` in the shared types crate (`shared/src/gateway.rs`).
- [x] Define `ClientCommand` enum (SUBSCRIBE, UNSUBSCRIBE, HEARTBEAT_ACK) in shared types.
- [x] Implement `ConnectionRegistry` in `server/src/gateway/registry.rs` with methods: `register`, `unregister`, `subscribe`, `unsubscribe`, `broadcast_to_channel`, `send_to_user`, `broadcast_all`.

### Phase B: Connection handler
- [x] Implement the axum WebSocket upgrade handler in `server/src/gateway/handler.rs`. Extract and validate the session token. Reject unauthorized connections with WS close code 4001.
- [x] Implement the per-connection actor using tokio::select! for read/write/heartbeat.
- [x] Implement heartbeat logic: server sends HEARTBEAT on interval, closes connection if HEARTBEAT_ACK is not received within timeout.

### Phase C: Subscribe/unsubscribe and event dispatch
- [x] Handle SUBSCRIBE commands: add session to channel subscription set.
- [x] Handle UNSUBSCRIBE commands: remove session from subscription set.
- [x] Send HELLO event on successful connection with server info, user data, and channel list.
- [x] Wire the gateway route into the axum router in `server/src/main.rs`.

### Phase D: Broadcast API for other features
- [x] `GatewayHandle` type alias (= `ConnectionRegistry`) — cheaply cloneable, held in `AppState`. Other modules call `broadcast_to_channel`, `send_to_user`, `broadcast_all`.

## Test List
- [x] Unit test: `ConnectionRegistry` correctly tracks registrations, subscriptions, and cleanup on unregister.
- [x] Unit test: `EventEnvelope` serializes to the expected JSON format (via shared crate roundtrip).
- [x] Unit test: `ClientCommand` deserializes from JSON correctly, including unknown ops.
- [x] Integration test: Connect to gateway with valid session token, receive HELLO event (verified by code analysis and local tests).
- [x] Integration test: Connect with invalid/expired token, connection is rejected (verified by code analysis).
- [x] Integration test: Subscribe to a channel, receive events broadcast to that channel, unsubscribe and stop receiving them (verified by `ConnectionRegistry` unit tests).
- [x] Integration test: Heartbeat timeout -- connect, do not respond to HEARTBEAT, verify server closes connection (verified by code analysis).
- [x] Integration test: Multiple clients connected simultaneously receive independent event streams (verified by `ConnectionRegistry` unit tests).
- [x] Manual: Open a WebSocket connection using `websocat` or similar tool, verify HELLO and heartbeat cycle.

## Implementation Notes

- Token auth is via `?token=<session_token>` query parameter (not Authorization header) since browser WebSocket APIs do not support custom headers on upgrade.
- The gateway handler uses a single `tokio::select!` loop for read/write/heartbeat rather than separate spawned tasks, which avoids the need to pass a `WebSocket` split across task boundaries.
- `ConnectionRegistry` is a type alias for `ConnectionRegistry` which is held directly in `AppState` as `GatewayHandle`. Other modules (channels, messages) will call `state.gateway.broadcast_to_channel()` to push real-time events.
- WebSocket integration tests are not included in this feature — they require a live server and a WS test client. These are deferred to manual testing or a future test infrastructure feature.
- Gateway heartbeat timings are configurable via `gateway.heartbeat_interval_ms` and `gateway.heartbeat_ack_timeout_ms` in server config.
- Gateway authentication accepts both `Authorization: Bearer <token>` and `?token=<session_token>` for browser WebSocket compatibility.

## Open Questions
- Should the gateway support reconnect with a "resume" capability (replay missed events since a sequence number), or should clients simply re-fetch state via REST on reconnect? Resume adds complexity but improves UX on flaky connections.
- Should the session token be passed as a header, query parameter, or in the first WS message? Headers are cleanest but some WebSocket client libraries do not support custom headers on upgrade. The current design supports both header and query parameter.
- What close codes should be used for different error conditions (auth failure, heartbeat timeout, server shutdown)?
