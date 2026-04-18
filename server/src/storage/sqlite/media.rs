use async_trait::async_trait;
use sqlx::Row;

use super::SqliteStorage;
use crate::storage::traits::MediaStore;
use crate::storage::StorageError;

#[async_trait]
impl MediaStore for SqliteStorage {
    async fn put(
        &self,
        content_hash: &str,
        mime_type: &str,
        size_bytes: u64,
        uploader_id: &str,
        bytes: &[u8],
    ) -> Result<String, StorageError> {
        // Write file to disk if media_path is configured.
        if let Some(dir) = self.media_dir() {
            tokio::fs::create_dir_all(dir).await.map_err(|e| {
                StorageError::Internal(format!("failed to create media directory: {e}"))
            })?;
            let file_path = dir.join(content_hash);
            if !file_path.exists() {
                tokio::fs::write(&file_path, bytes).await.map_err(|e| {
                    StorageError::Internal(format!("failed to write media file: {e}"))
                })?;
            }
        }

        // Check if a record with this hash already exists (dedup).
        let existing: Option<String> =
            sqlx::query_scalar("SELECT id FROM media WHERE content_hash = ?")
                .bind(content_hash)
                .fetch_optional(self.pool())
                .await?;

        if let Some(id) = existing {
            return Ok(id);
        }

        let id = self.new_id();
        sqlx::query(
            "INSERT INTO media (id, content_hash, mime_type, size_bytes, uploader_id) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(content_hash)
        .bind(mime_type)
        .bind(size_bytes as i64)
        .bind(uploader_id)
        .execute(self.pool())
        .await?;

        Ok(id)
    }

    async fn get(&self, id: &str) -> Result<Option<Vec<u8>>, StorageError> {
        let row = sqlx::query("SELECT content_hash FROM media WHERE id = ?")
            .bind(id)
            .fetch_optional(self.pool())
            .await?;

        let Some(row) = row else {
            return Ok(None);
        };

        let hash: String = row.try_get("content_hash")?;
        self.read_media_file(&hash).await.map(Some)
    }

    async fn get_by_content_hash(
        &self,
        content_hash: &str,
    ) -> Result<Option<(String, Vec<u8>)>, StorageError> {
        let row = sqlx::query("SELECT mime_type FROM media WHERE content_hash = ?")
            .bind(content_hash)
            .fetch_optional(self.pool())
            .await?;

        let Some(row) = row else {
            return Ok(None);
        };

        let mime_type: String = row.try_get("mime_type")?;
        let bytes = self.read_media_file(content_hash).await?;
        Ok(Some((mime_type, bytes)))
    }

    async fn delete(&self, id: &str) -> Result<(), StorageError> {
        sqlx::query("DELETE FROM media WHERE id = ?")
            .bind(id)
            .execute(self.pool())
            .await?;
        Ok(())
    }
}

impl SqliteStorage {
    async fn read_media_file(&self, content_hash: &str) -> Result<Vec<u8>, StorageError> {
        let dir = self
            .media_dir()
            .ok_or_else(|| StorageError::Internal("media_path not configured".into()))?;
        let file_path = dir.join(content_hash);
        tokio::fs::read(&file_path).await.map_err(|e| {
            StorageError::Internal(format!("failed to read media file: {e}"))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::traits::UserStore;
    use tempfile::TempDir;

    async fn storage_with_media() -> (SqliteStorage, TempDir) {
        let tmp = TempDir::new().unwrap();
        let db_path = tmp.path().join("test.db");
        let media_path = tmp.path().join("media");
        let s = SqliteStorage::open(&db_path, Some(media_path)).await.unwrap();
        (s, tmp)
    }

    #[tokio::test]
    async fn media_put_get_dedup() {
        let (s, _tmp) = storage_with_media().await;
        let user = s.create_user("pk1", None, false).await.unwrap();

        let id1 = s
            .put("abc123", "image/png", 100, &user.id, b"png data")
            .await
            .unwrap();

        // Dedup: same hash returns same id.
        let id2 = s
            .put("abc123", "image/png", 100, &user.id, b"png data")
            .await
            .unwrap();
        assert_eq!(id1, id2);

        let bytes = s.get(&id1).await.unwrap().unwrap();
        assert_eq!(bytes, b"png data");

        let (mime, bytes) = s.get_by_content_hash("abc123").await.unwrap().unwrap();
        assert_eq!(mime, "image/png");
        assert_eq!(bytes, b"png data");

        assert!(s.get_by_content_hash("nonexistent").await.unwrap().is_none());
    }
}
