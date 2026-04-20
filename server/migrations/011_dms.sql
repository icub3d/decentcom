CREATE TABLE pending_dms (
    id TEXT PRIMARY KEY,
    sender_pubkey TEXT NOT NULL,
    group_id TEXT NOT NULL,
    encrypted_blob TEXT NOT NULL,
    ttl_days INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE pending_dm_targets (
    dm_id TEXT NOT NULL REFERENCES pending_dms(id) ON DELETE CASCADE,
    device_id TEXT NOT NULL REFERENCES user_devices(id) ON DELETE CASCADE,
    PRIMARY KEY (dm_id, device_id)
);
CREATE INDEX idx_pending_dm_targets_device_id ON pending_dm_targets(device_id);

CREATE TABLE dm_acks (
    dm_id TEXT NOT NULL REFERENCES pending_dms(id) ON DELETE CASCADE,
    device_id TEXT NOT NULL REFERENCES user_devices(id) ON DELETE CASCADE,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    PRIMARY KEY (dm_id, device_id)
);
