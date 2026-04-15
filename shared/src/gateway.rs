use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Op {
    Hello,
    Ready,
    Heartbeat,
    MessageCreate,
    MessageUpdate,
    MessageDelete,
    ChannelCreate,
    ChannelUpdate,
    ChannelDelete,
    CategoryCreate,
    CategoryUpdate,
    CategoryDelete,
    RoleCreate,
    RoleUpdate,
    RoleDelete,
    MemberRoleAdd,
    MemberRoleRemove,
    MemberJoin,
    MemberLeave,
    MemberKick,
    MemberBan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventEnvelope<T> {
    pub op: Op,
    pub d: T,
    pub t: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HelloUser {
    pub id: String,
    pub pubkey: String,
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HelloChannel {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HelloData {
    pub server_name: String,
    pub user: HelloUser,
    pub channels: Vec<HelloChannel>,
    pub heartbeat_interval_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmptyData {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubscriptionData {
    pub channel_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum ClientCommand {
    Subscribe(SubscriptionData),
    Unsubscribe(SubscriptionData),
    HeartbeatAck(EmptyData),
    Unknown,
}

impl<'de> Deserialize<'de> for ClientCommand {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        let op = value.get("op").and_then(Value::as_str).unwrap_or_default();

        match op {
            "SUBSCRIBE" => {
                let data = value
                    .get("d")
                    .cloned()
                    .ok_or_else(|| serde::de::Error::custom("missing d field"))?;
                let payload = serde_json::from_value::<SubscriptionData>(data)
                    .map_err(serde::de::Error::custom)?;
                Ok(ClientCommand::Subscribe(payload))
            }
            "UNSUBSCRIBE" => {
                let data = value
                    .get("d")
                    .cloned()
                    .ok_or_else(|| serde::de::Error::custom("missing d field"))?;
                let payload = serde_json::from_value::<SubscriptionData>(data)
                    .map_err(serde::de::Error::custom)?;
                Ok(ClientCommand::Unsubscribe(payload))
            }
            "HEARTBEAT_ACK" => {
                let data = value
                    .get("d")
                    .cloned()
                    .unwrap_or_else(|| Value::Object(Default::default()));
                let payload =
                    serde_json::from_value::<EmptyData>(data).map_err(serde::de::Error::custom)?;
                Ok(ClientCommand::HeartbeatAck(payload))
            }
            _ => Ok(ClientCommand::Unknown),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_envelope_serializes_expected_shape() {
        let envelope = EventEnvelope {
            op: Op::Heartbeat,
            d: EmptyData {},
            t: 1_712_345_678,
        };

        let json = serde_json::to_value(envelope).unwrap();
        assert_eq!(json["op"], "HEARTBEAT");
        assert_eq!(json["d"], serde_json::json!({}));
        assert_eq!(json["t"], 1_712_345_678);
    }

    #[test]
    fn client_command_deserializes_known_and_unknown_ops() {
        let subscribe: ClientCommand =
            serde_json::from_str(r#"{"op":"SUBSCRIBE","d":{"channel_id":"c1"}}"#).unwrap();
        assert_eq!(
            subscribe,
            ClientCommand::Subscribe(SubscriptionData {
                channel_id: "c1".to_string()
            })
        );

        let unknown: ClientCommand =
            serde_json::from_str(r#"{"op":"SOMETHING_NEW","d":{}}"#).unwrap();
        assert_eq!(unknown, ClientCommand::Unknown);
    }
}
