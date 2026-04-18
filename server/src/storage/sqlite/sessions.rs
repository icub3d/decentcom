use async_trait::async_trait;
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use rand::RngCore;
use sqlx::Row;
use std::time::Duration;

use super::SqliteStorage;
use crate::storage::models::Session;
use crate::storage::traits::SessionStore;
use crate::storage::StorageError;

fn row_to_session(row: sqlx::sqlite::SqliteRow) -> Result<Session, StorageError> {
    let is_read_only_int: i64 = row.try_get("is_read_only")?;
    Ok(Session {
        token: row.try_get("token")?,
        user_id: row.try_get("user_id")?,
        is_read_only: is_read_only_int != 0,
        created_at: row.try_get::<DateTime<Utc>, _>("created_at")?,
        expires_at: row.try_get::<DateTime<Utc>, _>("expires_at")?,
    })
}

fn random_token() -> String {
    let mut bytes = [0u8; 16];
    rand::rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}

#[async_trait]
impl SessionStore for SqliteStorage {
    async fn create_session(
        &self,
        user_id: &str,
        duration: Duration,
        is_read_only: bool,
    ) -> Result<Session, StorageError> {
        let token = random_token();
        let expires_at: DateTime<Utc> = Utc::now()
            + ChronoDuration::from_std(duration)
                .map_err(|e| StorageError::Internal(e.to_string()))?;
        sqlx::query(
            "INSERT INTO sessions (token, user_id, expires_at, is_read_only) VALUES (?, ?, ?, ?)",
        )
        .bind(&token)
        .bind(user_id)
        .bind(expires_at)
        .bind(is_read_only as i64)
        .execute(self.pool())
        .await?;
        self.get_session(&token)
            .await?
            .ok_or(StorageError::NotFound)
    }

    async fn get_session(&self, token: &str) -> Result<Option<Session>, StorageError> {
        let row = sqlx::query("SELECT * FROM sessions WHERE token = ?")
            .bind(token)
            .fetch_optional(self.pool())
            .await?;
        row.map(row_to_session).transpose()
    }

    async fn delete_session(&self, token: &str) -> Result<(), StorageError> {
        let result = sqlx::query("DELETE FROM sessions WHERE token = ?")
            .bind(token)
            .execute(self.pool())
            .await?;
        if result.rows_affected() == 0 {
            return Err(StorageError::NotFound);
        }
        Ok(())
    }

    async fn delete_expired_sessions(&self) -> Result<u64, StorageError> {
        let now = Utc::now();
        let result = sqlx::query("DELETE FROM sessions WHERE expires_at < ?")
            .bind(now)
            .execute(self.pool())
            .await?;
        Ok(result.rows_affected())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::traits::UserStore;

    #[tokio::test]
    async fn session_create_get_expire() {
        let s = SqliteStorage::in_memory().await.unwrap();
        let u = s.create_user("pk", None, false).await.unwrap();
        let sess = s
            .create_session(&u.id, Duration::from_secs(3600), false)
            .await
            .unwrap();
        assert_eq!(sess.user_id, u.id);
        assert!(!sess.is_read_only);
        assert!(sess.expires_at > Utc::now());

        let got = s.get_session(&sess.token).await.unwrap().unwrap();
        assert_eq!(got.token, sess.token);
    }

    #[tokio::test]
    async fn read_only_session_flag_is_stored() {
        let s = SqliteStorage::in_memory().await.unwrap();
        let u = s.create_user("pk-bot", None, true).await.unwrap();
        let sess = s
            .create_session(&u.id, Duration::from_secs(3600), true)
            .await
            .unwrap();
        assert!(sess.is_read_only);
    }

    #[tokio::test]
    async fn delete_expired_sessions_only_removes_expired() {
        let s = SqliteStorage::in_memory().await.unwrap();
        let u = s.create_user("pk", None, false).await.unwrap();

        let live = s
            .create_session(&u.id, Duration::from_secs(3600), false)
            .await
            .unwrap();

        // Insert an already-expired row directly to avoid needing a clock.
        let expired_token = ulid::Ulid::new().to_string();
        let past = Utc::now() - ChronoDuration::hours(1);
        sqlx::query(
            "INSERT INTO sessions (token, user_id, expires_at, is_read_only) VALUES (?, ?, ?, 0)",
        )
        .bind(&expired_token)
        .bind(&u.id)
        .bind(past)
        .execute(s.pool())
        .await
        .unwrap();

        let removed = s.delete_expired_sessions().await.unwrap();
        assert_eq!(removed, 1);
        assert!(s.get_session(&expired_token).await.unwrap().is_none());
        assert!(s.get_session(&live.token).await.unwrap().is_some());
    }
}
