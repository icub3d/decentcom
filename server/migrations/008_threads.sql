CREATE TABLE threads (
    id                TEXT PRIMARY KEY,
    channel_id        TEXT NOT NULL REFERENCES channels(id),
    parent_message_id TEXT NOT NULL UNIQUE REFERENCES messages(id),
    creator_id        TEXT NOT NULL REFERENCES users(id),
    created_at        TEXT NOT NULL DEFAULT (datetime('now')),
    last_reply_at     TEXT,
    reply_count       INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_threads_channel ON threads(channel_id);
CREATE INDEX idx_threads_parent ON threads(parent_message_id);

CREATE TABLE thread_followers (
    thread_id       TEXT NOT NULL REFERENCES threads(id),
    user_id         TEXT NOT NULL REFERENCES users(id),
    last_read_at    TEXT,
    PRIMARY KEY (thread_id, user_id)
);

ALTER TABLE messages ADD COLUMN thread_id TEXT REFERENCES threads(id);
CREATE INDEX idx_messages_thread ON messages(thread_id);
