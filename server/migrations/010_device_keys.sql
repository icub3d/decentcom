CREATE TABLE user_devices (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id),
    pubkey TEXT NOT NULL UNIQUE,
    x25519_pubkey TEXT NOT NULL,
    device_name TEXT,
    issued_at TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX idx_user_devices_user_id ON user_devices(user_id);
