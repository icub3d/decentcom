# Feature: File Uploads

## Overview
Allow users to upload files and images to text channels with content-addressable storage, server-configurable size and type limits, and inline image previews in the client. This is a core communication feature that enables sharing screenshots, documents, and other media alongside text messages.

## Background
The storage design doc (`docs/design/storage.md`) defines content-addressable media storage: files are stored by their hash (SHA-256 or BLAKE3), with metadata in the relational database mapping `media_id -> content_hash`. Identical files are automatically deduplicated. The server model (`docs/design/server-model.md`) defines the `file_uploads` feature flag (enabled by default) and server config fields `max_file_size`, `allowed_file_types`, and the media retention policy. Media is served via the application server or pre-signed URLs for S3 backends.

## Requirements
- [ ] Users can upload one or more files alongside a message in a text channel or DM
- [ ] Files are stored content-addressably (hash-based key), deduplicating identical uploads
- [ ] Server enforces `max_file_size` from config (reject uploads exceeding the limit)
- [ ] Server enforces `allowed_file_types` from config (reject disallowed MIME types; empty list = all allowed)
- [ ] The `file_uploads` feature flag gates the entire upload capability; when disabled, upload endpoints return 403
- [ ] Uploaded images (JPEG, PNG, GIF, WebP) display as inline previews in the message view
- [ ] Non-image files display as a download link with filename, size, and MIME type
- [ ] Uploads are associated with the uploading user and the target channel
- [ ] Media files are served with appropriate `Cache-Control` headers for cacheability
- [ ] Deleting a message soft-deletes its attachments; orphaned media is purged by the retention system

## Design

### API / Interface Changes

**New REST endpoints:**

`POST /api/v1/channels/{channel_id}/attachments`
- Multipart form upload. Accepts one or more files.
- Returns a list of `attachment_id` values.
- Validates session token, channel permissions (`SEND_MESSAGES` + `ATTACH_FILES`), feature flag, file size, and MIME type.
- Response: `201 Created` with `[{ attachment_id, filename, content_hash, size, mime_type, url }]`

`GET /api/v1/media/{content_hash}`
- Serves the file bytes with correct `Content-Type` and `Cache-Control: public, max-age=31536000, immutable`.
- For S3 backends, redirects to a pre-signed URL (302).

**Modified endpoint:**

`POST /api/v1/channels/{channel_id}/messages`
- Add optional `attachment_ids: Vec<Uuid>` field to the message creation body.
- Attachments must have been uploaded by the same user and not yet associated with another message.

**New gateway event:**

`MESSAGE_CREATE` and `MESSAGE_UPDATE` events now include an `attachments` array in the payload.

**New Tauri IPC command:**

`upload_file(server_id, channel_id, file_path)` — Reads the file from disk via Tauri core (which has filesystem access), streams it to the server upload endpoint, returns the attachment metadata.

### Data Model Changes

**New table: `attachments`**

```sql
CREATE TABLE attachments (
    id          TEXT PRIMARY KEY,       -- UUID
    message_id  TEXT REFERENCES messages(id),  -- NULL until associated
    channel_id  TEXT NOT NULL REFERENCES channels(id),
    uploader_id TEXT NOT NULL REFERENCES users(id),
    filename    TEXT NOT NULL,
    content_hash TEXT NOT NULL,          -- SHA-256 or BLAKE3 hex
    size        INTEGER NOT NULL,       -- bytes
    mime_type   TEXT NOT NULL,
    width       INTEGER,                -- image width in px (NULL for non-images)
    height      INTEGER,                -- image height in px (NULL for non-images)
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    deleted_at  TEXT                     -- soft delete
);

CREATE INDEX idx_attachments_message ON attachments(message_id);
CREATE INDEX idx_attachments_hash ON attachments(content_hash);
```

**New entry in `MediaStore` trait:**

