use async_trait::async_trait;
use sqlx::Row;

use super::SqliteStorage;
use crate::storage::models::{Reaction, ReactionCount};
use crate::storage::traits::ReactionStore;
use crate::storage::StorageError;

fn row_to_reaction(row: sqlx::sqlite::SqliteRow) -> Result<Reaction, StorageError> {
    Ok(Reaction {
        message_id: row.try_get("message_id")?,
        user_id: row.try_get("user_id")?,
        emoji: row.try_get("emoji")?,
        created_at: row.try_get("created_at")?,
    })
}

#[async_trait]
impl ReactionStore for SqliteStorage {
    async fn upsert_reaction(
        &self,
        message_id: &str,
        user_id: &str,
        emoji: &str,
    ) -> Result<Option<Reaction>, StorageError> {
        // Check if user already has this exact emoji — toggle (remove) it.
        let existing: Option<String> =
            sqlx::query_scalar("SELECT emoji FROM reactions WHERE message_id = ? AND user_id = ?")
                .bind(message_id)
                .bind(user_id)
                .fetch_optional(self.pool())
                .await?;

        if existing.as_deref() == Some(emoji) {
            sqlx::query("DELETE FROM reactions WHERE message_id = ? AND user_id = ?")
                .bind(message_id)
                .bind(user_id)
                .execute(self.pool())
                .await?;
            return Ok(None);
        }

        sqlx::query(
            "INSERT INTO reactions (message_id, user_id, emoji) VALUES (?, ?, ?)
             ON CONFLICT(message_id, user_id) DO UPDATE SET emoji = excluded.emoji, created_at = datetime('now')",
        )
        .bind(message_id)
        .bind(user_id)
        .bind(emoji)
        .execute(self.pool())
        .await?;

        let row = sqlx::query(
            "SELECT message_id, user_id, emoji, created_at FROM reactions WHERE message_id = ? AND user_id = ?",
        )
        .bind(message_id)
        .bind(user_id)
        .fetch_one(self.pool())
        .await?;

        Ok(Some(row_to_reaction(row)?))
    }

    async fn remove_reaction(&self, message_id: &str, user_id: &str) -> Result<(), StorageError> {
        sqlx::query("DELETE FROM reactions WHERE message_id = ? AND user_id = ?")
            .bind(message_id)
            .bind(user_id)
            .execute(self.pool())
            .await?;
        Ok(())
    }

    async fn list_reaction_counts(
        &self,
        message_id: &str,
        viewer_id: &str,
    ) -> Result<Vec<ReactionCount>, StorageError> {
        let rows = sqlx::query(
            "SELECT emoji,
                    COUNT(*) as count,
                    SUM(CASE WHEN user_id = ? THEN 1 ELSE 0 END) as me_count
             FROM reactions
             WHERE message_id = ?
             GROUP BY emoji
             ORDER BY MIN(created_at)",
        )
        .bind(viewer_id)
        .bind(message_id)
        .fetch_all(self.pool())
        .await?;

        rows.into_iter()
            .map(|row| {
                let me_count: i64 = row.try_get("me_count")?;
                Ok(ReactionCount {
                    emoji: row.try_get("emoji")?,
                    count: row.try_get("count")?,
                    me: me_count > 0,
                })
            })
            .collect()
    }

