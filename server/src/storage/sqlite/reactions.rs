use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::Row;

use super::SqliteStorage;
use crate::storage::models::{Reaction, User};
use crate::storage::traits::ReactionStore;
use crate::storage::StorageError;

fn row_to_reaction(row: sqlx::sqlite::SqliteRow) -> Result<Reaction, StorageError> {
    Ok(Reaction {
        message_id: row.try_get("message_id")?,
        user_id: row.try_get("user_id")?,
        emoji: row.try_get("emoji")?,
        created_at: row.try_get::<DateTime<Utc>, _>("created_at")?,
    })
}

#[async_trait]
impl ReactionStore for SqliteStorage {
    async fn add_reaction(
        &self,
        message_id: &str,
        user_id: &str,
        emoji: &str,
    ) -> Result<Reaction, StorageError> {
        sqlx::query(
            "INSERT OR IGNORE INTO reactions (message_id, user_id, emoji) VALUES (?, ?, ?)",
        )
        .bind(message_id)
        .bind(user_id)
        .bind(emoji)
        .execute(self.pool())
        .await?;

        let row = sqlx::query("SELECT * FROM reactions WHERE message_id = ? AND user_id = ? AND emoji = ?")
            .bind(message_id)
            .bind(user_id)
            .bind(emoji)
            .fetch_one(self.pool())
            .await?;
        row_to_reaction(row)
    }

    async fn remove_reaction(
        &self,
        message_id: &str,
        user_id: &str,
        emoji: &str,
    ) -> Result<(), StorageError> {
        sqlx::query(
            "DELETE FROM reactions WHERE message_id = ? AND user_id = ? AND emoji = ?",
        )
        .bind(message_id)
        .bind(user_id)
        .bind(emoji)
        .execute(self.pool())
        .await?;
        Ok(())
    }

    async fn remove_user_reaction(
        &self,
        message_id: &str,
        user_id: &str,
        emoji: &str,
    ) -> Result<(), StorageError> {
        sqlx::query(
            "DELETE FROM reactions WHERE message_id = ? AND user_id = ? AND emoji = ?",
        )
        .bind(message_id)
        .bind(user_id)
        .bind(emoji)
        .execute(self.pool())
        .await?;
        Ok(())
    }

    async fn list_reactions_for_message(
        &self,
        message_id: &str,
    ) -> Result<Vec<Reaction>, StorageError> {
        let rows = sqlx::query("SELECT * FROM reactions WHERE message_id = ? ORDER BY created_at ASC")
            .bind(message_id)
            .fetch_all(self.pool())
            .await?;
        rows.into_iter().map(row_to_reaction).collect()
    }

