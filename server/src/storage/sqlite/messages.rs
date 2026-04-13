use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::Row;

use super::SqliteStorage;
use crate::storage::models::Message;
use crate::storage::traits::MessageStore;
use crate::storage::StorageError;

fn row_to_message(row: sqlx::sqlite::SqliteRow) -> Result<Message, StorageError> {
    let deleted_int: i64 = row.try_get("deleted")?;
    Ok(Message {
        id: row.try_get("id")?,
        channel_id: row.try_get("channel_id")?,
        author_id: row.try_get("author_id")?,
        content: row.try_get("content")?,
        edited_at: row.try_get::<Option<DateTime<Utc>>, _>("edited_at")?,
        deleted: deleted_int != 0,
        created_at: row.try_get::<DateTime<Utc>, _>("created_at")?,
    })
}

#[async_trait]
impl MessageStore for SqliteStorage {
    async fn create_message(
        &self,
        channel_id: &str,
        author_id: &str,
        content: &str,
    ) -> Result<Message, StorageError> {
        let id = self.new_id();
        sqlx::query("INSERT INTO messages (id, channel_id, author_id, content) VALUES (?, ?, ?, ?)")
            .bind(&id)
            .bind(channel_id)
            .bind(author_id)
            .bind(content)
            .execute(self.pool())
            .await?;
        self.get_message(&id).await?.ok_or(StorageError::NotFound)
    }

    async fn get_message(&self, id: &str) -> Result<Option<Message>, StorageError> {
        let row = sqlx::query("SELECT * FROM messages WHERE id = ? AND deleted = 0")
            .bind(id)
            .fetch_optional(self.pool())
            .await?;
        row.map(row_to_message).transpose()
    }

    async fn list_messages(
        &self,
        channel_id: &str,
        before: Option<&str>,
        limit: u32,
    ) -> Result<Vec<Message>, StorageError> {
        let limit = limit.min(500) as i64;
        let rows = match before {
            Some(cursor) => {
                sqlx::query(
                    "SELECT * FROM messages
                     WHERE channel_id = ? AND deleted = 0 AND id < ?
                     ORDER BY id DESC
                     LIMIT ?",
                )
                .bind(channel_id)
                .bind(cursor)
                .bind(limit)
                .fetch_all(self.pool())
                .await?
            }
            None => {
                sqlx::query(
                    "SELECT * FROM messages
                     WHERE channel_id = ? AND deleted = 0
                     ORDER BY id DESC
                     LIMIT ?",
                )
                .bind(channel_id)
                .bind(limit)
                .fetch_all(self.pool())
                .await?
            }
        };
        rows.into_iter().map(row_to_message).collect()
    }

    async fn update_message(&self, id: &str, content: &str) -> Result<Message, StorageError> {
        let result = sqlx::query(
            "UPDATE messages
             SET content = ?,
                 edited_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
             WHERE id = ? AND deleted = 0",
        )
        .bind(content)
        .bind(id)
        .execute(self.pool())
        .await?;
        if result.rows_affected() == 0 {
            return Err(StorageError::NotFound);
        }
        self.get_message(id).await?.ok_or(StorageError::NotFound)
    }

    async fn delete_message(&self, id: &str) -> Result<(), StorageError> {
        let result = sqlx::query("UPDATE messages SET deleted = 1 WHERE id = ?")
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
    use crate::storage::traits::{ChannelStore, UserStore};

    async fn setup() -> (SqliteStorage, String, String) {
        let s = SqliteStorage::in_memory().await.unwrap();
        let u = s.create_user("pk", None).await.unwrap();
        let c = s.create_channel("general", None, 0).await.unwrap();
        (s, u.id, c.id)
    }

    #[tokio::test]
    async fn create_and_get_message() {
        let (s, uid, cid) = setup().await;
        let m = s.create_message(&cid, &uid, "hi").await.unwrap();
        assert_eq!(m.content, "hi");
        assert!(!m.deleted);

        let got = s.get_message(&m.id).await.unwrap().unwrap();
        assert_eq!(got.id, m.id);

        let list = s.list_messages(&cid, None, 50).await.unwrap();
        assert_eq!(list.len(), 1);
    }

    #[tokio::test]
    async fn pagination_returns_correct_pages() {
        let (s, uid, cid) = setup().await;
        let mut ids = Vec::new();
        for i in 0..5 {
            let m = s
                .create_message(&cid, &uid, &format!("msg {i}"))
                .await
                .unwrap();
            ids.push(m.id);
        }
        // Page 1: newest 2
        let page1 = s.list_messages(&cid, None, 2).await.unwrap();
        assert_eq!(page1.len(), 2);
        assert_eq!(page1[0].id, ids[4]);
        assert_eq!(page1[1].id, ids[3]);

        // Page 2: next 2, before the oldest id on page 1
        let cursor = page1.last().unwrap().id.clone();
        let page2 = s.list_messages(&cid, Some(&cursor), 2).await.unwrap();
        assert_eq!(page2.len(), 2);
        assert_eq!(page2[0].id, ids[2]);
        assert_eq!(page2[1].id, ids[1]);
    }

    #[tokio::test]
    async fn soft_delete_hides_message() {
        let (s, uid, cid) = setup().await;
        let m = s.create_message(&cid, &uid, "bye").await.unwrap();
        s.delete_message(&m.id).await.unwrap();
        assert!(s.get_message(&m.id).await.unwrap().is_none());
        let list = s.list_messages(&cid, None, 50).await.unwrap();
        assert!(list.is_empty());
    }
}