    async fn list_emoji_reactors(
        &self,
        message_id: &str,
        emoji: &str,
        limit: u32,
        before: Option<&str>,
    ) -> Result<Vec<Reaction>, StorageError> {
        let limit = limit.min(100) as i64;
        let rows = match before {
            Some(cursor) => {
                sqlx::query(
                    "SELECT message_id, user_id, emoji, created_at FROM reactions
                     WHERE message_id = ? AND emoji = ? AND user_id < ?
                     ORDER BY user_id DESC LIMIT ?",
                )
                .bind(message_id)
                .bind(emoji)
                .bind(cursor)
                .bind(limit)
                .fetch_all(self.pool())
                .await?
            }
            None => {
                sqlx::query(
                    "SELECT message_id, user_id, emoji, created_at FROM reactions
                     WHERE message_id = ? AND emoji = ?
                     ORDER BY created_at ASC LIMIT ?",
                )
                .bind(message_id)
                .bind(emoji)
                .bind(limit)
                .fetch_all(self.pool())
                .await?
            }
        };
        rows.into_iter().map(row_to_reaction).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::traits::{ChannelStore, MessageStore, UserStore};

    async fn setup() -> (SqliteStorage, String, String, String) {
        let s = SqliteStorage::in_memory().await.unwrap();
        let u1 = s.create_user("pk1", Some("Alice")).await.unwrap();
        let u2 = s.create_user("pk2", Some("Bob")).await.unwrap();
        let c = s.create_channel("general", None, 0).await.unwrap();
        let m = s.create_message(&c.id, &u1.id, "hello").await.unwrap();
        (s, u1.id, u2.id, m.id)
    }

    #[tokio::test]
    async fn add_reaction_creates_record() {
        let (s, u1, _u2, msg) = setup().await;
        let r = s.upsert_reaction(&msg, &u1, "👍").await.unwrap();
        assert!(r.is_some());
        let r = r.unwrap();
        assert_eq!(r.emoji, "👍");
        assert_eq!(r.user_id, u1);
    }

    #[tokio::test]
    async fn same_emoji_twice_toggles_off() {
        let (s, u1, _u2, msg) = setup().await;
        s.upsert_reaction(&msg, &u1, "👍").await.unwrap();
        let result = s.upsert_reaction(&msg, &u1, "👍").await.unwrap();
        assert!(result.is_none());
        let counts = s.list_reaction_counts(&msg, &u1).await.unwrap();
        assert!(counts.is_empty());
    }

    #[tokio::test]
    async fn different_emoji_replaces_reaction() {
        let (s, u1, _u2, msg) = setup().await;
        s.upsert_reaction(&msg, &u1, "👍").await.unwrap();
        s.upsert_reaction(&msg, &u1, "❤️").await.unwrap();
        let counts = s.list_reaction_counts(&msg, &u1).await.unwrap();
        assert_eq!(counts.len(), 1);
        assert_eq!(counts[0].emoji, "❤️");
    }

    #[tokio::test]
    async fn remove_reaction_deletes_record() {
        let (s, u1, _u2, msg) = setup().await;
        s.upsert_reaction(&msg, &u1, "👍").await.unwrap();
        s.remove_reaction(&msg, &u1).await.unwrap();
        let counts = s.list_reaction_counts(&msg, &u1).await.unwrap();
        assert!(counts.is_empty());
    }

    #[tokio::test]
    async fn aggregate_counts_and_me_flag() {
        let (s, u1, u2, msg) = setup().await;
        s.upsert_reaction(&msg, &u1, "👍").await.unwrap();
        s.upsert_reaction(&msg, &u2, "👍").await.unwrap();
        s.upsert_reaction(&msg, &u1, "❤️").await.unwrap();

        // u1 toggled 👍 off by adding ❤️ replacing it, but wait — u1 added 👍 then ❤️ replaces it
        // So u2 has 👍, u1 has ❤️
        let counts = s.list_reaction_counts(&msg, &u1).await.unwrap();
        let thumbs = counts.iter().find(|c| c.emoji == "👍").unwrap();
        assert_eq!(thumbs.count, 1);
        assert!(!thumbs.me); // u1 does not have 👍
        let heart = counts.iter().find(|c| c.emoji == "❤️").unwrap();
        assert_eq!(heart.count, 1);
        assert!(heart.me); // u1 has ❤️
    }

    #[tokio::test]
    async fn list_emoji_reactors_returns_users() {
        let (s, u1, u2, msg) = setup().await;
        s.upsert_reaction(&msg, &u1, "👍").await.unwrap();
        s.upsert_reaction(&msg, &u2, "👍").await.unwrap();
        let reactors = s.list_emoji_reactors(&msg, "👍", 10, None).await.unwrap();
        assert_eq!(reactors.len(), 2);
    }
}
