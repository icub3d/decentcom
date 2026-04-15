use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ── Request bodies ─────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateChannelRequest {
    pub name: String,
    pub category: Option<String>,
    pub position: Option<i32>,
}

/// PATCH /channels/:id — all fields optional.
#[derive(Debug, Deserialize)]
pub struct UpdateChannelRequest {
    pub name: Option<String>,
    pub topic: Option<String>,
    /// `null` in JSON explicitly clears the category.
    /// Omitting the field leaves it unchanged.
    /// Represented as `Option<Option<String>>` where:
    ///   - `None` = field absent from request (leave unchanged)
    ///   - `Some(None)` = field present as JSON null (clear category)
    ///   - `Some(Some(name))` = field present as a string (change category)
    #[serde(default, deserialize_with = "deserialize_optional_nullable_string")]
    pub category: Option<Option<String>>,
    pub position: Option<i32>,
}

// ── Response bodies ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelResponse {
    pub id: String,
    pub name: String,
    pub topic: Option<String>,
    pub category: Option<String>,
    pub position: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(rename = "type")]
    pub channel_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListChannelsResponse {
    pub channels: Vec<ChannelResponse>,
}

// ── Conversions ────────────────────────────────────────────────────────────

impl From<crate::storage::models::Channel> for ChannelResponse {
    fn from(c: crate::storage::models::Channel) -> Self {
        ChannelResponse {
            id: c.id,
            name: c.name,
            topic: c.topic,
            category: c.category,
            position: c.position,
            created_at: c.created_at,
            updated_at: c.updated_at,
            channel_type: "text".to_owned(),
        }
    }
}

// ── Serde helper ───────────────────────────────────────────────────────────

/// Deserialize a field that may be absent, null, or a string, returning:
/// - `None` when field is absent (via `#[serde(default)]`)
/// - `Some(None)` when field is JSON null
/// - `Some(Some(s))` when field is a string
fn deserialize_optional_nullable_string<'de, D>(
    de: D,
) -> Result<Option<Option<String>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct Visitor;

    impl<'de> serde::de::Visitor<'de> for Visitor {
        type Value = Option<Option<String>>;

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "a string or null")
        }

        fn visit_none<E: serde::de::Error>(self) -> Result<Self::Value, E> {
            Ok(Some(None))
        }

        fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> {
            Ok(Some(None))
        }

        fn visit_some<D2: serde::Deserializer<'de>>(
            self,
            d: D2,
        ) -> Result<Self::Value, D2::Error> {
            use serde::Deserialize;
            let s = String::deserialize(d)?;
            Ok(Some(Some(s)))
        }

        fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
            Ok(Some(Some(v.to_owned())))
        }
    }

    de.deserialize_option(Visitor)
}
