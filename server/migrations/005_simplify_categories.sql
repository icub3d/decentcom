-- Replace the categories table with a simple category string on channels.

CREATE TABLE channels_new (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL UNIQUE,
    topic       TEXT,
    category    TEXT,
    position    INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

-- Migrate data: join with categories to copy the category name.
INSERT INTO channels_new (id, name, topic, category, position, created_at, updated_at)
SELECT c.id, c.name, c.topic, cat.name, c.position, c.created_at, c.updated_at
FROM channels c
LEFT JOIN categories cat ON c.category_id = cat.id;

DROP TABLE channels;
ALTER TABLE channels_new RENAME TO channels;

CREATE INDEX idx_channels_category_position ON channels(category, position);

DROP TABLE categories;
