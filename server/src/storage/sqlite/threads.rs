use async_trait::async_trait;
use sqlx::Row;

use super::SqliteStorage;
use crate::storage::models::{Thread, ThreadSummary};
use crate::storage::traits::ThreadStore;
use crate::storage::StorageError;

fn row_to_thread(row: sqlx::sqlite::SqliteRow) -> Result<Thread, StorageError> {
    Ok(Thread {
        id: row.try_get("id")?,
        channel_id: row.try_get("channel_id")?,
        parent_message_id: row.try_get("parent_message_id")?,
        creator_id: row.try_get("creator_id")?,
        created_at: row.try_get("created_at")?,
        last_reply_at: row.try_get("last_reply_at")?,
        reply_count: row.try_get("reply_count")?,
    })
}

#[async_trait]
impl ThreadStore for SqliteStorage {
    async fn create_thread(
        &self,
        channel_id: &str,
        parent_message_id: &str,
        creator_id: &str,
    ) -> Result<Thread, StorageError> {
        let id = self.new_id();
        sqlx::query(
            "INSERT INTO threads (id, channel_id, parent_message_id, creator_id) VALUES (?, ?, ?, ?)
             ON CONFLICT(parent_message_id) DO UPDATE SET id=id"
        )
        .bind(&id)
        .bind(channel_id)
        .bind(parent_message_id)
        .bind(creator_id)
        .execute(self.pool())
        .await?;

        let row = sqlx::query(
            "SELECT id, channel_id, parent_message_id, creator_id, created_at, last_reply_at, reply_count FROM threads WHERE parent_message_id = ?"
        )
        .bind(parent_message_id)
        .fetch_one(self.pool())
        .await?;

        Ok(row_to_thread(row)?)
    }

