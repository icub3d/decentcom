use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Serialize;
use shared::gateway::{Op, ThreadEventData, ThreadUpdateData};

use crate::gateway::events::event_json;
use crate::messages::models::{
    CreateMessageRequest, ListMessagesQuery, MessagePage, MessageResponse,
};
use crate::permissions::{
    effective_permissions, has_permission, UserPermissions, READ_MESSAGES, SEND_MESSAGES,
};
use crate::AppState;
use crate::threads::models::{CreateThreadResponse, ThreadResponse};

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

fn bad_request(msg: impl Into<String>) -> (StatusCode, Json<ErrorBody>) {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorBody { error: msg.into() }),
    )
}

async fn ensure_thread_exists(
    state: &AppState,
    thread_id: &str,
) -> Result<crate::storage::models::Thread, (StatusCode, Json<ErrorBody>)> {
    state
        .storage
        .get_thread(thread_id)
        .await
        .map_err(internal)?
        .ok_or_else(|| not_found("thread not found"))
}

/// POST /channels/:channel_id/messages/:message_id/threads
pub(super) async fn create_thread(
    State(state): State<AppState>,
    auth: UserPermissions,
    Path((channel_id, message_id)): Path<(String, String)>,
) -> ApiResult<CreateThreadResponse> {
    if !state.config.features.message_threads {
        return Err(forbidden("message_threads feature is disabled"));
    }

    let perms = effective_permissions(state.storage.as_ref(), &auth.user_id, Some(&channel_id))
        .await
        .map_err(internal)?;
    if !has_permission(perms, READ_MESSAGES) {
        return Err(forbidden("missing read_messages permission"));
    }

    let msg = state
        .storage
        .get_message(&message_id)
        .await
        .map_err(internal)?
        .ok_or_else(|| not_found("message not found"))?;

    if msg.channel_id != channel_id {
        return Err(not_found("message not found"));
    }

    let thread = state
        .storage
        .create_thread(&channel_id, &message_id, &auth.user_id)
        .await
        .map_err(internal)?;

    let followers = state.storage.list_thread_followers(&thread.id).await.map_err(internal)?;

    if let Some(payload) = event_json(
        Op::ThreadCreate,
        ThreadEventData {
            thread_id: thread.id.clone(),
            parent_message_id: thread.parent_message_id.clone(),
            channel_id: thread.channel_id.clone(),
            creator_id: thread.creator_id.clone(),
        },
    ) {
        state.gateway.broadcast_to_channel(&channel_id, &payload);
    }

    Ok(Json(CreateThreadResponse {
        thread_id: thread.id,
        parent_message_id: thread.parent_message_id,
        channel_id: thread.channel_id,
        created_at: thread.created_at,
        reply_count: thread.reply_count,
        follower_count: followers.len() as i64,
    }))
}

/// GET /threads/:thread_id
pub(super) async fn get_thread(
    State(state): State<AppState>,
    auth: UserPermissions,
    Path(thread_id): Path<String>,
) -> ApiResult<ThreadResponse> {
    let thread = ensure_thread_exists(&state, &thread_id).await?;
    
    let perms = effective_permissions(state.storage.as_ref(), &auth.user_id, Some(&thread.channel_id))
        .await
        .map_err(internal)?;
    if !has_permission(perms, READ_MESSAGES) {
        return Err(forbidden("missing read_messages permission"));
    }

    let followers = state.storage.list_thread_followers(&thread_id).await.map_err(internal)?;
    let is_following = followers.contains(&auth.user_id);

    Ok(Json(ThreadResponse {
        id: thread.id,
        channel_id: thread.channel_id,
        parent_message_id: thread.parent_message_id,
        creator_id: thread.creator_id,
        created_at: thread.created_at,
        last_reply_at: thread.last_reply_at,
        reply_count: thread.reply_count,
        follower_count: followers.len() as i64,
        is_following,
    }))
}

