use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::Row;

use super::SqliteStorage;
use crate::storage::models::{AllowlistEntry, Ban, Member};
use crate::storage::traits::MemberStore;
use crate::storage::StorageError;

fn row_to_member(row: sqlx::sqlite::SqliteRow) -> Result<Member, StorageError> {
    Ok(Member {
        user_id: row.try_get("user_id")?,
        pubkey: row.try_get("pubkey")?,
        joined_at: row.try_get::<DateTime<Utc>, _>("joined_at")?,
    })
}

fn row_to_ban(row: sqlx::sqlite::SqliteRow) -> Result<Ban, StorageError> {
    Ok(Ban {
        pubkey: row.try_get("pubkey")?,
        banned_by: row.try_get("banned_by")?,
        reason: row.try_get("reason")?,
        banned_at: row.try_get::<DateTime<Utc>, _>("banned_at")?,
    })
}

fn row_to_allowlist_entry(row: sqlx::sqlite::SqliteRow) -> Result<AllowlistEntry, StorageError> {
    Ok(AllowlistEntry {
        pubkey: row.try_get("pubkey")?,
        added_by: row.try_get("added_by")?,
        added_at: row.try_get::<DateTime<Utc>, _>("added_at")?,
    })
}

#[async_trait]
impl MemberStore for SqliteStorage {
    async fn add_member(&self, user_id: &str) -> Result<Member, StorageError> {
        sqlx::query("INSERT INTO members (user_id) VALUES (?)")
            .bind(user_id)
            .execute(self.pool())
            .await?;

        let row = sqlx::query(
            "SELECT m.user_id, u.pubkey, m.joined_at
             FROM members m
             INNER JOIN users u ON u.id = m.user_id
             WHERE m.user_id = ?",
        )
        .bind(user_id)
        .fetch_optional(self.pool())
        .await?;

        row.map(row_to_member).transpose()?.ok_or(StorageError::NotFound)
    }

    async fn remove_member(&self, user_id: &str) -> Result<(), StorageError> {
        let result = sqlx::query("DELETE FROM members WHERE user_id = ?")
            .bind(user_id)
            .execute(self.pool())
            .await?;

        if result.rows_affected() == 0 {
            return Err(StorageError::NotFound);
        }

        Ok(())
    }

