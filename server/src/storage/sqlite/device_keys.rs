use async_trait::async_trait;
use shared::dms::DeviceKey;

use super::SqliteStorage;
use crate::storage::{traits::DeviceKeyStore, StorageError};

#[async_trait]
impl DeviceKeyStore for SqliteStorage {
    async fn add_device_key(&self, user_id: &str, device_key: DeviceKey) -> Result<(), StorageError> {
        let id = self.new_id();
        sqlx::query!(
            r#"
            INSERT INTO user_devices (id, user_id, pubkey, x25519_pubkey, device_name, issued_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
            id,
            user_id,
            device_key.pubkey,
            device_key.x25519_pubkey,
            device_key.device_name,
            device_key.issued_at
        )
        .execute(self.pool())
        .await?;
        Ok(())
    }

    async fn get_device_keys(&self, user_id: &str) -> Result<Vec<DeviceKey>, StorageError> {
        let keys = sqlx::query_as!(
            DeviceKey,
            r#"
            SELECT 
                id as "device_id!", 
                pubkey as "pubkey!", 
                x25519_pubkey as "x25519_pubkey!", 
                device_name, 
                issued_at as "issued_at!"
            FROM user_devices
            WHERE user_id = ?
            ORDER BY created_at ASC
            "#,
            user_id
        )
        .fetch_all(self.pool())
        .await?;
        Ok(keys)
    }
}
