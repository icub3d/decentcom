# Feature: User Profiles

## Overview
User profiles allow members to set a per-server display name and avatar. Avatars are uploaded and stored using the server's media storage layer. Other members can view profiles in a profile popover or detail view. Profiles are per-server, meaning a user can have different display names and avatars on different servers.

## Background
The identity design doc (`docs/design/identity.md`) specifies that `display_name` and `avatar_hash` are per-server metadata stored in the user record. The storage design doc (`docs/design/storage.md`) describes content-addressable media storage where files are keyed by hash and deduplicated. This feature connects those two pieces: uploading an avatar image, storing it via the media layer, and associating the hash with the user's profile.

Phase 1 established the user record with pubkey. This feature adds the display name and avatar fields and the UI to manage them.

## Requirements
- [ ] Users can set a display name for the current server (1-32 characters, no leading/trailing whitespace)
- [ ] Users can upload an avatar image (JPEG, PNG, WebP; max 2 MB before processing)
- [ ] Uploaded avatars are resized server-side to a standard size (256x256) and stored as content-addressable blobs
- [ ] The user's `avatar_hash` field references the stored media blob
- [ ] Users can remove their avatar (revert to a default generated from their pubkey)
- [ ] Other members can view a user's profile (display name, avatar, roles, join date)
- [ ] Display names are shown in message views, member lists, and anywhere a user is referenced
- [ ] Profile updates are broadcast via the gateway so other clients update in real time
- [ ] A default avatar is generated deterministically from the user's pubkey (e.g., color-based identicon)

## Design

### API / Interface Changes

**REST endpoints** (all under `/api/v1/`):

| Method | Path | Description | Required Permission |
|--------|------|-------------|---------------------|
| GET | `/members/:pubkey/profile` | Get a member's profile | Authenticated (member) |
| PATCH | `/profile` | Update own display name | Authenticated (member) |
| PUT | `/profile/avatar` | Upload or replace own avatar | Authenticated (member) |
| DELETE | `/profile/avatar` | Remove own avatar | Authenticated (member) |
| GET | `/media/:hash` | Retrieve a media blob (avatar image) | Authenticated (member) |

**PATCH `/profile` request body:**
```json
{
  "display_name": "New Name"
}
```

**PUT `/profile/avatar`:** Multipart form upload with a single image file.

**Gateway events:**
- `MEMBER_UPDATE` — member's profile changed (display name or avatar)

### Data Model Changes

**Modified columns** on the existing `users` table:

```sql
ALTER TABLE users ADD COLUMN display_name TEXT;
ALTER TABLE users ADD COLUMN avatar_hash  TEXT;  -- references media store
```

If these columns already exist from Phase 1 scaffolding, no migration is needed — just ensure they are used.

**Media storage:** Avatars are stored via the existing `MediaStore` trait (Phase 1 storage feature). Each avatar is a content-addressable blob keyed by its SHA-256 or BLAKE3 hash. A metadata record maps the hash to MIME type and size.

### Component Changes

**Server (`server/`):**

- `server/src/routes/profile.rs` — REST handlers for profile update, avatar upload/delete
- `server/src/routes/media.rs` — Media serving endpoint (GET `/media/:hash`)
- `server/src/services/avatar.rs` — Avatar processing: validate format, resize to 256x256, compute hash, store via `MediaStore`
- Modify `server/src/models/user.rs` — Ensure `display_name` and `avatar_hash` fields are present
- Modify `server/src/store/user_store.rs` — Add methods for updating display_name and avatar_hash
- Modify `server/src/gateway/events.rs` — Add `MEMBER_UPDATE` event type

**Dependencies:** Add `image` crate for server-side image resizing.

**Client (`client/`):**

- `client/src/api/profile.ts` — API client functions for profile endpoints
- `client/src/components/profile/ProfilePopover.tsx` — Popover shown when clicking a user's name/avatar (shows display name, avatar, roles, join date)
- `client/src/components/profile/ProfileEditor.tsx` — Form for editing own display name and avatar (in user settings)
- `client/src/components/profile/Avatar.tsx` — Reusable avatar component that renders the user's avatar or a default identicon
- `client/src/components/profile/Identicon.tsx` — Deterministic default avatar generated from pubkey
- `client/src/utils/identicon.ts` — Generate a simple color-based identicon from a pubkey hash
- Modify `client/src/components/messages/MessageItem.tsx` — Show avatar and display name
- Modify `client/src/components/members/MemberList.tsx` — Show avatar and display name
- Modify `client/src/stores/members.ts` — Handle MEMBER_UPDATE events

## Task List

### Server
- [ ] Add or verify `display_name` and `avatar_hash` columns on the users table (migration if needed)
- [ ] Add update methods to `UserStore` trait for display_name and avatar_hash (`update_user` + new `set_avatar_hash`)
- [ ] Implement SQLite `UserStore` update methods
- [ ] Add `image` crate dependency for avatar processing
- [ ] Implement avatar processing service: validate image format (JPEG/PNG/WebP), resize to 256x256, compute SHA-256 content hash, store blob to disk (`server/src/profiles/avatar.rs`)
- [ ] Implement PATCH `/profile` handler (update display_name with validation)
- [ ] Implement PUT `/profile/avatar` handler (multipart upload, process, store, update user record)
- [ ] Implement DELETE `/profile/avatar` handler (clear avatar_hash)
- [ ] Implement GET `/media/:hash` handler for serving stored media blobs with appropriate Cache-Control headers
- [ ] Implement GET `/members/:pubkey/profile` handler
- [ ] Broadcast `MEMBER_UPDATE` event via gateway on profile changes

### Client
- [ ] Add profile API client functions
- [ ] Build Identicon component (deterministic color avatar from pubkey)
- [ ] Build reusable Avatar component (show avatar image or fall back to identicon)
- [ ] Update MessageItem to show avatar and display name
- [ ] Update MemberList to show avatar and display name
- [ ] Build ProfilePopover (click user to see profile details)
- [ ] Build ProfileEditor in user settings (edit display name, upload/remove avatar)
- [ ] Handle MEMBER_UPDATE gateway events in members store

## Test List
- [ ] Unit: display name validation (length, whitespace trimming)
- [ ] Unit: avatar processing resizes to 256x256 and produces correct hash
- [ ] Unit: identicon generation is deterministic for the same pubkey (client-side only)
- [ ] Integration: update display name via PATCH /profile, verify it persists
- [ ] Integration: upload avatar via PUT /profile/avatar, verify avatar_hash is set and media is retrievable via GET /media/:hash (requires real filesystem; covered by unit test in avatar.rs)
- [ ] Integration: delete avatar, verify avatar_hash is cleared
- [ ] Integration: MEMBER_UPDATE event is broadcast on profile change
- [ ] Integration: reject avatar uploads over 2 MB (validated in avatar.rs)
- [ ] Integration: reject non-image file uploads (validated in avatar.rs)
- [ ] Manual: avatar and display name appear in message view and member list
- [ ] Manual: profile popover displays correct information
- [ ] Manual: default identicon renders when no avatar is set

## Open Questions
- Should display names be unique within a server, or can multiple users have the same display name? Discord allows duplicates (disambiguated by username tag). Given pubkey-based identity, duplicates seem acceptable.
- Should we support animated avatars (GIF/APNG)? This adds complexity to processing and rendering. Probably not for the initial implementation.
- What identicon style should be used? Options include geometric patterns, color blocks, or a simple gradient from the pubkey hash.
