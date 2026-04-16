use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use shared::gateway::{Op, ReactionEventData};

use crate::gateway::events::event_json;
use crate::permissions::{
    effective_permissions, has_permission, UserPermissions, ADD_REACTIONS, MANAGE_MESSAGES,
    READ_MESSAGES,
};
use crate::AppState;

#[derive(Debug, Serialize)]
pub(super) struct ErrorBody {
    error: String,
}

type ApiResult<T> = Result<Json<T>, (StatusCode, Json<ErrorBody>)>;

fn forbidden(msg: impl Into<String>) -> (StatusCode, Json<ErrorBody>) {
    (
        StatusCode::FORBIDDEN,
        Json(ErrorBody { error: msg.into() }),
    )
}

fn not_found(msg: impl Into<String>) -> (StatusCode, Json<ErrorBody>) {
    (
        StatusCode::NOT_FOUND,
        Json(ErrorBody { error: msg.into() }),
    )
}

fn internal(e: crate::storage::StorageError) -> (StatusCode, Json<ErrorBody>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorBody {
            error: e.to_string(),
        }),
    )
}

async fn resolve_message(
    state: &AppState,
    channel_id: &str,
    message_id: &str,
) -> Result<crate::storage::models::Message, (StatusCode, Json<ErrorBody>)> {
    let msg = state
        .storage
        .get_message(message_id)
        .await
        .map_err(internal)?
        .ok_or_else(|| not_found("message not found"))?;
    if msg.channel_id != channel_id {
        return Err(not_found("message not found"));
    }
    Ok(msg)
}

/// PUT /channels/:channel_id/messages/:message_id/reactions/:emoji
pub(super) async fn put_reaction(
    State(state): State<AppState>,
    auth: UserPermissions,
    Path((channel_id, message_id, emoji)): Path<(String, String, String)>,
) -> Result<StatusCode, (StatusCode, Json<ErrorBody>)> {
    if !state.config.features.emoji_reactions {
        return Err(forbidden("emoji_reactions feature is disabled"));
    }

    let perms = effective_permissions(state.storage.as_ref(), &auth.user_id, Some(&channel_id))
        .await
        .map_err(internal)?;
    if !has_permission(perms, READ_MESSAGES) {
        return Err(forbidden("missing read_messages permission"));
    }
    if !has_permission(perms, ADD_REACTIONS) {
        return Err(forbidden("missing add_reactions permission"));
    }

    let _msg = resolve_message(&state, &channel_id, &message_id).await?;

    let result = state
        .storage
        .upsert_reaction(&message_id, &auth.user_id, &emoji)
        .await
        .map_err(internal)?;

    let op = if result.is_some() {
        Op::ReactionAdd
    } else {
        Op::ReactionRemove
    };

    if let Some(payload) = event_json(
        op,
        ReactionEventData {
            channel_id: channel_id.clone(),
            message_id: message_id.clone(),
            user_id: auth.user_id.clone(),
            emoji: emoji.clone(),
        },
    ) {
        state.gateway.broadcast_to_channel(&channel_id, &payload);
    }

    Ok(StatusCode::OK)
}

/// DELETE /channels/:channel_id/messages/:message_id/reactions
pub(super) async fn delete_own_reaction(
    State(state): State<AppState>,
    auth: UserPermissions,
    Path((channel_id, message_id)): Path<(String, String)>,
) -> Result<StatusCode, (StatusCode, Json<ErrorBody>)> {
    if !state.config.features.emoji_reactions {
        return Err(forbidden("emoji_reactions feature is disabled"));
    }

    let _msg = resolve_message(&state, &channel_id, &message_id).await?;

    let existing = state
        .storage
        .get_user_reaction(&message_id, &auth.user_id)
        .await
        .map_err(internal)?;

    state
        .storage
        .remove_reaction(&message_id, &auth.user_id)
        .await
        .map_err(internal)?;

    if let Some(r) = existing {
        let emoji = r.emoji;
        if let Some(payload) = event_json(
            Op::ReactionRemove,
            ReactionEventData {
                channel_id: channel_id.clone(),
                message_id: message_id.clone(),
                user_id: auth.user_id.clone(),
                emoji,
            },
        ) {
            state.gateway.broadcast_to_channel(&channel_id, &payload);
        }
    }

    Ok(StatusCode::NO_CONTENT)
}

/// DELETE /channels/:channel_id/messages/:message_id/reactions/:user_id
pub(super) async fn delete_user_reaction(
    State(state): State<AppState>,
    auth: UserPermissions,
    Path((channel_id, message_id, target_user_id)): Path<(String, String, String)>,
) -> Result<StatusCode, (StatusCode, Json<ErrorBody>)> {
    if !state.config.features.emoji_reactions {
        return Err(forbidden("emoji_reactions feature is disabled"));
    }

    let perms = effective_permissions(state.storage.as_ref(), &auth.user_id, Some(&channel_id))
        .await
        .map_err(internal)?;
    if !has_permission(perms, MANAGE_MESSAGES) {
        return Err(forbidden("missing manage_messages permission"));
    }

    let _msg = resolve_message(&state, &channel_id, &message_id).await?;

    let existing = state
        .storage
        .get_user_reaction(&message_id, &target_user_id)
        .await
        .map_err(internal)?;

    state
        .storage
        .remove_reaction(&message_id, &target_user_id)
        .await
        .map_err(internal)?;

    if let Some(r) = existing {
        let emoji = r.emoji;
        if let Some(payload) = event_json(
            Op::ReactionRemove,
            ReactionEventData {
                channel_id: channel_id.clone(),
                message_id: message_id.clone(),
                user_id: target_user_id.clone(),
                emoji,
            },
        ) {
            state.gateway.broadcast_to_channel(&channel_id, &payload);
        }
    }

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize)]
pub(super) struct ListReactorsQuery {
    pub limit: Option<u32>,
    pub before: Option<String>,
}

#[derive(Debug, Serialize)]
pub(super) struct ReactorEntry {
    pub id: String,
    pub display_name: Option<String>,
}

#[derive(Debug, Serialize)]
pub(super) struct ListReactorsResponse {
    pub users: Vec<ReactorEntry>,
    pub total: usize,
}

/// GET /channels/:channel_id/messages/:message_id/reactions/:emoji
pub(super) async fn list_reactors(
    State(state): State<AppState>,
    auth: UserPermissions,
    Path((channel_id, message_id, emoji)): Path<(String, String, String)>,
    Query(query): Query<ListReactorsQuery>,
) -> ApiResult<ListReactorsResponse> {
    if !state.config.features.emoji_reactions {
        return Err(forbidden("emoji_reactions feature is disabled"));
    }

    let perms = effective_permissions(state.storage.as_ref(), &auth.user_id, Some(&channel_id))
        .await
        .map_err(internal)?;
    if !has_permission(perms, READ_MESSAGES) {
        return Err(forbidden("missing read_messages permission"));
    }

    let _msg = resolve_message(&state, &channel_id, &message_id).await?;

    let limit = query.limit.unwrap_or(25).clamp(1, 100);
    let reactions = state
        .storage
        .list_emoji_reactors(&message_id, &emoji, limit, query.before.as_deref())
        .await
        .map_err(internal)?;

    let total = reactions.len();
    let mut users = Vec::with_capacity(total);
    for r in &reactions {
        if let Ok(Some(user)) = state.storage.get_user_by_id(&r.user_id).await {
            users.push(ReactorEntry {
                id: user.id,
                display_name: user.display_name,
            });
        }
    }

    Ok(Json(ListReactorsResponse { users, total }))
}
