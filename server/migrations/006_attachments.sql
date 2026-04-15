-- Attachments: link uploaded media to messages and channels.
CREATE TABLE attachments (
    id          TEXT PRIMARY KEY,
    message_id  TEXT REFERENCES messages(id),
    channel_id  TEXT NOT NULL REFERENCES channels(id),
    uploader_id TEXT NOT NULL REFERENCES users(id),
    filename    TEXT NOT NULL,
    content_hash TEXT NOT NULL,
    size        INTEGER NOT NULL,
    mime_type   TEXT NOT NULL,
    width       INTEGER,
    height      INTEGER,
    created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    deleted_at  TEXT
);

CREATE INDEX idx_attachments_message ON attachments(message_id);
CREATE INDEX idx_attachments_hash ON attachments(content_hash);
