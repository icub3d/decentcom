ALTER TABLE users ADD COLUMN is_bot INTEGER NOT NULL DEFAULT 0;

CREATE TABLE bot_approvals (
    pubkey      TEXT PRIMARY KEY,
    approved_by TEXT REFERENCES users(id),
    approved_at TEXT,
    revoked_at  TEXT,
    note        TEXT
);

ALTER TABLE sessions ADD COLUMN is_read_only INTEGER NOT NULL DEFAULT 0;
