use async_trait::async_trait;
use shared::dms::PendingDM;

use super::SqliteStorage;
use crate::storage::{traits::DmStore, StorageError};

#[async_trait]
impl DmStore for SqliteStorage {
    async fn store_dm(
        &self,
        dm: PendingDM,
        target_device_ids: &[String],
    ) -> Result<(), StorageError> {
        let mut tx = self.pool().begin().await?;

        sqlx::query!(
            r#"
            INSERT INTO pending_dms (id, sender_pubkey, group_id, encrypted_blob, ttl_days, created_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
            dm.id,
            dm.sender_pubkey,
            dm.group_id,
            dm.encrypted_blob,
            dm.ttl_days,
            dm.created_at
        )
        .execute(&mut *tx)
        .await?;

        for device_id in target_device_ids {
            sqlx::query!(
                r#"
                INSERT INTO pending_dm_targets (dm_id, device_id)
                VALUES (?, ?)
                "#,
                dm.id,
                device_id
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    async fn fetch_pending_dms(&self, device_id: &str) -> Result<Vec<PendingDM>, StorageError> {
        let dms = sqlx::query_as!(
            PendingDM,
            r#"
            SELECT
                d.id as "id!",
                d.sender_pubkey as "sender_pubkey!",
                d.group_id as "group_id!",
                d.encrypted_blob as "encrypted_blob!",
                d.ttl_days as "ttl_days: u32",
                d.created_at as "created_at!"
            FROM pending_dms d
            JOIN pending_dm_targets t ON d.id = t.dm_id
            WHERE t.device_id = ?
              AND NOT EXISTS (
                  SELECT 1 FROM dm_acks a
                  WHERE a.dm_id = d.id AND a.device_id = ?
              )
            ORDER BY d.created_at ASC
            "#,
            device_id,
            device_id
        )
        .fetch_all(self.pool())
        .await?;
        Ok(dms)
    }

    async fn ack_dm(&self, dm_id: &str, device_id: &str) -> Result<(), StorageError> {
        sqlx::query!(
            r#"
            INSERT OR IGNORE INTO dm_acks (dm_id, device_id)
            VALUES (?, ?)
            "#,
            dm_id,
            device_id
        )
        .execute(self.pool())
        .await?;
        Ok(())
    }

    async fn prune_dms(&self) -> Result<u64, StorageError> {
        let result = sqlx::query!(
            r#"
            DELETE FROM pending_dms
            WHERE id IN (
                SELECT id FROM pending_dms d
                WHERE (
                    -- All targeted devices have ACKed
                    (SELECT count(*) FROM pending_dm_targets t WHERE t.dm_id = d.id) =
                    (SELECT count(*) FROM dm_acks a WHERE a.dm_id = d.id)
                ) OR (
                    -- OR TTL has expired
                    unixepoch(d.created_at) + (d.ttl_days * 86400) < unixepoch('now')
                )
            )
            "#
        )
        .execute(self.pool())
        .await?;

        // NOTE: the result.rows_affected() might be 0 if the query is a NO-OP, but returns the correct count.
        // It's returned as u64.
        Ok(result.rows_affected() as u64)
    }
}
