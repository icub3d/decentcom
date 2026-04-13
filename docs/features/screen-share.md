# Feature: Screen Sharing

## Overview
Screen sharing allows a participant in a voice channel to share their entire screen or a specific application window as an additional video stream. Other participants see the shared screen alongside (or instead of) the sharer's camera feed. Screen sharing supports a "presenter mode" where the shared screen is displayed prominently while other participants' video tiles are minimized.

## Background
The server-model doc (`docs/design/server-model.md`) defines `screen_share` as a feature flag (enabled by default, requires `voice_channels` to be enabled). Screen sharing is architecturally identical to video from the SFU's perspective — it is an additional video track on the same WebRTC peer connection. The key differences are on the client side: the source is `getDisplayMedia()` instead of `getUserMedia()`, and the UI presents the shared screen differently from camera feeds.

Depends on: `voice` (feature #22), `video` (feature #23), all Phase 1 and Phase 2 features.

## Requirements
- [ ] Users in a voice channel can share their screen or a specific window
- [ ] Users can stop screen sharing at any time
- [ ] Only one user per voice channel can screen share at a time (simplifies layout and bandwidth)
- [ ] All participants see the shared screen as a distinct stream (not a camera feed)
- [ ] Presenter mode: shared screen is displayed large, participant video tiles are shown small alongside
- [ ] Screen sharing respects the `screen_share` feature flag
- [ ] The participant list indicates who is sharing their screen
- [ ] Screen share uses a high-resolution, low-framerate encoding profile optimized for content (text, slides) rather than motion video

## Design

### API / Interface Changes

**REST endpoints (additions to voice):**

| Method | Path | Description |
|---|---|---|
| POST | `/api/v1/voice/{channel_id}/screen-share/start` | Request to start screen sharing; returns error if another user is already sharing |
| POST | `/api/v1/voice/{channel_id}/screen-share/stop` | Stop screen sharing |

**WebSocket events (additions):**

| Event | Direction | Description |
|---|---|---|
| `VOICE_STATE_UPDATE` | server -> client | Extended to include `screen_sharing` field |
| `SCREEN_SHARE_STARTED` | server -> client | A participant started screen sharing (includes track info) |
| `SCREEN_SHARE_STOPPED` | server -> client | A participant stopped screen sharing |

**Tauri IPC commands (additions):**

| Command | Description |
|---|---|
| `voice_start_screen_share` | Triggers `getDisplayMedia()` in the WebView and adds the screen track to the peer connection |
| `voice_stop_screen_share` | Removes the screen share track and notifies the server |

### Data Model Changes

**Extended `voice_states` (in-memory):**

| Column | Type | Description |
|---|---|---|
| `screen_sharing` | BOOLEAN | Whether the user is currently sharing their screen |

**Extended per-channel ephemeral state:**

| Field | Type | Description |
|---|---|---|
| `active_screen_share_user_id` | INTEGER or NULL | The user currently sharing their screen in this channel |

No persistent schema changes.

### Component Changes

**Server (`server/`):**

- `server/src/sfu/mod.rs` — track which user (if any) is screen sharing per room; enforce one-at-a-time limit
- `server/src/sfu/session.rs` — handle renegotiation for screen share track (distinct from camera track via track labels or SDP metadata)
- `server/src/sfu/router.rs` — forward screen share track with content-optimized settings (higher resolution, lower framerate)
- `server/src/routes/voice.rs` — add `screen-share/start` and `screen-share/stop` endpoints
- `server/src/gateway/events.rs` — add `SCREEN_SHARE_STARTED` and `SCREEN_SHARE_STOPPED` events
- `server/src/config.rs` — add `screen_share` feature flag check

**Client (`client/`):**

- `client/src/hooks/useVoice.ts` — extend with `startScreenShare()` and `stopScreenShare()` methods; use `getDisplayMedia()` to capture screen
- `client/src/components/VoiceControls.tsx` — add screen share toggle button (disabled when another user is sharing)
- `client/src/components/ScreenShareView.tsx` — new component: large view of the shared screen with controls overlay
- `client/src/components/VideoGrid.tsx` — modify to support presenter mode layout: screen share large + participant tiles small on the side
- `client/src/stores/voiceStore.ts` — extend to track screen share state, active screen sharer, and screen share MediaStream

## Task List

### Phase A: Server Screen Share Support
- [ ] Add `screen_sharing` field to in-memory voice state (`VoiceParticipant.screen_sharing`)
- [ ] Add per-channel `active_screen_share` tracking in `VoiceStateMap` (`screen_shares` map)
- [ ] Implement one-at-a-time enforcement: reject screen share start if another user is already sharing (409 Conflict)
- [ ] Add `screen-share/start` and `screen-share/stop` REST endpoints (`POST /api/v1/voice/{channel_id}/screen-share/{start,stop}`)
- [ ] Add `screen_share` feature flag check
- [ ] Handle screen share track in `server/src/sfu/session.rs` *(deferred — SFU unresolved)*
- [ ] Configure content-optimized forwarding in `server/src/sfu/router.rs` *(deferred)*
- [ ] Add `SCREEN_SHARE_STARTED` and `SCREEN_SHARE_STOPPED` gateway events
- [ ] Clean up screen share state when the sharer disconnects or leaves the voice channel (gateway disconnect handler)

### Phase B: Client Screen Share UI
- [ ] Extend `client/src/hooks/useVoice.ts` with `startScreenShare()` using `navigator.mediaDevices.getDisplayMedia()`
- [ ] Add screen share track to the peer connection and trigger renegotiation
- [ ] Implement `stopScreenShare()` — remove track, notify server
- [ ] Handle the browser's built-in "stop sharing" callback (fires when user clicks the browser's stop-sharing button)
- [ ] Add screen share button to `client/src/components/VoiceControls.tsx` with disabled state when someone else is sharing
- [ ] Create `client/src/components/ScreenShareView.tsx` — renders the screen share stream in a large container
- [ ] Modify `client/src/components/VideoGrid.tsx` to switch to presenter mode layout when a screen share is active
- [ ] Extend `client/src/stores/voiceStore.ts` with screen share state tracking
- [ ] Wire up `SCREEN_SHARE_STARTED`/`SCREEN_SHARE_STOPPED` events to update the UI

