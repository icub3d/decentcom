use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Serialize;
use shared::gateway::Op;

use crate::gateway::events::event_json;
use crate::permissions::{
    effective_permissions, has_permission, UserPermissions, ADD_REACTIONS, MANAGE_MESSAGES,
    READ_MESSAGES,
};
use crate::AppState;

#[derive(Debug, Serialize)]
pub struct ErrorBody {
    error: String,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: String,
    pub display_name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ReactionListResponse {
    pub users: Vec<UserResponse>,
    pub total: i64,
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

fn storage_err(e: crate::storage::StorageError) -> (StatusCode, Json<ErrorBody>) {
    match e {
        crate::storage::StorageError::NotFound => not_found("not found"),
        _ => internal(e),
    }
}

async fn check_emoji_reactions_enabled(
    state: &AppState,
) -> Result<(), (StatusCode, Json<ErrorBody>)> {
    if !state.config.features.emoji_reactions {
        return Err(forbidden("emoji_reactions feature is disabled"));
    }
    Ok(())
}

pub async fn add_reaction(
    State(state): State<AppState>,
    auth: UserPermissions,
    Path((channel_id, message_id, emoji)): Path<(String, String, String)>,
) -> Result<StatusCode, (StatusCode, Json<ErrorBody>)> {
    check_emoji_reactions_enabled(&state).await?;

    let perms = effective_permissions(state.storage.as_ref(), &auth.user_id, Some(&channel_id))
        .await
        .map_err(internal)?;

    if !has_permission(perms, READ_MESSAGES) {
        return Err(forbidden("missing read_messages permission"));
    }

    if !has_permission(perms, ADD_REACTIONS) {
        return Err(forbidden("missing add_reactions permission"));
    }

    let message = state
        .storage
        .get_message(&message_id)
        .await
        .map_err(internal)?
        .ok_or_else(|| not_found("message not found"))?;

    if message.channel_id != channel_id {
        return Err(not_found("message not found"));
    }

    let _reaction = state
        .storage
        .add_reaction(&message_id, &auth.user_id, &emoji)
        .await
        .map_err(storage_err)?;

    let data = shared::gateway::ReactionAddData {
        channel_id: channel_id.clone(),
        message_id: message_id.clone(),
        user_id: auth.user_id.clone(),
        emoji: emoji.clone(),
    };

    if let Some(msg) = event_json(Op::ReactionAdd, data) {
        state.gateway.broadcast_to_channel(&channel_id, &msg);
    }

    Ok(StatusCode::OK)
}

pub async fn remove_own_reaction(
    State(state): State<AppState>,
    auth: UserPermissions,
    Path((channel_id, message_id, emoji)): Path<(String, String, String)>,
) -> Result<StatusCode, (StatusCode, Json<ErrorBody>)> {
    check_emoji_reactions_enabled(&state).await?;

    let perms = effective_permissions(state.storage.as_ref(), &auth.user_id, Some(&channel_id))
        .await
        .map_err(internal)?;

    if !has_permission(perms, READ_MESSAGES) {
        return Err(forbidden("missing read_messages permission"));
    }

    let message = state
        .storage
        .get_message(&message_id)
        .await
        .map_err(internal)?
        .ok_or_else(|| not_found("message not found"))?;

    if message.channel_id != channel_id {
        return Err(not_found("message not found"));
    }

    state
        .storage
        .remove_reaction(&message_id, &auth.user_id, &emoji)
        .await
        .map_err(storage_err)?;

    let data = shared::gateway::ReactionRemoveData {
        channel_id: channel_id.clone(),
        message_id: message_id.clone(),
        user_id: auth.user_id.clone(),
        emoji: emoji.clone(),
    };

    if let Some(msg) = event_json(Op::ReactionRemove, data) {
        state.gateway.broadcast_to_channel(&channel_id, &msg);
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn remove_user_reaction(
    State(state): State<AppState>,
    auth: UserPermissions,
    Path((channel_id, message_id, emoji, user_id)): Path<(String, String, String, String)>,
) -> Result<StatusCode, (StatusCode, Json<ErrorBody>)> {
    check_emoji_reactions_enabled(&state).await?;

    let perms = effective_permissions(state.storage.as_ref(), &auth.user_id, Some(&channel_id))
        .await
        .map_err(internal)?;

    if !has_permission(perms, MANAGE_MESSAGES) {
        return Err(forbidden("missing manage_messages permission"));
    }

    let message = state
        .storage
        .get_message(&message_id)
        .await
        .map_err(internal)?
        .ok_or_else(|| not_found("message not found"))?;

    if message.channel_id != channel_id {
        return Err(not_found("message not found"));
    }

    state
        .storage
        .remove_user_reaction(&message_id, &user_id, &emoji)
        .await
        .map_err(storage_err)?;

    let data = shared::gateway::ReactionRemoveData {
        channel_id: channel_id.clone(),
        message_id: message_id.clone(),
        user_id: user_id.clone(),
        emoji: emoji.clone(),
    };

    if let Some(msg) = event_json(Op::ReactionRemove, data) {
        state.gateway.broadcast_to_channel(&channel_id, &msg);
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_reaction_users(
    State(state): State<AppState>,
    auth: UserPermissions,
    Path((channel_id, message_id, emoji)): Path<(String, String, String)>,
) -> ApiResult<ReactionListResponse> {
    check_emoji_reactions_enabled(&state).await?;

    let perms = effective_permissions(state.storage.as_ref(), &auth.user_id, Some(&channel_id))
        .await
        .map_err(internal)?;

    if !has_permission(perms, READ_MESSAGES) {
        return Err(forbidden("missing read_messages permission"));
    }

    let message = state
        .storage
        .get_message(&message_id)
        .await
        .map_err(internal)?
        .ok_or_else(|| not_found("message not found"))?;

    if message.channel_id != channel_id {
        return Err(not_found("message not found"));
    }

    let users = state
        .storage
        .list_users_for_reaction(&message_id, &emoji)
        .await
        .map_err(storage_err)?;

    let total = users.len() as i64;
    let responses = users
        .into_iter()
        .map(|u| UserResponse {
            id: u.id,
            display_name: u.display_name,
        })
        .collect();

    Ok(Json(ReactionListResponse {
        users: responses,
        total,
    }))
}
