use chrono::{DateTime, Utc};
use serde::Deserialize;
use shared::gateway::{
    Op, ReactionEventData, ThreadEventData, ThreadUpdateData,
};

use decentcom_sdk::gateway::GatewayEvent;
use crate::error::Result;

// ── Per-op payload types ───────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct MessageEvent {
    pub id: String,
    pub channel_id: String,
    pub author_id: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub edited_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MessageDeleteEvent {
    pub id: String,
    pub channel_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChannelEvent {
    pub id: String,
    pub name: String,
    pub topic: Option<String>,
    pub category: Option<String>,
    pub position: i32,
    #[serde(rename = "type")]
    pub channel_type: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChannelDeleteEvent {
    pub id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RoleEvent {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
    pub permissions: i64,
    pub position: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RoleDeleteEvent {
    pub id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MemberRoleEvent {
    pub user_id: String,
    pub role_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MemberEvent {
    pub user_id: String,
    pub pubkey: String,
    pub display_name: Option<String>,
    pub is_bot: bool,
    pub joined_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MemberModEvent {
    pub user_id: String,
    pub pubkey: String,
    pub reason: Option<String>,
}

/// A fully-decoded, typed gateway event.
#[derive(Debug, Clone)]
pub enum Event {
    Message(MessageEvent),
    MessageUpdate(MessageEvent),
    MessageDelete(MessageDeleteEvent),
    ChannelCreate(ChannelEvent),
    ChannelUpdate(ChannelEvent),
    ChannelDelete(ChannelDeleteEvent),
    RoleCreate(RoleEvent),
    RoleUpdate(RoleEvent),
    RoleDelete(RoleDeleteEvent),
    MemberRoleAdd(MemberRoleEvent),
    MemberRoleRemove(MemberRoleEvent),
    MemberJoin(MemberEvent),
    MemberLeave(MemberEvent),
    MemberKick(MemberModEvent),
    MemberBan(MemberModEvent),
    MemberUpdate(MemberEvent),
    ReactionAdd(ReactionEventData),
    ReactionRemove(ReactionEventData),
    ThreadCreate(ThreadEventData),
    ThreadMessageCreate(ThreadEventData),
    ThreadUpdate(ThreadUpdateData),
}

impl Event {
    /// Decode a raw [`GatewayEvent`] from the SDK into a typed [`Event`].
    pub fn from_gateway(raw: &GatewayEvent) -> Result<Option<Event>> {
        let ev = match raw.op {
            Op::MessageCreate => Event::Message(raw.decode()?),
            Op::MessageUpdate => Event::MessageUpdate(raw.decode()?),
            Op::MessageDelete => Event::MessageDelete(raw.decode()?),
            Op::ChannelCreate => Event::ChannelCreate(raw.decode()?),
            Op::ChannelUpdate => Event::ChannelUpdate(raw.decode()?),
            Op::ChannelDelete => Event::ChannelDelete(raw.decode()?),
            Op::RoleCreate => Event::RoleCreate(raw.decode()?),
            Op::RoleUpdate => Event::RoleUpdate(raw.decode()?),
            Op::RoleDelete => Event::RoleDelete(raw.decode()?),
            Op::MemberRoleAdd => Event::MemberRoleAdd(raw.decode()?),
            Op::MemberRoleRemove => Event::MemberRoleRemove(raw.decode()?),
            Op::MemberJoin => Event::MemberJoin(raw.decode()?),
            Op::MemberLeave => Event::MemberLeave(raw.decode()?),
            Op::MemberKick => Event::MemberKick(raw.decode()?),
            Op::MemberBan => Event::MemberBan(raw.decode()?),
            Op::MemberUpdate => Event::MemberUpdate(raw.decode()?),
            Op::ReactionAdd => Event::ReactionAdd(raw.decode()?),
            Op::ReactionRemove => Event::ReactionRemove(raw.decode()?),
            Op::ThreadCreate => Event::ThreadCreate(raw.decode()?),
            Op::ThreadMessageCreate => Event::ThreadMessageCreate(raw.decode()?),
            Op::ThreadUpdate => Event::ThreadUpdate(raw.decode()?),
            // These are lifecycle ops handled by the run loop, not dispatched as events.
            Op::Hello | Op::Ready | Op::Heartbeat => return Ok(None),
        };
        Ok(Some(ev))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_raw(op: Op, data: serde_json::Value) -> GatewayEvent {
        GatewayEvent { op, data, t: 0 }
    }

    #[test]
    fn decode_message_create() {
        let data = json!({
            "id": "m1",
            "channel_id": "c1",
            "author_id": "u1",
            "content": "hello",
            "created_at": "2024-01-01T00:00:00Z",
            "edited_at": null,
        });
        let raw = make_raw(Op::MessageCreate, data);
        let ev = Event::from_gateway(&raw).unwrap().unwrap();
        if let Event::Message(msg) = ev {
            assert_eq!(msg.content, "hello");
            assert_eq!(msg.channel_id, "c1");
        } else {
            panic!("expected Event::Message");
        }
    }

    #[test]
    fn decode_member_join() {
        let data = json!({
            "user_id": "u2",
            "pubkey": "pk",
            "display_name": null,
            "is_bot": false,
            "joined_at": "2024-01-01T00:00:00Z",
        });
        let raw = make_raw(Op::MemberJoin, data);
        let ev = Event::from_gateway(&raw).unwrap().unwrap();
        assert!(matches!(ev, Event::MemberJoin(_)));
    }

    #[test]
    fn decode_member_leave() {
        let data = json!({
            "user_id": "u3",
            "pubkey": "pk",
            "display_name": null,
            "is_bot": false,
            "joined_at": "2024-01-01T00:00:00Z",
        });
        let raw = make_raw(Op::MemberLeave, data);
        let ev = Event::from_gateway(&raw).unwrap().unwrap();
        assert!(matches!(ev, Event::MemberLeave(_)));
    }

    #[test]
    fn hello_returns_none() {
        let raw = make_raw(Op::Hello, json!({}));
        let ev = Event::from_gateway(&raw).unwrap();
        assert!(ev.is_none());
    }

    #[test]
    fn heartbeat_returns_none() {
        let raw = make_raw(Op::Heartbeat, json!({}));
        let ev = Event::from_gateway(&raw).unwrap();
        assert!(ev.is_none());
    }

    #[test]
    fn decode_reaction_add() {
        let data = json!({
            "channel_id": "c1",
            "message_id": "m1",
            "user_id": "u1",
            "emoji": "👍",
        });
        let raw = make_raw(Op::ReactionAdd, data);
        let ev = Event::from_gateway(&raw).unwrap().unwrap();
        if let Event::ReactionAdd(r) = ev {
            assert_eq!(r.emoji, "👍");
        } else {
            panic!("expected Event::ReactionAdd");
        }
    }

    #[test]
    fn decode_member_kick() {
        let data = json!({
            "user_id": "u1",
            "pubkey": "pk",
            "reason": "spam",
        });
        let raw = make_raw(Op::MemberKick, data);
        let ev = Event::from_gateway(&raw).unwrap().unwrap();
        if let Event::MemberKick(m) = ev {
            assert_eq!(m.reason.as_deref(), Some("spam"));
        } else {
            panic!("expected Event::MemberKick");
        }
    }
}
