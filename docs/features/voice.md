# Feature: Voice Channels

## Overview
Voice channels allow users to join a real-time audio session within a channel using WebRTC. The server acts as a Selective Forwarding Unit (SFU), receiving each participant's audio stream and forwarding it to all other participants. This avoids the O(n^2) bandwidth cost of full-mesh peer-to-peer connections and is the standard architecture used by Discord, Teams, and similar platforms.

## Background
The architecture doc (`docs/design/architecture.md`) defines a Media / WebRTC SFU component as a first-class part of the server. The server-model doc (`docs/design/server-model.md`) specifies that voice channels are a feature-flagged capability (`voice_channels`, enabled by default) with configurable max participants, recording opt-in, and noise suppression requirements. The SFU strategy (build vs. integrate) is listed as an open question in the architecture doc. This feature assumes we integrate an existing SFU library rather than building one from scratch — specifically, using the `webrtc` Rust crate (pure-Rust WebRTC implementation) to build a minimal SFU within the server process.

Depends on: all Phase 1 and Phase 2 features (especially `channels`, `gateway`, `roles`, `client-shell`).

## Requirements
- [ ] Users can create and configure voice channels (distinct channel type from text)
- [ ] Users can join a voice channel and hear audio from all other participants
- [ ] Users can leave a voice channel
- [ ] Users can mute themselves (stop sending audio)
- [ ] Users can deafen themselves (stop receiving audio)
- [ ] A participant list is displayed showing who is in the voice channel and their mute/deafen state
- [ ] The server enforces a configurable max participant limit per voice channel
- [ ] Voice channels respect the `voice_channels` feature flag — disabled servers return an error
- [ ] Voice channel access respects role permissions (connect permission per channel)
- [ ] WebRTC connections use DTLS-SRTP for encrypted media transport

## Design

### API / Interface Changes

**REST endpoints:**

| Method | Path | Description |
|---|---|---|
| POST | `/api/v1/channels` | Create channel with `type: "voice"` and optional `max_participants` |
| POST | `/api/v1/voice/{channel_id}/join` | Request to join a voice channel; returns SFU offer (SDP) and ICE candidates |
| POST | `/api/v1/voice/{channel_id}/leave` | Explicitly leave a voice channel |
| PATCH | `/api/v1/voice/{channel_id}/state` | Update self-state (mute, deafen) |
| GET | `/api/v1/voice/{channel_id}/participants` | List current participants and their states |

**WebSocket events (via gateway):**

| Event | Direction | Description |
|---|---|---|
| `VOICE_STATE_UPDATE` | server -> client | A user joined, left, muted, or deafened |
| `VOICE_OFFER` | server -> client | SDP offer from the SFU for a new participant |
| `VOICE_ANSWER` | client -> server | SDP answer from the client |
| `VOICE_ICE_CANDIDATE` | bidirectional | ICE candidate exchange |

**Tauri IPC commands:**

| Command | Description |
|---|---|
| `voice_join` | Initiates microphone capture and WebRTC connection |
| `voice_leave` | Tears down the WebRTC connection and releases microphone |
| `voice_mute` | Toggles local mute (stops sending audio track) |
| `voice_deafen` | Toggles deafen (stops playback of received audio) |

### Data Model Changes

**New columns on `channels` table:**

| Column | Type | Description |
|---|---|---|
| `channel_type` | TEXT | `"text"` or `"voice"` (default `"text"`) |
| `max_participants` | INTEGER | NULL = unlimited, otherwise max concurrent users |

**New table: `voice_states` (ephemeral, could be in-memory only):**

| Column | Type | Description |
|---|---|---|
| `channel_id` | INTEGER | FK to channels |
| `user_id` | INTEGER | FK to users |
| `session_id` | TEXT | The WebSocket session that owns this voice state |
| `muted` | BOOLEAN | Whether the user is self-muted |
| `deafened` | BOOLEAN | Whether the user is self-deafened |
| `joined_at` | TIMESTAMP | When the user joined the voice channel |

Voice state is ephemeral. On server restart, all voice states are cleared. This table may be kept entirely in memory rather than in SQLite/PostgreSQL.

### Component Changes

**Server (`server/`):**

- `server/src/models/channel.rs` — add `channel_type` and `max_participants` fields
- `server/src/sfu/` — new module for the SFU implementation
  - `server/src/sfu/mod.rs` — SFU manager: tracks active voice sessions, routes media
  - `server/src/sfu/session.rs` — per-participant WebRTC session: SDP negotiation, ICE, media tracks
  - `server/src/sfu/router.rs` — media routing: receives audio from each participant, forwards to others
- `server/src/routes/voice.rs` — REST endpoints for join/leave/state
- `server/src/gateway/events.rs` — add voice event types
- `server/src/gateway/handler.rs` — handle voice signaling events over WebSocket
- `server/src/config.rs` — add `voice_channels` feature flag check

**Client (`client/`):**

