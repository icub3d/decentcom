use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateRoleRequest {
    pub name: String,
    pub color: Option<String>,
    pub permissions: i64,
    pub position: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRoleRequest {
    pub name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_nullable_string")]
    pub color: Option<Option<String>>,
    pub permissions: Option<i64>,
    pub position: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct SetChannelOverrideRequest {
    pub allow: i64,
    pub deny: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleResponse {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
    pub permissions: i64,
    pub position: i32,
    pub is_builtin: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListRolesResponse {
    pub roles: Vec<RoleResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelOverrideResponse {
    pub channel_id: String,
    pub role_id: String,
    pub allow: i64,
    pub deny: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListChannelOverridesResponse {
    pub overrides: Vec<ChannelOverrideResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberRoleEvent {
    pub user_id: String,
    pub role_id: String,
}

impl From<crate::storage::models::Role> for RoleResponse {
    fn from(role: crate::storage::models::Role) -> Self {
        Self {
            id: role.id,
            name: role.name,
            color: role.color,
            permissions: role.permissions,
            position: role.position,
            is_builtin: role.is_builtin,
            created_at: role.created_at,
            updated_at: role.updated_at,
        }
    }
}

impl From<crate::storage::models::ChannelPermissionOverride> for ChannelOverrideResponse {
    fn from(value: crate::storage::models::ChannelPermissionOverride) -> Self {
        Self {
            channel_id: value.channel_id,
            role_id: value.role_id,
            allow: value.allow,
            deny: value.deny,
        }
    }
}

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
