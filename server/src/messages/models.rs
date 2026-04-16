use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::attachments::models::AttachmentResponse;
use crate::storage::models::ReactionCount;

#[derive(Debug, Deserialize)]
pub struct CreateMessageRequest {
    pub content: String,
    #[serde(default)]
    pub attachment_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateMessageRequest {
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct ListMessagesQuery {
    pub before: Option<String>,
    pub after: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReactionSummary {
    pub emoji: String,
    pub count: i64,
    pub me: bool,
}

impl From<ReactionCount> for ReactionSummary {
    fn from(r: ReactionCount) -> Self {
        Self {
            emoji: r.emoji,
            count: r.count,
            me: r.me,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageResponse {
    pub id: String,
    pub channel_id: String,
    pub author_id: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub edited_at: Option<DateTime<Utc>>,
    pub deleted: bool,
    pub attachments: Vec<AttachmentResponse>,
    pub reactions: Vec<ReactionSummary>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessagePage {
    pub messages: Vec<MessageResponse>,
    pub has_more: bool,
}

impl MessageResponse {
    pub fn from_message(
        m: crate::storage::models::Message,
        attachments: Vec<AttachmentResponse>,
        reactions: Vec<ReactionSummary>,
    ) -> Self {
        Self {
            id: m.id,
            channel_id: m.channel_id,
            author_id: m.author_id,
            content: m.content,
            created_at: m.created_at,
            edited_at: m.edited_at,
            deleted: m.deleted,
            attachments,
            reactions,
        }
    }
}

impl From<crate::storage::models::Message> for MessageResponse {
    fn from(m: crate::storage::models::Message) -> Self {
        Self::from_message(m, Vec::new(), Vec::new())
    }
}
