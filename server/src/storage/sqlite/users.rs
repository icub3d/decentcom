use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::Row;

use super::SqliteStorage;
use crate::storage::models::User;
use crate::storage::traits::UserStore;
use crate::storage::StorageError;

fn row_to_user(row: sqlx::sqlite::SqliteRow) -> Result<User, StorageError> {
    let is_bot_int: i64 = row.try_get("is_bot")?;
    Ok(User {
        id: row.try_get("id")?,
        pubkey: row.try_get("pubkey")?,
        display_name: row.try_get("display_name")?,
        avatar_hash: row.try_get("avatar_hash")?,
        is_bot: is_bot_int != 0,
        created_at: row.try_get::<DateTime<Utc>, _>("created_at")?,
        updated_at: row.try_get::<DateTime<Utc>, _>("updated_at")?,
    })
}

#[async_trait]
impl UserStore for SqliteStorage {
    async fn create_user(
        &self,
        pubkey: &str,
        display_name: Option<&str>,
        is_bot: bool,
    ) -> Result<User, StorageError> {
        let id = self.new_id();
        sqlx::query(
            "INSERT INTO users (id, pubkey, display_name, is_bot) VALUES (?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(pubkey)
        .bind(display_name)
        .bind(is_bot as i64)
        .execute(self.pool())
        .await?;
        self.get_user_by_id(&id)
            .await?
            .ok_or(StorageError::NotFound)
    }

    async fn get_user_by_id(&self, id: &str) -> Result<Option<User>, StorageError> {
        let row = sqlx::query("SELECT * FROM users WHERE id = ?")
            .bind(id)
            .fetch_optional(self.pool())
            .await?;
        row.map(row_to_user).transpose()
    }

    async fn get_user_by_pubkey(&self, pubkey: &str) -> Result<Option<User>, StorageError> {
        let row = sqlx::query("SELECT * FROM users WHERE pubkey = ?")
            .bind(pubkey)
            .fetch_optional(self.pool())
            .await?;
        row.map(row_to_user).transpose()
    }

    async fn update_user(
        &self,
        id: &str,
        display_name: Option<&str>,
        avatar_hash: Option<&str>,
    ) -> Result<User, StorageError> {
        sqlx::query(
            "UPDATE users
             SET display_name = COALESCE(?, display_name),
                 avatar_hash  = COALESCE(?, avatar_hash),
                 updated_at   = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
             WHERE id = ?",
        )
        .bind(display_name)
        .bind(avatar_hash)
        .bind(id)
        .execute(self.pool())
        .await?;
        self.get_user_by_id(id)
            .await?
            .ok_or(StorageError::NotFound)
    }

    async fn clear_avatar_hash(&self, id: &str) -> Result<User, StorageError> {
        sqlx::query(
            "UPDATE users
             SET avatar_hash = NULL,
                 updated_at  = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
             WHERE id = ?",
        )
        .bind(id)
        .execute(self.pool())
        .await?;
        self.get_user_by_id(id)
            .await?
            .ok_or(StorageError::NotFound)
    }

    async fn list_users(&self) -> Result<Vec<User>, StorageError> {
        let rows = sqlx::query("SELECT * FROM users ORDER BY created_at ASC")
            .fetch_all(self.pool())
            .await?;
        rows.into_iter().map(row_to_user).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn user_crud() {
        let s = SqliteStorage::in_memory().await.unwrap();
        let u = s.create_user("pk1", Some("alice"), false).await.unwrap();
        assert_eq!(u.pubkey, "pk1");
        assert_eq!(u.display_name.as_deref(), Some("alice"));
        assert!(!u.is_bot);

        let by_id = s.get_user_by_id(&u.id).await.unwrap().unwrap();
        assert_eq!(by_id, u);

        let by_pk = s.get_user_by_pubkey("pk1").await.unwrap().unwrap();
        assert_eq!(by_pk.id, u.id);

        let updated = s
            .update_user(&u.id, Some("alice2"), Some("hash"))
            .await
            .unwrap();
        assert_eq!(updated.display_name.as_deref(), Some("alice2"));
        assert_eq!(updated.avatar_hash.as_deref(), Some("hash"));

        let list = s.list_users().await.unwrap();
        assert_eq!(list.len(), 1);
    }

    #[tokio::test]
    async fn bot_user_is_flagged() {
        let s = SqliteStorage::in_memory().await.unwrap();
        let bot = s.create_user("pk-bot", None, true).await.unwrap();
        assert!(bot.is_bot);
    }

    #[tokio::test]
    async fn duplicate_pubkey_is_conflict() {
        let s = SqliteStorage::in_memory().await.unwrap();
        s.create_user("dup", None, false).await.unwrap();
        let err = s.create_user("dup", None, false).await.unwrap_err();
        assert!(matches!(err, StorageError::Conflict(_)));
    }
}
