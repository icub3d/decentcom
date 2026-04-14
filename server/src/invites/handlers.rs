use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::{Duration as ChronoDuration, Utc};
use serde::Serialize;
use shared::gateway::Op;

use crate::auth::middleware::AuthUser;
use crate::gateway::events::event_json;
use crate::invites::{build_invite_link, generate_code};
use crate::invites::models::{
    CreateInviteRequest, InvitePreviewResponse, InviteResponse, JoinInviteResponse, JoinedMember,
    ListInvitesResponse,
};
use crate::permissions::{UserPermissions, MANAGE_INVITES};
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

fn gone(msg: impl Into<String>) -> (StatusCode, Json<ErrorBody>) {
    (
        StatusCode::GONE,
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

fn ensure_manage_invites(auth: &UserPermissions) -> Result<(), (StatusCode, Json<ErrorBody>)> {
    if auth.has(MANAGE_INVITES) {
        return Ok(());
    }
    Err(forbidden("missing manage_invites permission"))
}

pub(super) async fn create_invite(
    State(state): State<AppState>,
    auth: UserPermissions,
    Json(req): Json<CreateInviteRequest>,
) -> ApiResult<InviteResponse> {
    ensure_manage_invites(&auth)?;

    let max_uses = req.max_uses.unwrap_or(0);
    if max_uses < 0 {
        return Err(bad_request("max_uses must be >= 0"));
    }

    let expires_at = match req.expires_in_seconds {
        Some(seconds) if seconds <= 0 => {
            return Err(bad_request("expires_in_seconds must be > 0 when provided"));
        }
        Some(seconds) => Some(Utc::now() + ChronoDuration::seconds(seconds)),
        None => None,
    };

    if let Some(grant_role_id) = req.grant_role_id.as_deref() {
        let role = state
            .storage
            .get_role(grant_role_id)
            .await
            .map_err(internal)?;
        if role.is_none() {
            return Err(bad_request("grant_role_id does not exist"));
        }
    }

    let mut invite = None;
    for _ in 0..8 {
        let code = generate_code();
        match state
            .storage
            .create_invite(
                &code,
                &auth.user_id,
                req.grant_role_id.as_deref(),
                max_uses,
                expires_at,
            )
            .await
        {
            Ok(created) => {
                invite = Some(created);
                break;
            }
            Err(crate::storage::StorageError::Conflict(_)) => continue,
            Err(e) => return Err(storage_err(e)),
        }
    }

    let Some(invite) = invite else {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorBody {
                error: "failed to allocate unique invite code".to_string(),
            }),
        ));
    };

    let link = build_invite_link(state.config.network.bind_address.to_string().as_str(), &invite.code);
    Ok(Json(InviteResponse::from_invite(invite, link)))
}

pub(super) async fn list_invites(
    State(state): State<AppState>,
    auth: UserPermissions,
) -> ApiResult<ListInvitesResponse> {
    ensure_manage_invites(&auth)?;

    let invites = state
        .storage
        .list_active_invites(Utc::now())
        .await
        .map_err(internal)?
        .into_iter()
        .map(|invite| {
            let link = build_invite_link(
                state.config.network.bind_address.to_string().as_str(),
                &invite.code,
            );
            InviteResponse::from_invite(invite, link)
        })
        .collect();

    Ok(Json(ListInvitesResponse { invites }))
}

pub(super) async fn preview_invite(
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> ApiResult<InvitePreviewResponse> {
    let invite = state
        .storage
        .get_invite(&code)
        .await
        .map_err(internal)?
        .ok_or_else(|| not_found("invite not found"))?;

    let now = Utc::now();
    if invite.expires_at.is_some_and(|expires_at| expires_at <= now) {
        return Err(gone("invite has expired"));
    }
    if invite.max_uses > 0 && invite.use_count >= invite.max_uses {
        return Err(gone("invite has been exhausted"));
    }

    let users = state.storage.list_users().await.map_err(internal)?;

    Ok(Json(InvitePreviewResponse {
        code: invite.code,
        server_name: state.config.server.name.clone(),
        server_icon: state
            .config
            .server
            .icon_path
            .as_ref()
            .map(|p| p.to_string_lossy().to_string()),
        member_count: users.len(),
        expires_at: invite.expires_at,
    }))
}

pub(super) async fn revoke_invite(
    State(state): State<AppState>,
    auth: UserPermissions,
    Path(code): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ErrorBody>)> {
    ensure_manage_invites(&auth)?;

    state.storage.revoke_invite(&code).await.map_err(storage_err)?;
    Ok(StatusCode::NO_CONTENT)
}

pub(super) async fn join_invite(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(code): Path<String>,
) -> ApiResult<JoinInviteResponse> {
    let invite = match state.storage.consume_invite(&code, Utc::now()).await {
        Ok(invite) => invite,
        Err(crate::storage::StorageError::Conflict(msg)) if msg.contains("expired") => {
            return Err(gone("invite has expired"));
        }
        Err(crate::storage::StorageError::Conflict(msg)) if msg.contains("exhausted") => {
            return Err(gone("invite has been exhausted"));
        }
        Err(e) => return Err(storage_err(e)),
    };

    if !state
        .storage
        .user_has_role(&auth.user_id, "everyone")
        .await
        .map_err(internal)?
    {
        let _ = state.storage.add_member_role(&auth.user_id, "everyone").await;
    }

    if let Some(grant_role_id) = invite.grant_role_id.as_deref() {
        if !state
            .storage
            .user_has_role(&auth.user_id, grant_role_id)
            .await
            .map_err(internal)?
        {
            let _ = state
                .storage
                .add_member_role(&auth.user_id, grant_role_id)
                .await;
        }
    }

    let user = state
        .storage
        .get_user_by_id(&auth.user_id)
        .await
        .map_err(internal)?
        .ok_or_else(|| not_found("user not found"))?;

    let role_ids = state
        .storage
        .list_member_roles(&auth.user_id)
        .await
        .map_err(internal)?
        .into_iter()
        .map(|role| role.id)
        .collect::<Vec<_>>();

    let joined_at = Utc::now();
    let response = JoinInviteResponse {
        member: JoinedMember {
            pubkey: user.pubkey.clone(),
            roles: role_ids.clone(),
            joined_at,
        },
    };

    if let Some(msg) = event_json(
        Op::MemberJoin,
        serde_json::json!({
            "user_id": user.id,
            "pubkey": user.pubkey,
            "roles": role_ids,
            "joined_at": joined_at,
        }),
    ) {
        state.gateway.broadcast_all(&msg);
    }

    Ok(Json(response))
}