## Test List
- [ ] Unit test: starting screen share when another user is already sharing returns an error
- [ ] Unit test: screen share is rejected when `screen_share` feature flag is disabled
- [ ] Unit test: screen share state is included in voice state updates
- [ ] Unit test: disconnecting the sharer clears the active screen share for the room
- [ ] Integration test: client starts screen share, server forwards the track to a second client
- [ ] Integration test: client stops screen share, other clients receive SCREEN_SHARE_STOPPED
- [ ] Integration test: sharer disconnects unexpectedly, remaining clients are notified that screen share ended
- [ ] Manual test: share a screen in a voice channel with two clients — verify the screen is visible to the other participant
- [ ] Manual test: verify presenter mode layout shows screen share large and participant videos small
- [ ] Manual test: stop screen sharing and verify the layout reverts to the normal video grid
- [ ] Manual test: use the browser's "stop sharing" button and verify the server and other clients are notified

## Open Questions
- **Multiple screen shares:** The current design limits to one screen share per channel. Should this be configurable (e.g., allow 2 simultaneous shares for collaborative work)?
- **Audio sharing:** `getDisplayMedia()` can capture system audio on some platforms. Should we support sharing system audio alongside the screen? This is useful for sharing videos or presentations with sound.
- **Resolution and framerate:** What default constraints for `getDisplayMedia()`? Suggested: 1080p at 5-15fps for content, with the option to switch to higher framerate for video playback.
- **Tauri WebView limitations:** Does the Tauri v2 WebView support `getDisplayMedia()`? Some WebView implementations have restrictions on screen capture APIs. This needs verification early.