/// GET /threads/:thread_id/messages
pub(super) async fn list_thread_messages(
    State(state): State<AppState>,
    auth: UserPermissions,
    Path(thread_id): Path<String>,
    Query(query): Query<ListMessagesQuery>,
) -> ApiResult<MessagePage> {
    let thread = ensure_thread_exists(&state, &thread_id).await?;
    
    let perms = effective_permissions(state.storage.as_ref(), &auth.user_id, Some(&thread.channel_id))
        .await
        .map_err(internal)?;
    if !has_permission(perms, READ_MESSAGES) {
        return Err(forbidden("missing read_messages permission"));
    }

    let limit = query.limit.unwrap_or(50).clamp(1, 100);
    let fetch_limit = limit.saturating_add(1);

    let mut items = state
        .storage
        .list_thread_messages(&thread_id, query.before.as_deref(), query.after.as_deref(), fetch_limit)
        .await
        .map_err(internal)?;

    let has_more = items.len() as u32 > limit;
    if has_more {
        items.truncate(limit as usize);
    }

    let mut messages = Vec::with_capacity(items.len());
    for item in items {
        let enriched = crate::messages::enrich_message(&state, item, &auth.user_id)
            .await
            .map_err(|(s, Json(e))| (s, Json(ErrorBody { error: e.error })))?;
        messages.push(enriched);
    }

    Ok(Json(MessagePage { messages, has_more }))
}

/// POST /threads/:thread_id/messages
pub(super) async fn create_thread_message(
    State(state): State<AppState>,
    auth: UserPermissions,
    Path(thread_id): Path<String>,
    Json(req): Json<CreateMessageRequest>,
) -> ApiResult<MessageResponse> {
    let thread = ensure_thread_exists(&state, &thread_id).await?;
    
    let perms = effective_permissions(state.storage.as_ref(), &auth.user_id, Some(&thread.channel_id))
        .await
        .map_err(internal)?;
    if !has_permission(perms, SEND_MESSAGES) {
        return Err(forbidden("missing send_messages permission"));
    }

    if req.content.trim().is_empty() && req.attachment_ids.is_empty() {
        return Err(bad_request("message content must not be empty"));
    }

    let message = state
        .storage
        .create_message(&thread.channel_id, &auth.user_id, &req.content, Some(&thread_id))
        .await
        .map_err(internal)?;

    // Auto-follow
    state.storage.add_thread_follower(&thread_id, &auth.user_id).await.map_err(internal)?;

    let response = crate::messages::enrich_message(&state, message.clone(), &auth.user_id)
        .await
        .map_err(|(s, Json(e))| (s, Json(ErrorBody { error: e.error })))?;

    #[derive(Serialize)]
    struct ThreadMessageEvent {
        thread_id: String,
        message: MessageResponse,
    }

    if let Some(payload) = event_json(
        Op::ThreadMessageCreate,
        ThreadMessageEvent {
            thread_id: thread_id.clone(),
            message: response.clone(),
        },
    ) {
        state.gateway.broadcast_to_channel(&thread.channel_id, &payload);
    }

    // Also broadcast ThreadUpdate for reply count
    if let Some(payload) = event_json(
        Op::ThreadUpdate,
        ThreadUpdateData {
            thread_id: thread_id.clone(),
            reply_count: thread.reply_count + 1,
            last_reply_at: Some(message.created_at),
        },
    ) {
        state.gateway.broadcast_to_channel(&thread.channel_id, &payload);
    }

    Ok(Json(response))
}

/// PUT /threads/:thread_id/follow
pub(super) async fn follow_thread(
    State(state): State<AppState>,
    auth: UserPermissions,
    Path(thread_id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ErrorBody>)> {
    let thread = ensure_thread_exists(&state, &thread_id).await?;
    
    let perms = effective_permissions(state.storage.as_ref(), &auth.user_id, Some(&thread.channel_id))
        .await
        .map_err(internal)?;
    if !has_permission(perms, READ_MESSAGES) {
        return Err(forbidden("missing read_messages permission"));
    }

    state.storage.add_thread_follower(&thread_id, &auth.user_id).await.map_err(internal)?;
    Ok(StatusCode::OK)
}

/// DELETE /threads/:thread_id/follow
pub(super) async fn unfollow_thread(
    State(state): State<AppState>,
    auth: UserPermissions,
    Path(thread_id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ErrorBody>)> {
    let _thread = ensure_thread_exists(&state, &thread_id).await?;
    state.storage.remove_thread_follower(&thread_id, &auth.user_id).await.map_err(internal)?;
    Ok(StatusCode::NO_CONTENT)
}

/// PUT /threads/:thread_id/read
pub(super) async fn mark_thread_read(
    State(state): State<AppState>,
    auth: UserPermissions,
    Path(thread_id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ErrorBody>)> {
    let _thread = ensure_thread_exists(&state, &thread_id).await?;
    state.storage.update_thread_last_read(&thread_id, &auth.user_id).await.map_err(internal)?;
    Ok(StatusCode::OK)
}