    async fn get_thread(&self, id: &str) -> Result<Option<Thread>, StorageError> {
        let row = sqlx::query(
            "SELECT id, channel_id, parent_message_id, creator_id, created_at, last_reply_at, reply_count FROM threads WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(self.pool())
        .await?;
        row.map(row_to_thread).transpose()
    }

    async fn get_thread_by_parent(&self, parent_message_id: &str) -> Result<Option<Thread>, StorageError> {
        let row = sqlx::query(
            "SELECT id, channel_id, parent_message_id, creator_id, created_at, last_reply_at, reply_count FROM threads WHERE parent_message_id = ?"
        )
        .bind(parent_message_id)
        .fetch_optional(self.pool())
        .await?;
        row.map(row_to_thread).transpose()
    }

    async fn get_thread_summary(&self, parent_message_id: &str) -> Result<Option<ThreadSummary>, StorageError> {
        let row = sqlx::query(
            "SELECT t.id as thread_id, t.reply_count, t.last_reply_at, m.author_id as last_reply_author_id
             FROM threads t
             LEFT JOIN messages m ON m.thread_id = t.id AND m.created_at = t.last_reply_at
             WHERE t.parent_message_id = ?
             LIMIT 1"
        )
        .bind(parent_message_id)
        .fetch_optional(self.pool())
        .await?;

        Ok(row.map(|r| ThreadSummary {
            thread_id: r.get("thread_id"),
            reply_count: r.get("reply_count"),
            last_reply_at: r.get("last_reply_at"),
            last_reply_author_id: r.get("last_reply_author_id"),
        }))
    }

    async fn list_thread_summaries(&self, parent_message_ids: &[String]) -> Result<Vec<ThreadSummary>, StorageError> {
        if parent_message_ids.is_empty() {
            return Ok(Vec::new());
        }

        let mut query_builder = sqlx::QueryBuilder::new(
            "SELECT t.parent_message_id, t.id as thread_id, t.reply_count, t.last_reply_at, m.author_id as last_reply_author_id
             FROM threads t
             LEFT JOIN messages m ON m.thread_id = t.id AND m.created_at = t.last_reply_at
             WHERE t.parent_message_id IN ("
        );

        let mut separated = query_builder.separated(", ");
        for id in parent_message_ids {
            separated.push_bind(id);
        }
        separated.push_unseparated(") ");

        let rows = query_builder.build().fetch_all(self.pool()).await?;

        Ok(rows.into_iter().map(|r| ThreadSummary {
            thread_id: r.get("thread_id"),
            reply_count: r.get("reply_count"),
            last_reply_at: r.get("last_reply_at"),
            last_reply_author_id: r.get("last_reply_author_id"),
        }).collect())
    }

    async fn add_thread_follower(&self, thread_id: &str, user_id: &str) -> Result<(), StorageError> {
        sqlx::query(
            "INSERT INTO thread_followers (thread_id, user_id) VALUES (?, ?)
             ON CONFLICT(thread_id, user_id) DO NOTHING"
        )
        .bind(thread_id)
        .bind(user_id)
        .execute(self.pool())
        .await?;
        Ok(())
    }

    async fn remove_thread_follower(&self, thread_id: &str, user_id: &str) -> Result<(), StorageError> {
        sqlx::query("DELETE FROM thread_followers WHERE thread_id = ? AND user_id = ?")
            .bind(thread_id)
            .bind(user_id)
            .execute(self.pool())
            .await?;
        Ok(())
    }

    async fn is_thread_follower(&self, thread_id: &str, user_id: &str) -> Result<bool, StorageError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM thread_followers WHERE thread_id = ? AND user_id = ?")
            .bind(thread_id)
            .bind(user_id)
            .fetch_one(self.pool())
            .await?;
        Ok(count > 0)
    }

    async fn list_thread_followers(&self, thread_id: &str) -> Result<Vec<String>, StorageError> {
        let rows = sqlx::query("SELECT user_id FROM thread_followers WHERE thread_id = ?")
            .bind(thread_id)
            .fetch_all(self.pool())
            .await?;
        Ok(rows.into_iter().map(|r| r.get("user_id")).collect())
    }

    async fn update_thread_last_read(&self, thread_id: &str, user_id: &str) -> Result<(), StorageError> {
        sqlx::query(
            "UPDATE thread_followers SET last_read_at = datetime('now') WHERE thread_id = ? AND user_id = ?"
        )
        .bind(thread_id)
        .bind(user_id)
        .execute(self.pool())
        .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::traits::{ChannelStore, MessageStore, UserStore};

    async fn setup() -> (SqliteStorage, String, String, String) {
        let s = SqliteStorage::in_memory().await.unwrap();
        let u = s.create_user("pk", None, false).await.unwrap();
        let c = s.create_channel("general", None, 0).await.unwrap();
        let m = s.create_message(&c.id, &u.id, "root", None).await.unwrap();
        (s, u.id, c.id, m.id)
    }

    #[tokio::test]
    async fn create_thread_sets_metadata() {
        let (s, uid, cid, mid) = setup().await;
        let thread = s.create_thread(&cid, &mid, &uid).await.unwrap();
        assert_eq!(thread.channel_id, cid);
        assert_eq!(thread.parent_message_id, mid);
        assert_eq!(thread.creator_id, uid);
        assert_eq!(thread.reply_count, 0);
    }

    #[tokio::test]
    async fn posting_to_thread_updates_metadata() {
        let (s, uid, cid, mid) = setup().await;
        let thread = s.create_thread(&cid, &mid, &uid).await.unwrap();
        
        let m1 = s.create_message(&cid, &uid, "reply 1", Some(&thread.id)).await.unwrap();
        let t_updated = s.get_thread(&thread.id).await.unwrap().unwrap();
        assert_eq!(t_updated.reply_count, 1);
        assert_eq!(t_updated.last_reply_at, Some(m1.created_at));

        let _m2 = s.create_message(&cid, &uid, "reply 2", Some(&thread.id)).await.unwrap();
        let t_updated2 = s.get_thread(&thread.id).await.unwrap().unwrap();
        assert_eq!(t_updated2.reply_count, 2);
    }

    #[tokio::test]
    async fn follow_unfollow_crud() {
        let (s, uid, cid, mid) = setup().await;
        let thread = s.create_thread(&cid, &mid, &uid).await.unwrap();

        assert!(!s.is_thread_follower(&thread.id, &uid).await.unwrap());
        s.add_thread_follower(&thread.id, &uid).await.unwrap();
        assert!(s.is_thread_follower(&thread.id, &uid).await.unwrap());

        let followers = s.list_thread_followers(&thread.id).await.unwrap();
        assert_eq!(followers, vec![uid.clone()]);

        s.remove_thread_follower(&thread.id, &uid).await.unwrap();
        assert!(!s.is_thread_follower(&thread.id, &uid).await.unwrap());
    }

    #[tokio::test]
    async fn thread_summaries_enrichment() {
        let (s, uid, cid, mid) = setup().await;
        let thread = s.create_thread(&cid, &mid, &uid).await.unwrap();
        let m1 = s.create_message(&cid, &uid, "reply", Some(&thread.id)).await.unwrap();

        let summaries = s.list_thread_summaries(std::slice::from_ref(&mid)).await.unwrap();
        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].thread_id, thread.id);
        assert_eq!(summaries[0].reply_count, 1);
        assert_eq!(summaries[0].last_reply_at, Some(m1.created_at));
        assert_eq!(summaries[0].last_reply_author_id, Some(uid));
    }
}
