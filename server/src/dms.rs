use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use shared::dms::{DeviceKey, PendingDM};

use crate::{auth::middleware::AuthUser, storage::StorageError, AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/dms/sync", get(sync_dms))
        .route("/dms/send", post(send_dm))
        .route("/dms/ack", post(ack_dm))
        .route("/users/:pubkey/devices", get(get_device_keys))
        // also allow self-publishing device keys
        .route("/users/@me/devices", post(add_device_key))
}

#[derive(Debug, Deserialize)]
pub struct SendDmRequest {
    pub dm: PendingDM,
    pub target_device_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct AckDmRequest {
    pub message_id: String,
    pub device_id: String, // Ideally, we should ensure the device_id belongs to AuthUser
}

async fn get_device_keys(
    State(state): State<AppState>,
    Path(pubkey): Path<String>,
) -> Result<Json<Vec<DeviceKey>>, StatusCode> {
    let user = state
        .storage
        .get_user_by_pubkey(&pubkey)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let keys = state
        .storage
        .get_device_keys(&user.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(keys))
}

async fn add_device_key(
    State(state): State<AppState>,
    user: AuthUser,
    Json(device_key): Json<DeviceKey>,
) -> Result<StatusCode, StatusCode> {
    state
        .storage
        .add_device_key(&user.user_id, device_key)
        .await
        .map_err(|e| match e {
            StorageError::Conflict(_) => StatusCode::CONFLICT,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        })?;

    Ok(StatusCode::CREATED)
}

async fn sync_dms(
    State(state): State<AppState>,
    user: AuthUser,
    // we would extract the device_id from headers or query, 
    // but we can just use a query param for now.
    axum::extract::Query(params): axum::extract::Query<SyncParams>,
) -> Result<Json<Vec<PendingDM>>, StatusCode> {
    // Basic verification that the device belongs to the user
    let keys = state
        .storage
        .get_device_keys(&user.user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    if !keys.iter().any(|k| k.device_id == params.device_id) {
        return Err(StatusCode::FORBIDDEN);
    }

    let dms = state
        .storage
        .fetch_pending_dms(&params.device_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(dms))
}

#[derive(Debug, Deserialize)]
pub struct SyncParams {
    pub device_id: String,
}

async fn send_dm(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<SendDmRequest>,
) -> Result<StatusCode, StatusCode> {
    let db_user = state
        .storage
        .get_user_by_id(&user.user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Validate sender_pubkey matches authenticated user
    if req.dm.sender_pubkey != db_user.pubkey {
        return Err(StatusCode::FORBIDDEN);
    }

    state
        .storage
        .store_dm(req.dm, &req.target_device_ids)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Notify targeted devices
    for _device_id in req.target_device_ids {
        let envelope = shared::gateway::EventEnvelope {
            op: shared::gateway::Op::DmNotify,
            d: shared::gateway::DmNotifyData { pending_count: 1 },
            t: chrono::Utc::now().timestamp_millis(),
        };
        if let Ok(json) = serde_json::to_string(&envelope) {
            state.gateway.broadcast_all(&json);
        }
    }

    Ok(StatusCode::ACCEPTED)
}

async fn ack_dm(
    State(state): State<AppState>,
    _user: AuthUser, // require auth
    Json(req): Json<AckDmRequest>,
) -> Result<StatusCode, StatusCode> {
    state
        .storage
        .ack_dm(&req.message_id, &req.device_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}
