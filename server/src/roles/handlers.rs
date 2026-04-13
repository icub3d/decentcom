use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Serialize;
use shared::gateway::Op;

use crate::gateway::events::event_json;
use crate::permissions::{UserPermissions, ADMINISTRATOR, MANAGE_CHANNELS, MANAGE_ROLES};
use crate::roles::models::{
    ChannelOverrideResponse, CreateRoleRequest, ListChannelOverridesResponse, ListRolesResponse,
    MemberRoleEvent, RoleResponse, SetChannelOverrideRequest, UpdateRoleRequest,
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

fn bad_request(msg: impl Into<String>) -> (StatusCode, Json<ErrorBody>) {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorBody { error: msg.into() }),
    )
}

fn not_found(msg: impl Into<String>) -> (StatusCode, Json<ErrorBody>) {
    (
        StatusCode::NOT_FOUND,
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

fn ensure_manage_roles(auth: &UserPermissions) -> Result<(), (StatusCode, Json<ErrorBody>)> {
    if auth.has(MANAGE_ROLES) {
        return Ok(());
    }
    Err(forbidden("missing manage_roles permission"))
}

fn ensure_manage_channels(auth: &UserPermissions) -> Result<(), (StatusCode, Json<ErrorBody>)> {
    if auth.has(MANAGE_CHANNELS) {
        return Ok(());
    }
    Err(forbidden("missing manage_channels permission"))
}

fn ensure_higher_than(
    auth: &UserPermissions,
    target_position: i32,
) -> Result<(), (StatusCode, Json<ErrorBody>)> {
    if auth.has(ADMINISTRATOR) || auth.highest_role_position > target_position {
        return Ok(());
    }
    Err(forbidden(
        "cannot manage role at or above your highest role position",
    ))
}

pub(super) async fn list_roles(
    State(state): State<AppState>,
    _auth: UserPermissions,
) -> ApiResult<ListRolesResponse> {
    let roles = state
        .storage
        .list_roles()
        .await
        .map_err(internal)?
        .into_iter()
        .map(RoleResponse::from)
        .collect();

    Ok(Json(ListRolesResponse { roles }))
}

pub(super) async fn create_role(
    State(state): State<AppState>,
    auth: UserPermissions,
    Json(req): Json<CreateRoleRequest>,
) -> ApiResult<RoleResponse> {
    ensure_manage_roles(&auth)?;

    let position = req.position.unwrap_or(auth.highest_role_position - 1);
    if position >= auth.highest_role_position && !auth.has(ADMINISTRATOR) {
        return Err(forbidden("cannot create role at or above your own role"));
    }

    let role = state
        .storage
        .create_role(
            &req.name,
            req.color.as_deref(),
            req.permissions,
            position,
            false,
        )
        .await
        .map_err(storage_err)?;

    let response = RoleResponse::from(role);
    if let Some(msg) = event_json(Op::RoleCreate, response.clone()) {
        state.gateway.broadcast_all(&msg);
    }

    Ok(Json(response))
}

pub(super) async fn update_role(
    State(state): State<AppState>,
    auth: UserPermissions,
    Path(role_id): Path<String>,
    Json(req): Json<UpdateRoleRequest>,
) -> ApiResult<RoleResponse> {
    ensure_manage_roles(&auth)?;

    let existing = state
        .storage
        .get_role(&role_id)
        .await
        .map_err(internal)?
        .ok_or_else(|| not_found("role not found"))?;

    ensure_higher_than(&auth, existing.position)?;

    if existing.id == "admin" && req.permissions.is_some() {
        return Err(bad_request("@admin permissions cannot be modified"));
    }

    let updated = state
        .storage
        .update_role(
            &role_id,
            req.name.as_deref(),
            req.color.as_ref().map(|v| v.as_deref()),
            req.permissions,
            req.position,
        )
        .await
        .map_err(storage_err)?;

    let response = RoleResponse::from(updated);
    if let Some(msg) = event_json(Op::RoleUpdate, response.clone()) {
        state.gateway.broadcast_all(&msg);
    }

    Ok(Json(response))
}

pub(super) async fn delete_role(
    State(state): State<AppState>,
    auth: UserPermissions,
    Path(role_id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ErrorBody>)> {
    ensure_manage_roles(&auth)?;

    let role = state
        .storage
        .get_role(&role_id)
        .await
        .map_err(internal)?
        .ok_or_else(|| not_found("role not found"))?;

    if role.is_builtin || role.id == "everyone" || role.id == "admin" {
        return Err(bad_request("built-in roles cannot be deleted"));
    }

    ensure_higher_than(&auth, role.position)?;

    state.storage.delete_role(&role_id).await.map_err(storage_err)?;

    #[derive(Serialize)]
    struct DeletedRole {
        id: String,
    }

    if let Some(msg) = event_json(Op::RoleDelete, DeletedRole { id: role_id }) {
        state.gateway.broadcast_all(&msg);
    }

    Ok(StatusCode::NO_CONTENT)
}

pub(super) async fn add_member_role(
    State(state): State<AppState>,
    auth: UserPermissions,
    Path((pubkey, role_id)): Path<(String, String)>,
) -> Result<StatusCode, (StatusCode, Json<ErrorBody>)> {
    ensure_manage_roles(&auth)?;

    let user = state
        .storage
        .get_user_by_pubkey(&pubkey)
        .await
        .map_err(internal)?
        .ok_or_else(|| not_found("member not found"))?;

    let role = state
        .storage
        .get_role(&role_id)
        .await
        .map_err(internal)?
        .ok_or_else(|| not_found("role not found"))?;

    ensure_higher_than(&auth, role.position)?;

    state
        .storage
        .add_member_role(&user.id, &role.id)
        .await
        .map_err(storage_err)?;

    if let Some(msg) = event_json(
        Op::MemberRoleAdd,
        MemberRoleEvent {
            user_id: user.id,
            role_id: role.id,
        },
    ) {
        state.gateway.broadcast_all(&msg);
    }

    Ok(StatusCode::NO_CONTENT)
}

pub(super) async fn remove_member_role(
    State(state): State<AppState>,
    auth: UserPermissions,
    Path((pubkey, role_id)): Path<(String, String)>,
) -> Result<StatusCode, (StatusCode, Json<ErrorBody>)> {
    ensure_manage_roles(&auth)?;

    let user = state
        .storage
        .get_user_by_pubkey(&pubkey)
        .await
        .map_err(internal)?
        .ok_or_else(|| not_found("member not found"))?;

    let role = state
        .storage
        .get_role(&role_id)
        .await
        .map_err(internal)?
        .ok_or_else(|| not_found("role not found"))?;

    if role.id == "admin" {
        return Err(forbidden("@admin role cannot be removed"));
    }

    ensure_higher_than(&auth, role.position)?;

    state
        .storage
        .remove_member_role(&user.id, &role.id)
        .await
        .map_err(storage_err)?;

    if let Some(msg) = event_json(
        Op::MemberRoleRemove,
        MemberRoleEvent {
            user_id: user.id,
            role_id: role.id,
        },
    ) {
        state.gateway.broadcast_all(&msg);
    }

    Ok(StatusCode::NO_CONTENT)
}

pub(super) async fn list_channel_overrides(
    State(state): State<AppState>,
    auth: UserPermissions,
    Path(channel_id): Path<String>,
) -> ApiResult<ListChannelOverridesResponse> {
    ensure_manage_channels(&auth)?;

    let overrides = state
        .storage
        .list_channel_permission_overrides(&channel_id)
        .await
        .map_err(internal)?
        .into_iter()
        .map(ChannelOverrideResponse::from)
        .collect();

    Ok(Json(ListChannelOverridesResponse { overrides }))
}

pub(super) async fn set_channel_override(
    State(state): State<AppState>,
    auth: UserPermissions,
    Path((channel_id, role_id)): Path<(String, String)>,
    Json(req): Json<SetChannelOverrideRequest>,
) -> ApiResult<ChannelOverrideResponse> {
    ensure_manage_channels(&auth)?;

    let role = state
        .storage
        .get_role(&role_id)
        .await
        .map_err(internal)?
        .ok_or_else(|| not_found("role not found"))?;
    ensure_higher_than(&auth, role.position)?;

    let override_row = state
        .storage
        .upsert_channel_permission_override(&channel_id, &role_id, req.allow, req.deny)
        .await
        .map_err(storage_err)?;

    Ok(Json(ChannelOverrideResponse::from(override_row)))
}

pub(super) async fn delete_channel_override(
    State(state): State<AppState>,
    auth: UserPermissions,
    Path((channel_id, role_id)): Path<(String, String)>,
) -> Result<StatusCode, (StatusCode, Json<ErrorBody>)> {
    ensure_manage_channels(&auth)?;

    let role = state
        .storage
        .get_role(&role_id)
        .await
        .map_err(internal)?
        .ok_or_else(|| not_found("role not found"))?;
    ensure_higher_than(&auth, role.position)?;

    state
        .storage
        .delete_channel_permission_override(&channel_id, &role_id)
        .await
        .map_err(storage_err)?;

    Ok(StatusCode::NO_CONTENT)
}

