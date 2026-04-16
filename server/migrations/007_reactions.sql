CREATE TABLE reactions (
    message_id TEXT NOT NULL REFERENCES messages(id),
    user_id    TEXT NOT NULL REFERENCES users(id),
    emoji      TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (message_id, user_id)
);

CREATE INDEX idx_reactions_message ON reactions(message_id);
