# Feature: Client Shell

## Overview
The client shell is the foundational React application layout inside the Tauri window. It provides the main UI structure -- server sidebar, channel list, message view, and message input -- along with Zustand state management, server connection handling, authentication flow, and the ability to send and receive messages in real-time. This is the minimum viable client that ties together the server-side features (auth, gateway, channels, messages) into a usable application.

## Background
The architecture doc (`docs/design/architecture.md`) defines the client as a Tauri v2 app with a Rust core handling key management/signing and a React frontend handling UI, WebSocket management, and state. State management uses Zustand with one store per connected server and a top-level app store. The identity doc (`docs/design/identity.md`) specifies that the private key never leaves the Tauri core -- signing is performed via IPC. This feature depends on identity (feature 4) for key management IPC, auth (feature 5) for the challenge-response flow, the gateway (feature 6) for WebSocket connection, channels (feature 7) for the channel list, and messages (feature 8) for sending/receiving messages.

## Requirements
- [ ] The app displays a server sidebar on the left showing the connected server.
- [ ] Clicking a server shows its channel list in a second sidebar panel.
- [ ] Clicking a channel shows the message view for that channel.
- [ ] The message view displays message history with infinite scroll (loads older messages on scroll-up).
- [ ] A message input bar at the bottom allows typing and sending messages.
- [ ] On launch, the app prompts the user to enter a server address (if no server is configured).
- [ ] The app authenticates with the server using the challenge-response flow via Tauri IPC for signing.
- [ ] After authentication, the app opens a WebSocket connection to the gateway.
- [ ] The app subscribes to the currently viewed channel and displays incoming messages in real-time.
- [ ] The app auto-scrolls to new messages when the user is at the bottom of the message view.
- [ ] The app shows a connection status indicator (connected, connecting, disconnected).
- [ ] Channel and message state is managed via Zustand store (`serverStore`).

## Design

### API / Interface Changes

**Tauri IPC commands (Rust core -> React):**

These commands are invoked from the React app via `@tauri-apps/api/core`:

| Command | Input | Output | Purpose |
|---------|-------|--------|---------|
| `get_public_key` | -- | `{ pubkey: string }` | Get the user's public key (base58-encoded) |
| `sign_challenge` | `{ challenge: string }` | `{ signature: string }` | Sign a nonce with the private key |

**No new REST or gateway endpoints.** The client consumes endpoints defined in features 5-8.

### Data Model Changes

No server-side changes. Client-side Zustand stores:

**AppStore (top-level):**
```typescript
interface AppStore {
  currentServerId: string | null;
  servers: Record<string, ServerConnection>;
  theme: ThemeName;
  addServer(address: string): void;
  setCurrentServer(id: string): void;
}
```

**ServerStore (one per connected server):**
```typescript
interface ServerStore {
  serverId: string;
  address: string;
  status: "connecting" | "connected" | "disconnected";
  sessionToken: string | null;
  channels: Channel[];
  categories: Category[];
  currentChannelId: string | null;
  messages: Record<string, Message[]>; // channel_id -> messages
  hasMore: Record<string, boolean>;    // channel_id -> has more history
  setCurrentChannel(id: string): void;
  sendMessage(content: string): Promise<void>;
  loadMoreMessages(channelId: string): Promise<void>;
}
```

### Component Changes

**New files (client React app, `client/src/`):**

Layout components:
- `client/src/components/layout/AppShell.tsx` -- Top-level layout: server sidebar + content area.
- `client/src/components/layout/ServerSidebar.tsx` -- Vertical icon bar of connected servers.
- `client/src/components/layout/ChannelSidebar.tsx` -- Channel list grouped by category.
- `client/src/components/layout/MessageView.tsx` -- Scrollable message history with infinite scroll.
- `client/src/components/layout/MessageInput.tsx` -- Text input with send button (Enter to send).

Message components:
- `client/src/components/messages/MessageItem.tsx` -- Single message display (author, content, timestamp).
- `client/src/components/messages/MessageList.tsx` -- List of MessageItem with scroll handling.

