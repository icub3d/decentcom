use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::Row;

use super::SqliteStorage;
use crate::storage::models::Attachment;
use crate::storage::traits::{AttachmentStore, CreateAttachmentParams};
use crate::storage::StorageError;

fn row_to_attachment(row: &sqlx::sqlite::SqliteRow) -> Result<Attachment, sqlx::Error> {
    Ok(Attachment {
        id: row.try_get("id")?,
        message_id: row.try_get("message_id")?,
        channel_id: row.try_get("channel_id")?,
        uploader_id: row.try_get("uploader_id")?,
        filename: row.try_get("filename")?,
        content_hash: row.try_get("content_hash")?,
        size: row.try_get("size")?,
        mime_type: row.try_get("mime_type")?,
        width: row.try_get("width")?,
        height: row.try_get("height")?,
        created_at: row.try_get::<String, _>("created_at")?
            .parse::<DateTime<Utc>>()
            .unwrap_or_default(),
        deleted_at: row.try_get::<Option<String>, _>("deleted_at")?
            .and_then(|s| s.parse::<DateTime<Utc>>().ok()),
    })
}

#[async_trait]
impl AttachmentStore for SqliteStorage {
    async fn create_attachment(
        &self,
        params: CreateAttachmentParams<'_>,
    ) -> Result<Attachment, StorageError> {
        let id = self.new_id();
        sqlx::query(
            "INSERT INTO attachments (id, channel_id, uploader_id, filename, content_hash, size, mime_type, width, height) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(params.channel_id)
        .bind(params.uploader_id)
        .bind(params.filename)
        .bind(params.content_hash)
        .bind(params.size)
        .bind(params.mime_type)
        .bind(params.width)
        .bind(params.height)
        .execute(self.pool())
        .await?;

        self.get_attachment(&id)
            .await?
            .ok_or(StorageError::Internal("attachment not found after insert".into()))
    }

    async fn get_attachment(&self, id: &str) -> Result<Option<Attachment>, StorageError> {
        let row = sqlx::query("SELECT * FROM attachments WHERE id = ? AND deleted_at IS NULL")
            .bind(id)
            .fetch_optional(self.pool())
            .await?;
        match row {
            Some(r) => Ok(Some(row_to_attachment(&r).map_err(StorageError::from)?)),
            None => Ok(None),
        }
    }

    async fn associate_attachments(
        &self,
        attachment_ids: &[String],
        message_id: &str,
        uploader_id: &str,
    ) -> Result<Vec<Attachment>, StorageError> {
        for aid in attachment_ids {
            let result = sqlx::query(
                "UPDATE attachments SET message_id = ? \
                 WHERE id = ? AND uploader_id = ? AND message_id IS NULL AND deleted_at IS NULL",
            )
            .bind(message_id)
            .bind(aid)
            .bind(uploader_id)
            .execute(self.pool())
            .await?;

            if result.rows_affected() == 0 {
                return Err(StorageError::Conflict(format!(
                    "attachment {aid} not found, not owned by you, or already associated"
                )));
            }
        }

        self.list_attachments_for_message(message_id).await
    }

    async fn list_attachments_for_message(
        &self,
        message_id: &str,
    ) -> Result<Vec<Attachment>, StorageError> {
        let rows = sqlx::query(
            "SELECT * FROM attachments WHERE message_id = ? AND deleted_at IS NULL ORDER BY created_at ASC",
        )
        .bind(message_id)
        .fetch_all(self.pool())
        .await?;

        rows.iter()
            .map(|r| row_to_attachment(r).map_err(StorageError::from))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::traits::{ChannelStore, CreateAttachmentParams, MessageStore, UserStore};

    #[tokio::test]
    async fn create_and_get_attachment() {
        let s = SqliteStorage::in_memory().await.unwrap();
        let user = s.create_user("pk1", None, false).await.unwrap();
        let ch = s.create_channel("general", None, 0).await.unwrap();

        let att = s
            .create_attachment(CreateAttachmentParams {
                channel_id: &ch.id,
                uploader_id: &user.id,
                filename: "photo.png",
                content_hash: "abc123",
                size: 1024,
                mime_type: "image/png",
                width: Some(800),
                height: Some(600),
            })
            .await
            .unwrap();

        assert_eq!(att.filename, "photo.png");
        assert_eq!(att.content_hash, "abc123");
        assert_eq!(att.size, 1024);
        assert!(att.message_id.is_none());

        let fetched = s.get_attachment(&att.id).await.unwrap().unwrap();
        assert_eq!(fetched.id, att.id);
    }

    #[tokio::test]
    async fn associate_attachments_with_message() {
        let s = SqliteStorage::in_memory().await.unwrap();
        let user = s.create_user("pk1", None, false).await.unwrap();
        let ch = s.create_channel("general", None, 0).await.unwrap();
        let msg = s.create_message(&ch.id, &user.id, "hello", None).await.unwrap();

        let att1 = s
            .create_attachment(CreateAttachmentParams {
                channel_id: &ch.id,
                uploader_id: &user.id,
                filename: "a.png",
                content_hash: "h1",
                size: 100,
                mime_type: "image/png",
                width: None,
                height: None,
            })
            .await
            .unwrap();
        let att2 = s
            .create_attachment(CreateAttachmentParams {
                channel_id: &ch.id,
                uploader_id: &user.id,
                filename: "b.txt",
                content_hash: "h2",
                size: 200,
                mime_type: "text/plain",
                width: None,
                height: None,
            })
            .await
            .unwrap();

        let associated = s
            .associate_attachments(&[att1.id.clone(), att2.id.clone()], &msg.id, &user.id)
            .await
            .unwrap();
        assert_eq!(associated.len(), 2);
        assert!(associated.iter().all(|a| a.message_id.as_deref() == Some(&msg.id)));
    }

    #[tokio::test]
    async fn cannot_associate_others_attachment() {
        let s = SqliteStorage::in_memory().await.unwrap();
        let user1 = s.create_user("pk1", None, false).await.unwrap();
        let user2 = s.create_user("pk2", None, false).await.unwrap();
        let ch = s.create_channel("general", None, 0).await.unwrap();
        let msg = s.create_message(&ch.id, &user2.id, "hi", None).await.unwrap();

        let att = s
            .create_attachment(CreateAttachmentParams {
                channel_id: &ch.id,
                uploader_id: &user1.id,
                filename: "a.png",
                content_hash: "h1",
                size: 100,
                mime_type: "image/png",
                width: None,
                height: None,
            })
            .await
            .unwrap();

        let err = s
            .associate_attachments(&[att.id], &msg.id, &user2.id)
            .await;
        assert!(err.is_err());
    }

    #[tokio::test]
    async fn cannot_double_associate() {
        let s = SqliteStorage::in_memory().await.unwrap();
        let user = s.create_user("pk1", None, false).await.unwrap();
        let ch = s.create_channel("general", None, 0).await.unwrap();
        let msg1 = s.create_message(&ch.id, &user.id, "m1", None).await.unwrap();
        let msg2 = s.create_message(&ch.id, &user.id, "m2", None).await.unwrap();

        let att = s
            .create_attachment(CreateAttachmentParams {
                channel_id: &ch.id,
                uploader_id: &user.id,
                filename: "a.png",
                content_hash: "h1",
                size: 100,
                mime_type: "image/png",
                width: None,
                height: None,
            })
            .await
            .unwrap();

        s.associate_attachments(std::slice::from_ref(&att.id), &msg1.id, &user.id)
            .await
            .unwrap();

        let err = s
            .associate_attachments(&[att.id], &msg2.id, &user.id)
            .await;
        assert!(err.is_err());
    }
}
