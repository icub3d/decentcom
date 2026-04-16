use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateThreadResponse {
    pub thread_id: String,
    pub parent_message_id: String,
    pub channel_id: String,
    pub created_at: DateTime<Utc>,
    pub reply_count: i64,
    pub follower_count: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ThreadResponse {
    pub id: String,
    pub channel_id: String,
    pub parent_message_id: String,
    pub creator_id: String,
    pub created_at: DateTime<Utc>,
    pub last_reply_at: Option<DateTime<Utc>>,
    pub reply_count: i64,
    pub follower_count: i64,
    pub is_following: bool,
}