- `client/src-tauri/src/commands/voice.rs` — Tauri IPC commands for voice operations
- `client/src/hooks/useVoice.ts` — React hook managing WebRTC peer connection lifecycle
- `client/src/components/VoiceChannel.tsx` — voice channel UI in channel list (shows participant count)
- `client/src/components/VoiceControls.tsx` — mute/deafen/disconnect buttons
- `client/src/components/VoiceParticipants.tsx` — participant list with mute/deafen indicators
- `client/src/stores/voiceStore.ts` — Zustand store for voice connection state

**Dependencies:**

- `webrtc` crate (server-side WebRTC)
- Browser WebRTC APIs via the React app's WebView

## Task List

### Phase A: Server SFU Foundation
- [ ] Add `channel_type` and `max_participants` columns to the channels schema and storage trait
- [ ] Add the `webrtc` crate dependency to the server `Cargo.toml` *(deferred — SFU library choice unresolved)*
- [ ] Create `server/src/sfu/mod.rs` with the SFU manager struct *(deferred — SFU library choice unresolved)*
- [ ] Create `server/src/sfu/session.rs` — WebRTC peer connection per participant *(deferred)*
- [ ] Create `server/src/sfu/router.rs` — media forwarding *(deferred)*
- [ ] Add in-memory voice state tracking (`VoiceStateMap` in `server/src/voice/state.rs`)

### Phase B: Server API & Signaling
- [ ] Create `server/src/voice/handlers.rs` with join, leave, state, and participants endpoints
- [ ] Add voice event types to the gateway event enum (`VOICE_STATE_UPDATE`, `VOICE_OFFER`, `VOICE_ANSWER`, `VOICE_ICE_CANDIDATE`)
- [ ] Implement WebSocket handlers for voice signaling (SDP/ICE forwarded via `forward_sdp` / `forward_ice_candidate`)
- [ ] Add feature flag check — reject voice operations if `voice_channels` is disabled
- [ ] Add permission check — verify user has `READ_MESSAGES` permission for the voice channel
- [ ] Enforce `max_participants` limit on join
- [ ] Clean up voice state when a user's WebSocket disconnects

### Phase C: Client Integration
- [ ] Create `client/src/stores/voiceStore.ts` — track connection state, current channel, mute/deafen
- [ ] Create `client/src/hooks/useVoice.ts` — manage RTCPeerConnection lifecycle, handle SDP offer/answer exchange, ICE candidate exchange via gateway
- [ ] Create Tauri IPC commands in `client/src-tauri/src/commands/voice.rs` (if microphone permissions need native handling)
- [ ] Create `client/src/components/VoiceChannel.tsx` — voice channel entry in channel list showing participant avatars/count
- [ ] Create `client/src/components/VoiceControls.tsx` — persistent bottom bar with mute, deafen, disconnect buttons
- [ ] Create `client/src/components/VoiceParticipants.tsx` — sidebar showing current participants with state indicators
- [ ] Wire up `VOICE_STATE_UPDATE` events to update the participant list in real time

## Test List
- [ ] Unit test: SFU manager creates and destroys rooms correctly *(deferred — SFU not yet implemented)*
- [ ] Unit test: SFU router forwards audio tracks to all participants except the sender *(deferred)*
- [ ] Unit test: joining a full voice channel (at max_participants) returns an error (`join_full_channel_returns_409`)
- [ ] Unit test: voice join is rejected when `voice_channels` feature flag is disabled (`join_disabled_voice_channel_returns_403`)
- [ ] Unit test: voice join is rejected when user tries to join a text channel (`join_text_channel_returns_400`)
- [ ] Integration test: client disconnecting from WebSocket clears their voice state (disconnect cleanup in gateway handler)
- [ ] Integration test: mute/deafen state changes (`update_mute_deafen_state`, `update_voice_state_when_not_joined_returns_404`)
- [ ] Integration test: multiple users join same channel (`multiple_users_join_same_channel`)
- [ ] Integration test: voice channels appear in channel list (`voice_channels_appear_in_channel_list`)
- [ ] Integration test: two clients exchange SDP/ICE and establish media flow *(deferred — SFU not yet implemented)*
- [ ] Manual test: join a voice channel from two Tauri clients and verify audio is transmitted bidirectionally *(deferred)*
- [ ] Manual test: mute self and verify other participants stop hearing audio *(deferred)*
- [ ] Manual test: deafen self and verify local playback stops while others still hear you *(deferred)*
- [ ] Manual test: disconnect and verify participant list updates for remaining users *(deferred)*

## Open Questions
- **SFU library choice:** The `webrtc` crate provides a pure-Rust WebRTC stack. Alternatively, we could run mediasoup or livekit as a sidecar process. The in-process approach is simpler to deploy but may be less mature. Decision needed before implementation begins.
- **TURN relay:** Participants behind symmetric NATs may need a TURN server to relay media. Should decentcom bundle a TURN server, or require operators to configure one externally (e.g. coturn)?
- **Opus codec configuration:** What bitrate and frame size defaults? Should the server mandate codec parameters, or let clients negotiate?
- **Voice activity detection:** Should the server or client handle VAD for showing "who is speaking" indicators?
