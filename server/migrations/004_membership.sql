CREATE TABLE members (
    user_id    TEXT PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    joined_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE bans (
    pubkey     TEXT PRIMARY KEY,
    banned_by  TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    reason     TEXT,
    banned_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE allowlist (
    pubkey     TEXT PRIMARY KEY,
    added_by   TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    added_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX idx_members_joined_at ON members(joined_at);
CREATE INDEX idx_bans_banned_at ON bans(banned_at);
CREATE INDEX idx_allowlist_added_at ON allowlist(added_at);