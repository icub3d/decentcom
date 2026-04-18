use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::Row;

use super::SqliteStorage;
use crate::storage::models::BotApproval;
use crate::storage::traits::BotStore;
use crate::storage::StorageError;

fn row_to_bot_approval(row: sqlx::sqlite::SqliteRow) -> Result<BotApproval, StorageError> {
    Ok(BotApproval {
        pubkey: row.try_get("pubkey")?,
        approved_by: row.try_get("approved_by")?,
        approved_at: row.try_get::<Option<DateTime<Utc>>, _>("approved_at")?,
        revoked_at: row.try_get::<Option<DateTime<Utc>>, _>("revoked_at")?,
        note: row.try_get("note")?,
    })
}

#[async_trait]
impl BotStore for SqliteStorage {
    async fn upsert_bot_pending(&self, pubkey: &str) -> Result<BotApproval, StorageError> {
        sqlx::query(
            "INSERT INTO bot_approvals (pubkey) VALUES (?)
             ON CONFLICT(pubkey) DO NOTHING",
        )
        .bind(pubkey)
        .execute(self.pool())
        .await?;

        self.get_bot_approval(pubkey)
            .await?
            .ok_or(StorageError::NotFound)
    }

    async fn approve_bot(
        &self,
        pubkey: &str,
        approved_by: &str,
        note: Option<&str>,
    ) -> Result<BotApproval, StorageError> {
        let now = Utc::now();
        let result = sqlx::query(
            "UPDATE bot_approvals
             SET approved_by = ?, approved_at = ?, revoked_at = NULL, note = COALESCE(?, note)
             WHERE pubkey = ?",
        )
        .bind(approved_by)
        .bind(now)
        .bind(note)
        .bind(pubkey)
        .execute(self.pool())
        .await?;

        if result.rows_affected() == 0 {
            return Err(StorageError::NotFound);
        }

        self.get_bot_approval(pubkey)
            .await?
            .ok_or(StorageError::NotFound)
    }

    async fn revoke_bot(&self, pubkey: &str) -> Result<BotApproval, StorageError> {
        let now = Utc::now();
        let result = sqlx::query(
            "UPDATE bot_approvals
             SET revoked_at = ?, approved_at = NULL, approved_by = NULL
             WHERE pubkey = ?",
        )
        .bind(now)
        .bind(pubkey)
        .execute(self.pool())
        .await?;

        if result.rows_affected() == 0 {
            return Err(StorageError::NotFound);
        }

        self.get_bot_approval(pubkey)
            .await?
            .ok_or(StorageError::NotFound)
    }

    async fn get_bot_approval(&self, pubkey: &str) -> Result<Option<BotApproval>, StorageError> {
        let row = sqlx::query("SELECT * FROM bot_approvals WHERE pubkey = ?")
            .bind(pubkey)
            .fetch_optional(self.pool())
            .await?;
        row.map(row_to_bot_approval).transpose()
    }

    async fn list_pending_bots(&self) -> Result<Vec<BotApproval>, StorageError> {
        let rows = sqlx::query(
            "SELECT * FROM bot_approvals WHERE approved_at IS NULL AND revoked_at IS NULL
             ORDER BY rowid ASC",
        )
        .fetch_all(self.pool())
        .await?;
        rows.into_iter().map(row_to_bot_approval).collect()
    }

    async fn is_bot_approved(&self, pubkey: &str) -> Result<bool, StorageError> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM bot_approvals
             WHERE pubkey = ? AND approved_at IS NOT NULL AND revoked_at IS NULL",
        )
        .bind(pubkey)
        .fetch_one(self.pool())
        .await?;
        Ok(count > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::traits::UserStore;

    #[tokio::test]
    async fn bot_approval_lifecycle() {
        let s = SqliteStorage::in_memory().await.unwrap();
        let admin = s.create_user("pk-admin", None, false).await.unwrap();
        let _bot = s.create_user("pk-bot", None, true).await.unwrap();

        // Upsert as pending
        let pending = s.upsert_bot_pending("pk-bot").await.unwrap();
        assert!(pending.approved_at.is_none());
        assert!(pending.revoked_at.is_none());

        // Not approved yet
        assert!(!s.is_bot_approved("pk-bot").await.unwrap());

        // Shows in pending list
        let pending_list = s.list_pending_bots().await.unwrap();
        assert_eq!(pending_list.len(), 1);
        assert_eq!(pending_list[0].pubkey, "pk-bot");

        // Approve
        let approved = s.approve_bot("pk-bot", &admin.id, Some("looks good")).await.unwrap();
        assert!(approved.approved_at.is_some());
        assert!(approved.revoked_at.is_none());
        assert!(s.is_bot_approved("pk-bot").await.unwrap());

        // No longer in pending list
        let pending_list = s.list_pending_bots().await.unwrap();
        assert!(pending_list.is_empty());

        // Revoke
        let revoked = s.revoke_bot("pk-bot").await.unwrap();
        assert!(revoked.revoked_at.is_some());
        assert!(!s.is_bot_approved("pk-bot").await.unwrap());
    }

    #[tokio::test]
    async fn upsert_bot_pending_is_idempotent() {
        let s = SqliteStorage::in_memory().await.unwrap();
        let _bot = s.create_user("pk-bot2", None, true).await.unwrap();

        s.upsert_bot_pending("pk-bot2").await.unwrap();
        s.upsert_bot_pending("pk-bot2").await.unwrap(); // should not fail
        let pending_list = s.list_pending_bots().await.unwrap();
        assert_eq!(pending_list.len(), 1);
    }
}