```rust
async fn store_media(&self, content_hash: &str, data: &[u8]) -> Result<()>;
async fn get_media(&self, content_hash: &str) -> Result<Option<Vec<u8>>>;
async fn media_exists(&self, content_hash: &str) -> Result<bool>;
```

### Component Changes

**Server (`server/`):**
- `server/src/routes/media.rs` — New module: upload endpoint, media serving endpoint
- `server/src/models/attachment.rs` — Attachment struct and DB operations
- `server/src/storage/media.rs` — MediaStore trait implementation for SQLite (local disk) backend
- `server/src/routes/messages.rs` — Modify message creation to accept and associate attachment IDs
- `server/src/config.rs` — Ensure `max_file_size`, `allowed_file_types`, `file_uploads` flag are parsed
- `server/src/gateway/events.rs` — Add attachments to message event payloads

**Client (`client/`):**
- `client/src-tauri/src/commands/upload.rs` — Tauri IPC command for file upload
- `client/src/components/MessageInput.tsx` — File picker button, drag-and-drop zone, upload progress indicator
- `client/src/components/MessageAttachment.tsx` — New component: renders inline image preview or file download link
- `client/src/components/Message.tsx` — Render attachments within message bubbles
- `client/src/api/media.ts` — API client functions for upload and media URL construction

## Task List

### Server
- [ ] Add `attachments` table to SQLite migrations (`server/migrations/007_attachments.sql`)
- [ ] Implement `Attachment` model struct and CRUD in `server/src/storage/sqlite/attachments.rs`
- [ ] Implement `AttachmentStore` trait and SQLite backend (content-addressable blobs via `server/src/media.rs`)
- [ ] Add `POST /api/v1/channels/{channel_id}/attachments` endpoint with multipart parsing, hash computation, size/type validation, and feature flag check (`server/src/attachments/handlers.rs`)
- [ ] Media is served via `GET /api/v1/media/{hash}` in existing profiles endpoint with correct Content-Type
- [ ] Modify `POST /api/v1/channels/{channel_id}/messages` to accept and associate `attachment_ids`
- [ ] `ATTACH_FILES` permission flag enforced on upload
- [ ] `attachments` array included in message responses via `enrich_messages`

### Client
- [ ] Add `upload_file` Tauri IPC command in `client/src-tauri/src/commands/upload.rs`
- [ ] Add media API client functions in `client/src/api/media.ts`
- [ ] Add file picker button and drag-and-drop to `MessageInput.tsx` with upload progress
- [ ] Create `MessageAttachment.tsx` component with inline image preview (thumbnail with lightbox) and file download link
- [ ] Integrate `MessageAttachment` into `Message.tsx` for rendering attachments
- [ ] Add upload size/type error handling and user feedback

## Test List
- [ ] Unit test: content hash computation produces correct hash for known input (storage/sqlite/attachments.rs)
- [ ] Unit test: MIME type detection from magic bytes (`server/src/media.rs`)
- [ ] Integration test: upload a file, verify metadata returned with correct filename/size/mime
- [ ] Integration test: create message with attachment IDs, verify attachments appear in response (`routes/attachment_tests.rs`)
- [ ] Integration test: upload rejected when file is empty (422)
- [ ] Integration test: upload rejected when `file_uploads` feature flag is disabled (not yet tested — flag check is in handler)
- [ ] Manual: drag-and-drop a file into the message input, verify upload progress and preview
- [ ] Manual: send a message with an image attachment, verify inline preview renders
- [ ] Manual: send a message with a non-image file, verify download link with correct filename and size

## Open Questions
- Should we use SHA-256 or BLAKE3 for content hashing? BLAKE3 is faster but SHA-256 is more widely understood. The storage doc mentions both.
- Should image thumbnails be generated server-side (for bandwidth savings) or should the client resize on display? Server-side thumbnails add complexity but improve performance for large images.
- What should the default `max_file_size` be? 10 MB is reasonable for a starting point.
- Should uploads be allowed in DM channels, or only in server text channels? The current design implies both.
