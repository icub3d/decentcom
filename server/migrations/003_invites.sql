CREATE TABLE invites (
    code          TEXT PRIMARY KEY,
    created_by    TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    grant_role_id TEXT REFERENCES roles(id) ON DELETE SET NULL,
    max_uses      INTEGER NOT NULL DEFAULT 0,
    use_count     INTEGER NOT NULL DEFAULT 0,
    expires_at    TEXT,
    created_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX idx_invites_created_by ON invites(created_by);
CREATE INDEX idx_invites_expires_at ON invites(expires_at);