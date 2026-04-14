use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::Row;

use super::SqliteStorage;
use crate::storage::models::Invite;
use crate::storage::traits::InviteStore;
use crate::storage::StorageError;

fn row_to_invite(row: sqlx::sqlite::SqliteRow) -> Result<Invite, StorageError> {
    Ok(Invite {
        code: row.try_get("code")?,
        created_by: row.try_get("created_by")?,
        grant_role_id: row.try_get("grant_role_id")?,
        max_uses: row.try_get("max_uses")?,
        use_count: row.try_get("use_count")?,
        expires_at: row.try_get::<Option<DateTime<Utc>>, _>("expires_at")?,
        created_at: row.try_get::<DateTime<Utc>, _>("created_at")?,
    })
}

#[async_trait]
impl InviteStore for SqliteStorage {
    async fn create_invite(
        &self,
        code: &str,
        created_by: &str,
        grant_role_id: Option<&str>,
        max_uses: i64,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<Invite, StorageError> {
        sqlx::query(
            "INSERT INTO invites (code, created_by, grant_role_id, max_uses, expires_at)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(code)
        .bind(created_by)
        .bind(grant_role_id)
        .bind(max_uses)
        .bind(expires_at)
        .execute(self.pool())
        .await?;

        self.get_invite(code).await?.ok_or(StorageError::NotFound)
    }

    async fn get_invite(&self, code: &str) -> Result<Option<Invite>, StorageError> {
        let row = sqlx::query("SELECT * FROM invites WHERE code = ?")
            .bind(code)
            .fetch_optional(self.pool())
            .await?;
        row.map(row_to_invite).transpose()
    }

    async fn list_active_invites(&self, now: DateTime<Utc>) -> Result<Vec<Invite>, StorageError> {
        let rows = sqlx::query(
            "SELECT *
             FROM invites
             WHERE (expires_at IS NULL OR expires_at > ?)
               AND (max_uses = 0 OR use_count < max_uses)
             ORDER BY created_at DESC",
        )
        .bind(now)
        .fetch_all(self.pool())
        .await?;
        rows.into_iter().map(row_to_invite).collect()
    }

    async fn revoke_invite(&self, code: &str) -> Result<(), StorageError> {
        let result = sqlx::query("DELETE FROM invites WHERE code = ?")
            .bind(code)
            .execute(self.pool())
            .await?;
        if result.rows_affected() == 0 {
            return Err(StorageError::NotFound);
        }
        Ok(())
    }

    async fn consume_invite(&self, code: &str, now: DateTime<Utc>) -> Result<Invite, StorageError> {
        let updated = sqlx::query(
            "UPDATE invites
             SET use_count = use_count + 1
             WHERE code = ?
               AND (expires_at IS NULL OR expires_at > ?)
               AND (max_uses = 0 OR use_count < max_uses)",
        )
        .bind(code)
        .bind(now)
        .execute(self.pool())
        .await?;

        if updated.rows_affected() == 0 {
            let invite = self.get_invite(code).await?;
            let Some(invite) = invite else {
                return Err(StorageError::NotFound);
            };

            if invite.expires_at.is_some_and(|expires_at| expires_at <= now) {
                return Err(StorageError::Conflict("invite expired".to_string()));
            }

            if invite.max_uses > 0 && invite.use_count >= invite.max_uses {
                return Err(StorageError::Conflict("invite exhausted".to_string()));
            }

            return Err(StorageError::Conflict("invite cannot be consumed".to_string()));
        }

        self.get_invite(code).await?.ok_or(StorageError::NotFound)
    }

    async fn delete_expired_invites(&self, now: DateTime<Utc>) -> Result<u64, StorageError> {
        let result = sqlx::query("DELETE FROM invites WHERE expires_at IS NOT NULL AND expires_at <= ?")
            .bind(now)
            .execute(self.pool())
            .await?;
        Ok(result.rows_affected())
    }
}

#[cfg(test)]
mod tests {
    use chrono::Duration as ChronoDuration;

    use super::*;
    use crate::storage::traits::UserStore;

    #[tokio::test]
    async fn code_can_be_created_and_listed() {
        let storage = SqliteStorage::in_memory().await.unwrap();
        let creator = storage.create_user("pk-invite-creator", None).await.unwrap();

        let invite = storage
            .create_invite("abc12345", &creator.id, None, 1, None)
            .await
            .unwrap();
        assert_eq!(invite.code, "abc12345");

        let listed = storage.list_active_invites(Utc::now()).await.unwrap();
        assert_eq!(listed.len(), 1);
    }

    #[tokio::test]
    async fn single_use_invite_exhausts() {
        let storage = SqliteStorage::in_memory().await.unwrap();
        let creator = storage.create_user("pk-invite-consume", None).await.unwrap();

        storage
            .create_invite("consume01", &creator.id, None, 1, None)
            .await
            .unwrap();

        storage
            .consume_invite("consume01", Utc::now())
            .await
            .unwrap();

        let second = storage.consume_invite("consume01", Utc::now()).await;
        assert!(matches!(second, Err(StorageError::Conflict(_))));
    }

    #[tokio::test]
    async fn expiry_is_checked() {
        let storage = SqliteStorage::in_memory().await.unwrap();
        let creator = storage.create_user("pk-invite-expiry", None).await.unwrap();

        let expires_at = Utc::now() - ChronoDuration::seconds(1);
        storage
            .create_invite("expired01", &creator.id, None, 0, Some(expires_at))
            .await
            .unwrap();

        let consume = storage.consume_invite("expired01", Utc::now()).await;
        assert!(matches!(consume, Err(StorageError::Conflict(_))));

        let active = storage.list_active_invites(Utc::now()).await.unwrap();
        assert!(active.is_empty());
    }
}
