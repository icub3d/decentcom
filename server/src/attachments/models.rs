use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::storage::models::Attachment;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentResponse {
    pub id: String,
    pub message_id: Option<String>,
    pub channel_id: String,
    pub uploader_id: String,
    pub filename: String,
    pub content_hash: String,
    pub size: i64,
    pub mime_type: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub created_at: DateTime<Utc>,
}

impl From<Attachment> for AttachmentResponse {
    fn from(a: Attachment) -> Self {
        Self {
            id: a.id,
            message_id: a.message_id,
            channel_id: a.channel_id,
            uploader_id: a.uploader_id,
            filename: a.filename,
            content_hash: a.content_hash,
            size: a.size,
            mime_type: a.mime_type,
            width: a.width,
            height: a.height,
            created_at: a.created_at,
        }
    }
}