    async fn count_reactions_for_message(
        &self,
        message_id: &str,
        current_user_id: &str,
    ) -> Result<Vec<(String, i64, bool)>, StorageError> {
        let rows = sqlx::query(
            "SELECT emoji, COUNT(*) as count, MAX(CASE WHEN user_id = ? THEN 1 ELSE 0 END) as me
             FROM reactions
             WHERE message_id = ?
             GROUP BY emoji
             ORDER BY emoji",
        )
        .bind(current_user_id)
        .bind(message_id)
        .fetch_all(self.pool())
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| {
                let emoji: String = row.try_get("emoji").unwrap_or_default();
                let count: i64 = row.try_get("count").unwrap_or(0);
                let me: i64 = row.try_get("me").unwrap_or(0);
                (emoji, count, me != 0)
            })
            .collect())
    }

    async fn list_users_for_reaction(
        &self,
        message_id: &str,
        emoji: &str,
    ) -> Result<Vec<User>, StorageError> {
        let rows = sqlx::query(
            "SELECT u.* FROM users u
             INNER JOIN reactions r ON u.id = r.user_id
             WHERE r.message_id = ? AND r.emoji = ?
             ORDER BY r.created_at ASC",
        )
        .bind(message_id)
        .bind(emoji)
        .fetch_all(self.pool())
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| User {
                id: row.try_get("id").unwrap_or_default(),
                pubkey: row.try_get("pubkey").unwrap_or_default(),
                display_name: row.try_get("display_name").ok(),
                avatar_hash: row.try_get("avatar_hash").ok(),
                created_at: row.try_get("created_at").unwrap_or_else(|_| Utc::now()),
                updated_at: row.try_get("updated_at").unwrap_or_else(|_| Utc::now()),
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::traits::{ChannelStore, MessageStore, UserStore};

    async fn setup() -> (SqliteStorage, String, String, String) {
        let s = SqliteStorage::in_memory().await.unwrap();
        let u1 = s.create_user("pk1", Some("user1")).await.unwrap();
        let u2 = s.create_user("pk2", Some("user2")).await.unwrap();
        let c = s.create_channel("general", None, 0).await.unwrap();
        (s, u1.id, u2.id, c.id)
    }

    #[tokio::test]
    async fn add_reaction_idempotent() {
        let (s, u1, _u2, cid) = setup().await;
        let msg = s.create_message(&cid, &u1, "hello").await.unwrap();

        let r1 = s.add_reaction(&msg.id, &u1, "👍").await.unwrap();
        assert_eq!(r1.emoji, "👍");
        assert_eq!(r1.user_id, u1);

        let r2 = s.add_reaction(&msg.id, &u1, "👍").await.unwrap();
        assert_eq!(r2.emoji, "👍");

        let reactions = s.list_reactions_for_message(&msg.id).await.unwrap();
        assert_eq!(reactions.len(), 1);
    }

    #[tokio::test]
    async fn remove_reaction() {
        let (s, u1, _u2, cid) = setup().await;
        let msg = s.create_message(&cid, &u1, "hello").await.unwrap();

        s.add_reaction(&msg.id, &u1, "👍").await.unwrap();
        let reactions = s.list_reactions_for_message(&msg.id).await.unwrap();
        assert_eq!(reactions.len(), 1);

        s.remove_reaction(&msg.id, &u1, "👍").await.unwrap();
        let reactions = s.list_reactions_for_message(&msg.id).await.unwrap();
        assert_eq!(reactions.len(), 0);
    }

    #[tokio::test]
    async fn count_reactions_with_me_flag() {
        let (s, u1, u2, cid) = setup().await;
        let msg = s.create_message(&cid, &u1, "hello").await.unwrap();

        s.add_reaction(&msg.id, &u1, "👍").await.unwrap();
        s.add_reaction(&msg.id, &u2, "👍").await.unwrap();
        s.add_reaction(&msg.id, &u1, "❤️").await.unwrap();

        let counts = s
            .count_reactions_for_message(&msg.id, &u1)
            .await
            .unwrap();
        assert_eq!(counts.len(), 2);

        let thumbs_up = counts.iter().find(|(e, _, _)| e == "👍").unwrap();
        assert_eq!(thumbs_up.1, 2);
        assert!(thumbs_up.2); // me=true for u1

        let heart = counts.iter().find(|(e, _, _)| e == "❤️").unwrap();
        assert_eq!(heart.1, 1);
        assert!(heart.2); // me=true for u1

        let counts_u2 = s
            .count_reactions_for_message(&msg.id, &u2)
            .await
            .unwrap();
        let heart_u2 = counts_u2.iter().find(|(e, _, _)| e == "❤️").unwrap();
        assert!(!heart_u2.2); // me=false for u2
    }

    #[tokio::test]
    async fn list_users_for_reaction() {
        let (s, u1, u2, cid) = setup().await;
        let msg = s.create_message(&cid, &u1, "hello").await.unwrap();

        s.add_reaction(&msg.id, &u1, "👍").await.unwrap();
        s.add_reaction(&msg.id, &u2, "👍").await.unwrap();

        let users = s
            .list_users_for_reaction(&msg.id, "👍")
            .await
            .unwrap();
        assert_eq!(users.len(), 2);
        assert!(users.iter().any(|u| u.id == u1));
        assert!(users.iter().any(|u| u.id == u2));
    }
}
