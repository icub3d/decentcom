#![allow(dead_code)]

use async_trait::async_trait;
use axum::extract::{FromRef, FromRequestParts};
use axum::http::request::Parts;
use axum::response::{IntoResponse, Response};
use axum::{http::StatusCode, Json};
use serde::Serialize;

use crate::auth::middleware::AuthUser;
use crate::storage::StorageError;
use crate::AppState;

pub const SEND_MESSAGES: i64 = 1 << 0;
pub const READ_MESSAGES: i64 = 1 << 1;
pub const MANAGE_MESSAGES: i64 = 1 << 2;
pub const MANAGE_CHANNELS: i64 = 1 << 3;
pub const MANAGE_ROLES: i64 = 1 << 4;
pub const KICK_MEMBERS: i64 = 1 << 5;
pub const BAN_MEMBERS: i64 = 1 << 6;
pub const MANAGE_INVITES: i64 = 1 << 7;
pub const MANAGE_SERVER: i64 = 1 << 8;
pub const ATTACH_FILES: i64 = 1 << 9;
pub const ADD_REACTIONS: i64 = 1 << 10;
pub const MENTION_EVERYONE: i64 = 1 << 11;
pub const VIEW_AUDIT_LOG: i64 = 1 << 12;
pub const ADMINISTRATOR: i64 = 1 << 13;
pub const ALL_PERMISSIONS: i64 = (1 << 14) - 1;

#[derive(Debug, Clone)]
pub struct MemberUser {
    pub user_id: String,
    pub permissions: i64,
    pub highest_role_position: i32,
}

impl MemberUser {
    pub fn has(&self, permission: i64) -> bool {
        has_permission(self.permissions, permission)
    }
}

pub type UserPermissions = MemberUser;

#[derive(Debug)]
pub struct PermissionRejection {
    status: StatusCode,
    message: String,
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    error: String,
}

impl PermissionRejection {
    fn internal(message: String) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message,
        }
    }
}

impl IntoResponse for PermissionRejection {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ErrorBody {
                error: self.message,
            }),
        )
            .into_response()
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for MemberUser
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let auth = AuthUser::from_request_parts(parts, state)
            .await
            .map_err(IntoResponse::into_response)?;
        let app_state = AppState::from_ref(state);

        let is_member = app_state
            .storage
            .is_member(&auth.user_id)
            .await
            .map_err(|e| PermissionRejection::internal(e.to_string()).into_response())?;
        if !is_member {
            return Err(
                (
                    StatusCode::FORBIDDEN,
                    Json(ErrorBody {
                        error: "membership required".to_string(),
                    }),
                )
                    .into_response(),
            );
        }

        let (permissions, highest_role_position) = compute_base_permissions(
            app_state.storage.as_ref(),
            &auth.user_id,
        )
        .await
        .map_err(|e| PermissionRejection::internal(e.to_string()).into_response())?;

        Ok(Self {
            user_id: auth.user_id,
            permissions,
            highest_role_position,
        })
    }
}

pub fn has_permission(bits: i64, permission: i64) -> bool {
    (bits & ADMINISTRATOR) == ADMINISTRATOR || (bits & permission) == permission
}

pub fn apply_channel_override(bits: i64, allow: i64, deny: i64) -> i64 {
    (bits | allow) & !deny
}

pub async fn compute_base_permissions(
    storage: &dyn crate::storage::Storage,
    user_id: &str,
) -> Result<(i64, i32), StorageError> {
    let everyone = storage
        .get_role("everyone")
        .await?
        .ok_or_else(|| StorageError::Internal("missing built-in @everyone role".to_string()))?;

    let roles = storage.list_member_roles(user_id).await?;
    let mut bits = everyone.permissions;
    let mut highest = everyone.position;

    for role in roles {
        bits |= role.permissions;
        highest = highest.max(role.position);
    }

    if has_permission(bits, ADMINISTRATOR) {
        return Ok((ALL_PERMISSIONS, highest));
    }

    Ok((bits, highest))
}

pub async fn effective_permissions(
    storage: &dyn crate::storage::Storage,
    user_id: &str,
    channel_id: Option<&str>,
) -> Result<i64, StorageError> {
    let (mut bits, _) = compute_base_permissions(storage, user_id).await?;
    if has_permission(bits, ADMINISTRATOR) {
        return Ok(ALL_PERMISSIONS);
    }

    let Some(channel_id) = channel_id else {
        return Ok(bits);
    };

    let mut role_ids = vec!["everyone".to_string()];
    role_ids.extend(
        storage
            .list_member_roles(user_id)
            .await?
            .into_iter()
            .map(|r| r.id),
    );

    let overrides = storage.list_channel_permission_overrides(channel_id).await?;
    for override_row in overrides {
        if role_ids.iter().any(|id| id == &override_row.role_id) {
            bits = apply_channel_override(bits, override_row.allow, override_row.deny);
        }
    }

    Ok(bits)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn permission_helpers_work() {
        let base = READ_MESSAGES;
        let bits = apply_channel_override(base, SEND_MESSAGES, READ_MESSAGES);
        assert!(has_permission(bits, SEND_MESSAGES));
        assert!(!has_permission(bits, READ_MESSAGES));
    }
}
