use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Serialize;
use shared::gateway::Op;

use crate::channels::models::{
    CategoryResponse, ChannelResponse, CreateCategoryRequest, CreateChannelRequest,
    ListChannelsResponse, UpdateCategoryRequest, UpdateChannelRequest,
};
use crate::channels::validation::validate_channel_name;
use crate::gateway::events::event_json;
use crate::permissions::{UserPermissions, MANAGE_CHANNELS};
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

fn not_found(msg: impl Into<String>) -> (StatusCode, Json<ErrorBody>) {
    (StatusCode::NOT_FOUND, Json(ErrorBody { error: msg.into() }))
}

fn conflict(msg: impl Into<String>) -> (StatusCode, Json<ErrorBody>) {
    (StatusCode::CONFLICT, Json(ErrorBody { error: msg.into() }))
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

// ── Channel handlers ───────────────────────────────────────────────────────

pub(super) async fn list_channels(
    State(state): State<AppState>,
    _auth: UserPermissions,
) -> ApiResult<ListChannelsResponse> {
    let channels = state
        .storage
        .list_channels()
        .await
        .map_err(internal)?
        .into_iter()
        .map(ChannelResponse::from)
        .collect();
    let categories = state
        .storage
        .list_categories()
        .await
        .map_err(internal)?
        .into_iter()
        .map(CategoryResponse::from)
        .collect();
    Ok(Json(ListChannelsResponse {
        channels,
        categories,
    }))
}

pub(super) async fn create_channel(
    State(state): State<AppState>,
    auth: UserPermissions,
    Json(req): Json<CreateChannelRequest>,
) -> ApiResult<ChannelResponse> {
    if !auth.has(MANAGE_CHANNELS) {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorBody {
                error: "missing manage_channels permission".to_string(),
            }),
        ));
    }
    validate_channel_name(&req.name).map_err(|e| bad_request(e.to_string()))?;
    let channel = state
        .storage
        .create_channel(
            &req.name,
            req.category_id.as_deref(),
            req.position.unwrap_or(0),
        )
        .await
        .map_err(storage_err)?;
    let response = ChannelResponse::from(channel);
    if let Some(msg) = event_json(Op::ChannelCreate, response.clone()) {
        state.gateway.broadcast_all(&msg);
    }
    Ok(Json(response))
}

pub(super) async fn get_channel(
    State(state): State<AppState>,
    _auth: UserPermissions,
    Path(channel_id): Path<String>,
) -> ApiResult<ChannelResponse> {
    let channel = state
        .storage
        .get_channel(&channel_id)
        .await
        .map_err(internal)?
        .ok_or_else(|| not_found("channel not found"))?;
    Ok(Json(ChannelResponse::from(channel)))
}

pub(super) async fn update_channel(
    State(state): State<AppState>,
    auth: UserPermissions,
    Path(channel_id): Path<String>,
    Json(req): Json<UpdateChannelRequest>,
) -> ApiResult<ChannelResponse> {
    if !auth.has(MANAGE_CHANNELS) {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorBody {
                error: "missing manage_channels permission".to_string(),
            }),
        ));
    }
    if let Some(ref name) = req.name {
        validate_channel_name(name).map_err(|e| bad_request(e.to_string()))?;
    }
    // Convert Option<Option<String>> -> Option<Option<&str>> without moving.
    let category_id = req.category_id.as_ref().map(|opt| opt.as_deref());
    let channel = state
        .storage
        .update_channel(
            &channel_id,
            req.name.as_deref(),
            req.topic.as_deref(),
            category_id,
            req.position,
        )
        .await
        .map_err(storage_err)?;
    let response = ChannelResponse::from(channel);
    if let Some(msg) = event_json(Op::ChannelUpdate, response.clone()) {
        state.gateway.broadcast_all(&msg);
    }
    Ok(Json(response))
}

pub(super) async fn delete_channel(
    State(state): State<AppState>,
    auth: UserPermissions,
    Path(channel_id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ErrorBody>)> {
    if !auth.has(MANAGE_CHANNELS) {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorBody {
                error: "missing manage_channels permission".to_string(),
            }),
        ));
    }
    state
        .storage
        .delete_channel(&channel_id)
        .await
        .map_err(storage_err)?;
    #[derive(Serialize)]
    struct DeletedId {
        id: String,
    }
    if let Some(msg) = event_json(Op::ChannelDelete, DeletedId { id: channel_id }) {
        state.gateway.broadcast_all(&msg);
    }
    Ok(StatusCode::NO_CONTENT)
}

// ── Category handlers ──────────────────────────────────────────────────────

pub(super) async fn create_category(
    State(state): State<AppState>,
    auth: UserPermissions,
    Json(req): Json<CreateCategoryRequest>,
) -> ApiResult<CategoryResponse> {
    if !auth.has(MANAGE_CHANNELS) {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorBody {
                error: "missing manage_channels permission".to_string(),
            }),
        ));
    }
    let category = state
        .storage
        .create_category(&req.name, req.position.unwrap_or(0))
        .await
        .map_err(storage_err)?;
    let response = CategoryResponse::from(category);
    if let Some(msg) = event_json(Op::CategoryCreate, response.clone()) {
        state.gateway.broadcast_all(&msg);
    }
    Ok(Json(response))
}

pub(super) async fn update_category(
    State(state): State<AppState>,
    auth: UserPermissions,
    Path(category_id): Path<String>,
    Json(req): Json<UpdateCategoryRequest>,
) -> ApiResult<CategoryResponse> {
    if !auth.has(MANAGE_CHANNELS) {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorBody {
                error: "missing manage_channels permission".to_string(),
            }),
        ));
    }
    let category = state
        .storage
        .update_category(&category_id, req.name.as_deref(), req.position)
        .await
        .map_err(storage_err)?;
    let response = CategoryResponse::from(category);
    if let Some(msg) = event_json(Op::CategoryUpdate, response.clone()) {
        state.gateway.broadcast_all(&msg);
    }
    Ok(Json(response))
}

pub(super) async fn delete_category(
    State(state): State<AppState>,
    auth: UserPermissions,
    Path(category_id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ErrorBody>)> {
    if !auth.has(MANAGE_CHANNELS) {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorBody {
                error: "missing manage_channels permission".to_string(),
            }),
        ));
    }
    state
        .storage
        .delete_category(&category_id)
        .await
        .map_err(storage_err)?;
    #[derive(Serialize)]
    struct DeletedId {
        id: String,
    }
    if let Some(msg) = event_json(Op::CategoryDelete, DeletedId { id: category_id }) {
        state.gateway.broadcast_all(&msg);
    }
    Ok(StatusCode::NO_CONTENT)
}
