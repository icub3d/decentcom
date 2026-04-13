use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::Row;

use super::SqliteStorage;
use crate::storage::models::Channel;
use crate::storage::traits::ChannelStore;
use crate::storage::StorageError;

fn row_to_channel(row: sqlx::sqlite::SqliteRow) -> Result<Channel, StorageError> {
    Ok(Channel {
        id: row.try_get("id")?,
        name: row.try_get("name")?,
        topic: row.try_get("topic")?,
        category_id: row.try_get("category_id")?,
        position: row.try_get("position")?,
        created_at: row.try_get::<DateTime<Utc>, _>("created_at")?,
        updated_at: row.try_get::<DateTime<Utc>, _>("updated_at")?,
    })
}

#[async_trait]
impl ChannelStore for SqliteStorage {
    async fn create_channel(
        &self,
        name: &str,
        category_id: Option<&str>,
        position: i32,
    ) -> Result<Channel, StorageError> {
        let id = self.new_id();
        sqlx::query("INSERT INTO channels (id, name, category_id, position) VALUES (?, ?, ?, ?)")
            .bind(&id)
            .bind(name)
            .bind(category_id)
            .bind(position)
            .execute(self.pool())
            .await?;
        self.get_channel(&id).await?.ok_or(StorageError::NotFound)
    }

    async fn get_channel(&self, id: &str) -> Result<Option<Channel>, StorageError> {
        let row = sqlx::query("SELECT * FROM channels WHERE id = ?")
            .bind(id)
            .fetch_optional(self.pool())
            .await?;
        row.map(row_to_channel).transpose()
    }

    async fn list_channels(&self) -> Result<Vec<Channel>, StorageError> {
        let rows = sqlx::query("SELECT * FROM channels ORDER BY position ASC, created_at ASC")
            .fetch_all(self.pool())
            .await?;
        rows.into_iter().map(row_to_channel).collect()
    }

    async fn update_channel(
        &self,
        id: &str,
        name: Option<&str>,
        topic: Option<&str>,
        position: Option<i32>,
    ) -> Result<Channel, StorageError> {
        sqlx::query(
            "UPDATE channels
             SET name     = COALESCE(?, name),
                 topic    = COALESCE(?, topic),
                 position = COALESCE(?, position),
                 updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
             WHERE id = ?",
        )
        .bind(name)
        .bind(topic)
        .bind(position)
        .bind(id)
        .execute(self.pool())
        .await?;
        self.get_channel(id).await?.ok_or(StorageError::NotFound)
    }

    async fn delete_channel(&self, id: &str) -> Result<(), StorageError> {
        let result = sqlx::query("DELETE FROM channels WHERE id = ?")
            .bind(id)
            .execute(self.pool())
            .await?;
        if result.rows_affected() == 0 {
            return Err(StorageError::NotFound);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn channel_crud() {
        let s = SqliteStorage::in_memory().await.unwrap();
        let c = s.create_channel("general", None, 0).await.unwrap();
        assert_eq!(c.name, "general");

        let got = s.get_channel(&c.id).await.unwrap().unwrap();
        assert_eq!(got.id, c.id);

        let list = s.list_channels().await.unwrap();
        assert_eq!(list.len(), 1);

        let updated = s
            .update_channel(&c.id, Some("random"), Some("chat"), Some(5))
            .await
            .unwrap();
        assert_eq!(updated.name, "random");
        assert_eq!(updated.topic.as_deref(), Some("chat"));
        assert_eq!(updated.position, 5);

        s.delete_channel(&c.id).await.unwrap();
        assert!(s.get_channel(&c.id).await.unwrap().is_none());
    }
}
