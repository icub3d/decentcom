# Feature: Video Chat

## Overview
Video chat adds camera streams to voice channels, allowing participants to see each other alongside hearing audio. Video is an optional layer on top of voice — joining a voice channel does not require enabling video, and video can be toggled on/off at any time. The server SFU forwards video tracks the same way it forwards audio tracks, selectively routing each participant's camera stream to all others.

## Background
The architecture doc (`docs/design/architecture.md`) lists video as part of the Media / SFU component. The server-model doc (`docs/design/server-model.md`) defines `video` as a feature flag (enabled by default, requires `voice_channels` to also be enabled). Video is an additional media track on the same WebRTC peer connection established for voice — it does not require a separate signaling flow.

Depends on: `voice` (feature #22), all Phase 1 and Phase 2 features.

## Requirements
- [ ] Users in a voice channel can enable their camera to send a video stream
- [ ] Users can disable their camera at any time without leaving the voice channel
- [ ] All participants in the voice channel see a grid of active video streams
- [ ] The participant list indicates who has video enabled
- [ ] Video respects the `video` feature flag — disabled servers do not allow video tracks
- [ ] Video tracks use VP8 or VP9 codec with configurable quality constraints
- [ ] Simulcast: clients send multiple quality layers so the SFU can forward appropriate quality based on receiver bandwidth

## Design

### API / Interface Changes

**REST endpoints (additions to voice):**

| Method | Path | Description |
|---|---|---|
| PATCH | `/api/v1/voice/{channel_id}/state` | Extended to include `video_enabled` field |

**WebSocket events (additions):**

| Event | Direction | Description |
|---|---|---|
| `VOICE_STATE_UPDATE` | server -> client | Extended to include `video_enabled` field |
| `VOICE_TRACK_ADDED` | server -> client | A new video track is available from a participant |
| `VOICE_TRACK_REMOVED` | server -> client | A participant's video track was removed |

**Tauri IPC commands (additions):**

| Command | Description |
|---|---|
| `voice_toggle_video` | Toggles camera capture; adds/removes video track on the peer connection |

### Data Model Changes

**Extended `voice_states` (in-memory):**

| Column | Type | Description |
|---|---|---|
| `video_enabled` | BOOLEAN | Whether the user's camera is active |

No persistent schema changes. Video state is entirely ephemeral.

### Component Changes

**Server (`server/`):**

- `server/src/sfu/session.rs` — handle renegotiation when a video track is added/removed to an existing peer connection
- `server/src/sfu/router.rs` — forward video tracks alongside audio; implement simulcast layer selection based on receiver count or available bandwidth
- `server/src/routes/voice.rs` — extend state endpoint to accept `video_enabled`
- `server/src/gateway/events.rs` — add `VOICE_TRACK_ADDED` and `VOICE_TRACK_REMOVED` events
- `server/src/config.rs` — add `video` feature flag check (must also have `voice_channels` enabled)

**Client (`client/`):**

- `client/src/hooks/useVoice.ts` — extend to manage video track: getUserMedia with video constraints, add/remove track on peer connection, handle renegotiation
- `client/src/components/VoiceControls.tsx` — add camera toggle button
- `client/src/components/VideoGrid.tsx` — new component: grid layout of participant video streams, adapts layout based on participant count (1=full, 2=split, 3-4=2x2, 5+=scrollable grid)
- `client/src/components/VideoTile.tsx` — new component: single participant's video with name overlay and mute indicator
- `client/src/stores/voiceStore.ts` — extend to track video-enabled state per participant and video MediaStream references

## Task List

### Phase A: Server Video Track Handling
- [ ] Extend voice state to track `video_enabled` per participant (`VoiceParticipant.video_enabled`)
- [ ] Add `video` feature flag check — reject video track addition if `video` is disabled on the server
- [ ] Implement SDP renegotiation in `server/src/sfu/session.rs` *(deferred — SFU unresolved)*
- [ ] Extend `server/src/sfu/router.rs` to forward video tracks *(deferred)*
- [ ] Add simulcast support in the SFU *(deferred)*
- [ ] Add `VOICE_TRACK_ADDED` and `VOICE_TRACK_REMOVED` gateway events

### Phase B: Client Video Capture and Display
- [ ] Extend `client/src/hooks/useVoice.ts` to request camera access via `getUserMedia({ video: true })` and add the video track to the peer connection
- [ ] Implement SDP renegotiation on the client side when toggling video
- [ ] Add camera toggle button to `client/src/components/VoiceControls.tsx`
- [ ] Create `client/src/components/VideoGrid.tsx` — responsive grid layout for video streams
- [ ] Create `client/src/components/VideoTile.tsx` — individual video element with participant name overlay
- [ ] Extend `client/src/stores/voiceStore.ts` to track video-enabled state and remote video MediaStream objects
- [ ] Wire up `VOICE_TRACK_ADDED`/`VOICE_TRACK_REMOVED` events to attach/detach video streams in the grid

## Test List
- [ ] Unit test: enabling video when the `video` feature flag is disabled returns an error
- [ ] Unit test: video state is correctly tracked in voice state updates
- [ ] Unit test: SFU router forwards video tracks to all participants except the sender
- [ ] Unit test: simulcast layer selection picks lower quality when forwarding to many receivers
- [ ] Integration test: client adds a video track, server renegotiates and forwards to a second client
- [ ] Integration test: client removes a video track, other clients receive VOICE_TRACK_REMOVED
- [ ] Integration test: video state update is broadcast to all participants
- [ ] Manual test: enable camera in a voice channel with two clients — verify video appears on both sides
- [ ] Manual test: toggle camera off — verify video disappears for other participants but audio continues
- [ ] Manual test: verify VideoGrid layout adapts correctly for 1, 2, 4, and 6 participants

## Open Questions
- **Simulcast layers:** How many layers (e.g., 3 at 720p/360p/180p)? Should the SFU dynamically switch layers based on receiver bandwidth, or should the client request a specific quality?
- **Bandwidth limits:** Should the server enforce per-participant or per-channel bandwidth caps for video? This affects server resource usage significantly.
- **Camera selection:** Should the client support selecting a specific camera device when multiple are available? This is a UX question — likely yes, but the UI for device selection needs design.
- **Video codec:** VP8 is universally supported; VP9 offers better compression. Should we prefer VP9 and fall back to VP8, or let the SDP negotiation decide?
