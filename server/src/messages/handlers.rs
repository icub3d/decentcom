use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Serialize;
use shared::gateway::Op;

use crate::auth::middleware::AuthUser;
use crate::gateway::events::event_json;
use crate::messages::models::{
    CreateMessageRequest, ListMessagesQuery, MessagePage, MessageResponse, UpdateMessageRequest,
};
use crate::AppState;

#[derive(Debug, Serialize)]
pub(super) struct ErrorBody {
    error: String,
}

type ApiResult<T> = Result<Json<T>, (StatusCode, Json<ErrorBody>)>;

fn bad_request(msg: impl Into<String>) -> (StatusCode, Json<ErrorBody>) {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorBody { error: msg.into() }),
    )
}

fn unprocessable(msg: impl Into<String>) -> (StatusCode, Json<ErrorBody>) {
    (
        StatusCode::UNPROCESSABLE_ENTITY,
        Json(ErrorBody { error: msg.into() }),
    )
}

fn not_found(msg: impl Into<String>) -> (StatusCode, Json<ErrorBody>) {
    (
        StatusCode::NOT_FOUND,
        Json(ErrorBody { error: msg.into() }),
    )
}

fn forbidden(msg: impl Into<String>) -> (StatusCode, Json<ErrorBody>) {
    (
        StatusCode::FORBIDDEN,
        Json(ErrorBody { error: msg.into() }),
    )
}

fn conflict(msg: impl Into<String>) -> (StatusCode, Json<ErrorBody>) {
    (
        StatusCode::CONFLICT,
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
        crate::storage::StorageError::Conflict(msg) => conflict(msg),
        _ => internal(e),
    }
}

fn validate_message_content(content: &str, max_len: usize) -> Result<(), (StatusCode, Json<ErrorBody>)> {
    if content.trim().is_empty() {
        return Err(unprocessable("message content must not be empty"));
    }
    if content.chars().count() > max_len {
        return Err(unprocessable(format!(
            "message content must be at most {max_len} characters"
        )));
    }
    Ok(())
}

async fn ensure_channel_exists(
    state: &AppState,
    channel_id: &str,
) -> Result<(), (StatusCode, Json<ErrorBody>)> {
    let exists = state
        .storage
        .get_channel(channel_id)
        .await
        .map_err(internal)?
        .is_some();
    if !exists {
        return Err(not_found("channel not found"));
    }
    Ok(())
}

pub(super) async fn create_message(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(channel_id): Path<String>,
    Json(req): Json<CreateMessageRequest>,
) -> ApiResult<MessageResponse> {
    ensure_channel_exists(&state, &channel_id).await?;
    validate_message_content(&req.content, state.config.content.max_message_length)?;

    let message = state
        .storage
        .create_message(&channel_id, &auth.user_id, &req.content)
        .await
        .map_err(storage_err)?;
    let response = MessageResponse::from(message);

    if let Some(msg) = event_json(Op::MessageCreate, response.clone()) {
        state.gateway.broadcast_to_channel(&channel_id, &msg);
    }

    Ok(Json(response))
}

pub(super) async fn list_messages(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(channel_id): Path<String>,
    Query(query): Query<ListMessagesQuery>,
) -> ApiResult<MessagePage> {
    ensure_channel_exists(&state, &channel_id).await?;

    if query.after.is_some() {
        return Err(bad_request("after cursor is not implemented yet; use before"));
    }

    let limit = query.limit.unwrap_or(50).clamp(1, 100);
    let fetch_limit = limit.saturating_add(1);

    let mut items = state
        .storage
        .list_messages(&channel_id, query.before.as_deref(), fetch_limit)
        .await
        .map_err(internal)?;

    let has_more = items.len() as u32 > limit;
    if has_more {
        items.truncate(limit as usize);
    }

    let messages = items.into_iter().map(MessageResponse::from).collect();
    Ok(Json(MessagePage { messages, has_more }))
}

pub(super) async fn get_message(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path((channel_id, message_id)): Path<(String, String)>,
) -> ApiResult<MessageResponse> {
    ensure_channel_exists(&state, &channel_id).await?;

    let message = state
        .storage
        .get_message(&message_id)
        .await
        .map_err(internal)?
        .ok_or_else(|| not_found("message not found"))?;

    if message.channel_id != channel_id {
        return Err(not_found("message not found"));
    }

    Ok(Json(MessageResponse::from(message)))
}

pub(super) async fn update_message(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((channel_id, message_id)): Path<(String, String)>,
    Json(req): Json<UpdateMessageRequest>,
) -> ApiResult<MessageResponse> {
    ensure_channel_exists(&state, &channel_id).await?;
    validate_message_content(&req.content, state.config.content.max_message_length)?;

    let existing = state
        .storage
        .get_message(&message_id)
        .await
        .map_err(internal)?
        .ok_or_else(|| not_found("message not found"))?;

    if existing.channel_id != channel_id {
        return Err(not_found("message not found"));
    }
    if existing.author_id != auth.user_id {
        return Err(forbidden("cannot edit another user's message"));
    }
    if existing.deleted {
        return Err(conflict("cannot edit a deleted message"));
    }

    let updated = state
        .storage
        .update_message(&message_id, &req.content)
        .await
        .map_err(storage_err)?;
    let response = MessageResponse::from(updated);

    if let Some(msg) = event_json(Op::MessageUpdate, response.clone()) {
        state.gateway.broadcast_to_channel(&channel_id, &msg);
    }

    Ok(Json(response))
}

pub(super) async fn delete_message(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((channel_id, message_id)): Path<(String, String)>,
) -> Result<StatusCode, (StatusCode, Json<ErrorBody>)> {
    ensure_channel_exists(&state, &channel_id).await?;

    let existing = state
        .storage
        .get_message(&message_id)
        .await
        .map_err(internal)?
        .ok_or_else(|| not_found("message not found"))?;

    if existing.channel_id != channel_id {
        return Err(not_found("message not found"));
    }
    if existing.author_id != auth.user_id {
        return Err(forbidden("cannot delete another user's message"));
    }

    state
        .storage
        .delete_message(&message_id)
        .await
        .map_err(storage_err)?;

    #[derive(Serialize)]
    struct MessageDeleteEvent {
        id: String,
        channel_id: String,
    }

    if let Some(msg) = event_json(
        Op::MessageDelete,
        MessageDeleteEvent {
            id: message_id,
            channel_id: channel_id.clone(),
        },
    ) {
        state.gateway.broadcast_to_channel(&channel_id, &msg);
    }

    Ok(StatusCode::NO_CONTENT)
}