Connection components:
- `client/src/components/connection/ServerConnect.tsx` -- Form to enter server address and connect.
- `client/src/components/connection/StatusIndicator.tsx` -- Connection status badge.

State management:
- `client/src/stores/appStore.ts` -- AppStore (Zustand).
- `client/src/stores/serverStore.ts` -- ServerStore factory (Zustand).

Services:
- `client/src/services/auth.ts` -- Authentication flow: call challenge endpoint, sign via IPC, call verify endpoint.
- `client/src/services/gateway.ts` -- WebSocket connection management: connect, reconnect, message dispatch to store.
- `client/src/services/api.ts` -- REST API client (fetch wrapper with auth header).

**New files (Tauri core, `client/src-tauri/src/`):**

- `client/src-tauri/src/commands/mod.rs` -- IPC command module root.
- `client/src-tauri/src/commands/identity.rs` -- `get_public_key` and `sign_challenge` Tauri commands.

**Modified files:**

- `client/src/App.tsx` -- Render AppShell or ServerConnect based on connection state.
- `client/src-tauri/src/main.rs` (or `lib.rs`) -- Register IPC commands.

## Task List

### Phase A: Tauri IPC
- [ ] `get_public_key` and `sign` Tauri commands implemented in identity feature (feature 4).

### Phase B: Services layer
- [ ] `client/src/services/api.ts` — REST API client with auth header injection.
- [ ] `client/src/api/auth.ts` — challenge-response auth flow (implemented in auth feature, feature 5).
- [ ] `client/src/services/gateway.ts` — WebSocket manager: connect, auto-reconnect, heartbeat, event dispatch.

### Phase C: State management
- [ ] `client/src/stores/serverStore.ts` — Zustand store with connect/disconnect/selectChannel/sendMessage/loadMoreMessages.
- [ ] Gateway event handlers wired in store: MESSAGE_CREATE/UPDATE/DELETE, CHANNEL_CREATE/UPDATE/DELETE, CATEGORY_*.

### Phase D: Layout components
- [ ] `AppShell.tsx` — three-column flexbox layout.
- [ ] `ServerSidebar.tsx` — server icon/initial.
- [ ] `ChannelSidebar.tsx` — categories + channels, active highlight, click to switch, status indicator.
- [ ] `MessageView.tsx` — channel header + MessageList + MessageInput.
- [ ] `MessageList.tsx` — scroll-to-bottom on new messages, scroll-up to load more.
- [ ] `MessageItem.tsx` — message display with author, content, edited indicator, tombstone for deleted.
- [ ] `MessageInput.tsx` — Enter to send, Shift+Enter for newline, disabled when not connected.
- [ ] `StatusIndicator.tsx` — color-coded connection status badge.

### Phase E: Connection flow
- [ ] `ServerConnect.tsx` — form with server URL input, error display, connect button.
- [ ] `App.tsx` updated: Setup → ServerConnect → AppShell based on state.

## Test List
- [ ] Unit test: `ServerStore` -- connect/disconnect state transitions. (Deferred — Zustand mocking in vitest requires more setup.)
- [ ] Unit test: `MessageInput` -- Enter key triggers send, Shift+Enter inserts newline, disabled state.
- [ ] Unit test: `MessageItem` -- renders content, author, edited indicator, deleted tombstone.
- [ ] Unit test: Gateway service -- mock WebSocket. (Deferred — jsdom WebSocket mock setup needed.)
- [ ] Integration test: Full flow. (Manual only — requires running server.)
- [ ] Manual: Launch the Tauri app, enter a server address, see channels load, click a channel, send a message, verify it appears immediately.
- [ ] Manual: Open two client instances, send a message from one, verify it appears in the other in real-time.
- [ ] Manual: Scroll up in message history, verify older messages load.

## Open Questions
- Should the client persist the server list and last-viewed channel across app restarts? Tauri provides local storage APIs. This is likely needed for usable UX but could be deferred to a follow-up.
- How should the client handle token expiration mid-session? The gateway will disconnect; the client should detect this and re-authenticate transparently.
- Should message sending be optimistic (show immediately, confirm later) or wait for the server echo via gateway? Optimistic is better UX but requires handling the case where the server rejects the message.