    async fn is_member(&self, user_id: &str) -> Result<bool, StorageError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM members WHERE user_id = ?")
            .bind(user_id)
            .fetch_one(self.pool())
            .await?;
        Ok(count > 0)
    }

    async fn list_members(&self) -> Result<Vec<Member>, StorageError> {
        let rows = sqlx::query(
            "SELECT m.user_id, u.pubkey, m.joined_at
             FROM members m
             INNER JOIN users u ON u.id = m.user_id
             ORDER BY m.joined_at ASC",
        )
        .fetch_all(self.pool())
        .await?;

        rows.into_iter().map(row_to_member).collect()
    }

    async fn get_member_by_pubkey(&self, pubkey: &str) -> Result<Option<Member>, StorageError> {
        let row = sqlx::query(
            "SELECT m.user_id, u.pubkey, m.joined_at
             FROM members m
             INNER JOIN users u ON u.id = m.user_id
             WHERE u.pubkey = ?",
        )
        .bind(pubkey)
        .fetch_optional(self.pool())
        .await?;

        row.map(row_to_member).transpose()
    }

    async fn add_ban(
        &self,
        pubkey: &str,
        banned_by: &str,
        reason: Option<&str>,
    ) -> Result<Ban, StorageError> {
        sqlx::query("INSERT OR REPLACE INTO bans (pubkey, banned_by, reason) VALUES (?, ?, ?)")
            .bind(pubkey)
            .bind(banned_by)
            .bind(reason)
            .execute(self.pool())
            .await?;

        let row = sqlx::query("SELECT pubkey, banned_by, reason, banned_at FROM bans WHERE pubkey = ?")
            .bind(pubkey)
            .fetch_optional(self.pool())
            .await?;

        row.map(row_to_ban).transpose()?.ok_or(StorageError::NotFound)
    }

    async fn remove_ban(&self, pubkey: &str) -> Result<(), StorageError> {
        let result = sqlx::query("DELETE FROM bans WHERE pubkey = ?")
            .bind(pubkey)
            .execute(self.pool())
            .await?;

        if result.rows_affected() == 0 {
            return Err(StorageError::NotFound);
        }

        Ok(())
    }

    async fn is_banned_pubkey(&self, pubkey: &str) -> Result<bool, StorageError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM bans WHERE pubkey = ?")
            .bind(pubkey)
            .fetch_one(self.pool())
            .await?;
        Ok(count > 0)
    }

    async fn list_bans(&self) -> Result<Vec<Ban>, StorageError> {
        let rows = sqlx::query("SELECT pubkey, banned_by, reason, banned_at FROM bans ORDER BY banned_at DESC")
            .fetch_all(self.pool())
            .await?;

        rows.into_iter().map(row_to_ban).collect()
    }

    async fn add_allowlist_entry(
        &self,
        pubkey: &str,
        added_by: &str,
    ) -> Result<AllowlistEntry, StorageError> {
        sqlx::query("INSERT OR REPLACE INTO allowlist (pubkey, added_by) VALUES (?, ?)")
            .bind(pubkey)
            .bind(added_by)
            .execute(self.pool())
            .await?;

        let row = sqlx::query("SELECT pubkey, added_by, added_at FROM allowlist WHERE pubkey = ?")
            .bind(pubkey)
            .fetch_optional(self.pool())
            .await?;

        row.map(row_to_allowlist_entry)
            .transpose()?
            .ok_or(StorageError::NotFound)
    }

    async fn remove_allowlist_entry(&self, pubkey: &str) -> Result<(), StorageError> {
        let result = sqlx::query("DELETE FROM allowlist WHERE pubkey = ?")
            .bind(pubkey)
            .execute(self.pool())
            .await?;

        if result.rows_affected() == 0 {
            return Err(StorageError::NotFound);
        }

        Ok(())
    }

    async fn is_allowlisted_pubkey(&self, pubkey: &str) -> Result<bool, StorageError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM allowlist WHERE pubkey = ?")
            .bind(pubkey)
            .fetch_one(self.pool())
            .await?;
        Ok(count > 0)
    }

    async fn list_allowlist_entries(&self) -> Result<Vec<AllowlistEntry>, StorageError> {
        let rows = sqlx::query("SELECT pubkey, added_by, added_at FROM allowlist ORDER BY added_at ASC")
            .fetch_all(self.pool())
            .await?;

        rows.into_iter().map(row_to_allowlist_entry).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::traits::UserStore;

    #[tokio::test]
    async fn member_ban_and_allowlist_crud() {
        let storage = SqliteStorage::in_memory().await.unwrap();
        let owner = storage.create_user("pk-owner", None).await.unwrap();
        let member = storage.create_user("pk-member", None).await.unwrap();

        let added = storage.add_member(&member.id).await.unwrap();
        assert_eq!(added.pubkey, "pk-member");
        assert!(storage.is_member(&member.id).await.unwrap());

        let banned = storage
            .add_ban("pk-member", &owner.id, Some("spam"))
            .await
            .unwrap();
        assert_eq!(banned.reason.as_deref(), Some("spam"));
        assert!(storage.is_banned_pubkey("pk-member").await.unwrap());

        let allow = storage
            .add_allowlist_entry("pk-allow", &owner.id)
            .await
            .unwrap();
        assert_eq!(allow.pubkey, "pk-allow");
        assert!(storage.is_allowlisted_pubkey("pk-allow").await.unwrap());

        storage.remove_member(&member.id).await.unwrap();
        assert!(!storage.is_member(&member.id).await.unwrap());

        storage.remove_ban("pk-member").await.unwrap();
        assert!(!storage.is_banned_pubkey("pk-member").await.unwrap());

        storage.remove_allowlist_entry("pk-allow").await.unwrap();
        assert!(!storage.is_allowlisted_pubkey("pk-allow").await.unwrap());
    }
}
