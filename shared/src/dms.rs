use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceKey {
    pub device_id: String,
    pub pubkey: String, // Ed25519 public key
    pub x25519_pubkey: String, // X25519 public key for DM E2EE
    pub device_name: Option<String>,
    pub issued_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PendingDM {
    pub id: String,
    pub sender_pubkey: String,
    pub group_id: String,
    pub encrypted_blob: String, // Base64 AES-GCM
    pub created_at: String,
    pub ttl_days: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DMAck {
    pub message_id: String,
    pub device_id: String,
}
