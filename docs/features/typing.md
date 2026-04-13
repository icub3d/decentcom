# Feature: Typing Indicators

## Overview
Show real-time typing indicators when other users are composing messages in a channel or DM. The client broadcasts typing events through the WebSocket gateway, and other clients in the same channel display a "User is typing..." indicator with appropriate debounce and timeout logic to avoid flickering.

## Background
The architecture doc (`docs/design/architecture.md`) lists typing indicators as a core WebSocket gateway responsibility. The storage doc (`docs/design/storage.md`) explicitly notes that presence state (including typing) is entirely ephemeral and maintained in memory only — it is never persisted. Typing state is cleared on server restart and clients re-establish on reconnect.

## Requirements
- [ ] When a user starts typing in a channel, other users in that channel see a typing indicator
- [ ] Typing indicators show which user(s) are currently typing
- [ ] The client sends typing events with debounce: no more than one event per 5 seconds while the user is actively typing
- [ ] Typing indicators timeout after 8 seconds of no new typing event from that user
- [ ] Typing indicators disappear immediately when the user sends a message
- [ ] Typing state is ephemeral (in-memory only, not persisted)
- [ ] Typing events respect channel visibility — only users with `VIEW_CHANNEL` permission receive them
- [ ] Typing events work in both text channels and DM channels
- [ ] The server does not broadcast typing events back to the sender

## Design

### API / Interface Changes

**New gateway client-to-server event:**

```json
{
  "op": "TYPING_START",
  "d": {
    "channel_id": "uuid"
  }
}
```

Sent by the client when the user begins typing. The client is responsible for debouncing (send at most once per 5 seconds while the user continues typing).

**New gateway server-to-client event:**

```json
{
  "op": "TYPING_START",
  "d": {
    "channel_id": "uuid",
    "user_id": "uuid",
    "display_name": "Alice",
    "timestamp": 1712345678
  },
  "t": 1712345678
}
```

Broadcast to all other users subscribed to the channel.

No REST endpoint is needed — typing is purely a real-time ephemeral event.

### Data Model Changes

No database changes. Typing state is held in-memory on the server in a structure like:

```rust
// In the gateway session manager
struct TypingState {
    /// channel_id -> set of (user_id, last_typing_at)
    channels: HashMap<Uuid, HashMap<Uuid, Instant>>,
}
```

A background task periodically prunes entries older than the timeout threshold (8 seconds).

### Component Changes

**Server (`server/`):**
- `server/src/gateway/events.rs` — Add `TYPING_START` to the event type enum
- `server/src/gateway/handler.rs` — Handle incoming `TYPING_START` events: validate channel subscription, check `VIEW_CHANNEL` permission, update in-memory typing state, broadcast to other subscribers
- `server/src/gateway/typing.rs` — New module: `TypingState` struct, insertion, pruning logic, background cleanup task

**Client (`client/`):**
- `client/src/hooks/useTyping.ts` — New hook: manages sending typing events with 5-second debounce, resets on message send
- `client/src/components/TypingIndicator.tsx` — New component: displays "X is typing...", "X and Y are typing...", or "Several people are typing..." with animated dots
- `client/src/components/MessageInput.tsx` — Integrate `useTyping` hook to fire typing events on keystrokes
- `client/src/stores/channelStore.ts` — Add typing users state per channel, update on `TYPING_START` events, clear on timeout or message receipt
- `client/src/gateway/handlers.ts` — Add handler for incoming `TYPING_START` events

## Task List

### Server
- [ ] `TYPING_START` Op added to gateway event enum in `shared/src/lib.rs`
- [ ] Created `server/src/gateway/typing.rs` with `TypingState` struct and methods: `set_typing`, `get_typing`, `prune_expired`
- [ ] Background tokio task in `server/src/main.rs` calls `prune_expired()` every 2 seconds
- [ ] Inbound `TYPING_START` handled in `server/src/gateway/handler.rs`: updates `TypingState`, broadcasts via `broadcast_to_channel_except` (excludes sender)

### Client
- [ ] Add typing users state to the channel store in `client/src/stores/channelStore.ts`
- [ ] Add `TYPING_START` handler in `client/src/gateway/handlers.ts` that updates the channel store and sets an 8-second expiry timer per user
- [ ] Clear a user's typing state when a `MESSAGE_CREATE` event arrives from that user in that channel
- [ ] Create `useTyping` hook in `client/src/hooks/useTyping.ts` with 5-second debounce: sends `TYPING_START` on first keystroke, suppresses repeats for 5 seconds, stops on message send
- [ ] Create `TypingIndicator.tsx` component with animated dots and user name display
- [ ] Integrate `useTyping` into `MessageInput.tsx` (fire on input change)
- [ ] Render `TypingIndicator` below the message input area in the channel view

## Test List
- [ ] Unit test: `TypingState::set_typing` adds user, `prune_expired` removes stale entries (typing.rs unit tests)
- [ ] Integration test: client A types in a channel, client B receives `TYPING_START` event
- [ ] Integration test: typing event is not sent back to the sender
- [ ] Integration test: typing state expires after 8 seconds of no new event
- [ ] Manual: type in a channel, verify other connected clients show typing indicator
- [ ] Manual: send the message, verify typing indicator disappears immediately for other clients
- [ ] Manual: verify no flickering when typing continuously (debounce working)

## Open Questions
- Should typing indicators be shown for bot/automated users in the future, or suppressed?
- Should the typing indicator display a maximum number of names (e.g., show 3 names then "and N others are typing...")?
- Should typing events be rate-limited server-side as an additional guard beyond the client-side debounce?
