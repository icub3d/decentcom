use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::Utc;
use serde::Serialize;
use shared::gateway::Op;

use crate::auth::middleware::AuthUser;
use crate::config::MembershipMode;
use crate::gateway::events::event_json;
use crate::membership::models::{
    AllowlistRequest, AllowlistEntryResponse, ApproveBotRequest, BotApprovalResponse, BanRequest,
    BanResponse, JoinMemberResponse, ListAllowlistResponse, ListBansResponse, ListMembersResponse,
    ListPendingBotsResponse, MemberWithRoles, RoleSummary,
};
use crate::permissions::{MemberUser, UserPermissions, ADMINISTRATOR, BAN_MEMBERS, KICK_MEMBERS, MANAGE_SERVER};
use crate::storage::models::Member;
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

fn is_valid_pubkey(pubkey: &str) -> bool {
    bs58::decode(pubkey)
        .into_vec()
        .map(|bytes| bytes.len() == 32)
        .unwrap_or(false)
}

async fn member_to_response(
    state: &AppState,
    member: Member,
) -> Result<MemberWithRoles, crate::storage::StorageError> {
    let roles = state
        .storage
        .list_member_roles(&member.user_id)
        .await?
        .into_iter()
        .map(RoleSummary::from)
        .collect();

    Ok(MemberWithRoles {
        user_id: member.user_id,
        pubkey: member.pubkey,
        display_name: member.display_name,
        avatar_hash: member.avatar_hash,
        is_bot: member.is_bot,
        joined_at: member.joined_at,
        roles,
    })
}

fn ensure_higher_than(
    auth: &MemberUser,
    target_position: i32,
) -> Result<(), (StatusCode, Json<ErrorBody>)> {
    if auth.has(ADMINISTRATOR) || auth.highest_role_position > target_position {
        return Ok(());
    }
    Err(forbidden(
        "cannot manage a member with an equal or higher role",
    ))
}

async fn enforce_join_policy(
    state: &AppState,
    user_id: &str,
    pubkey: &str,
    via_invite: bool,
) -> Result<(), (StatusCode, Json<ErrorBody>)> {
    if state
        .storage
        .is_banned_pubkey(pubkey)
        .await
        .map_err(internal)?
    {
        return Err(forbidden("this pubkey is banned from the server"));
    }

    match state.config.membership.mode {
        MembershipMode::Open => Ok(()),
        MembershipMode::InviteOnly => {
            if via_invite {
                Ok(())
            } else {
                Err(forbidden("server is invite-only"))
            }
        }
        MembershipMode::Allowlist => {
            let allowed = state
                .storage
                .is_allowlisted_pubkey(pubkey)
                .await
                .map_err(internal)?;
            if allowed {
                Ok(())
            } else {
                Err(forbidden("pubkey is not on the allowlist"))
            }
        }
        MembershipMode::Closed => {
            if state.storage.is_member(user_id).await.map_err(internal)? {
                Ok(())
            } else {
                Err(forbidden("server membership is closed"))
            }
        }
    }
}

pub(super) async fn join_member(
    State(state): State<AppState>,
    auth: AuthUser,
) -> ApiResult<JoinMemberResponse> {
    let user = state
        .storage
        .get_user_by_id(&auth.user_id)
        .await
        .map_err(internal)?
        .ok_or_else(|| not_found("user not found"))?;

    let already_member = state
        .storage
        .is_member(&auth.user_id)
        .await
        .map_err(internal)?;

    if !already_member {
        enforce_join_policy(&state, &auth.user_id, &user.pubkey, false).await?;

        state.storage.add_member(&auth.user_id).await.map_err(storage_err)?;

        // Automatically assign roles
        if !state
            .storage
            .user_has_role(&auth.user_id, "everyone")
            .await
            .map_err(internal)?
        {
            let _ = state.storage.add_member_role(&auth.user_id, "everyone").await;
        }

        // If this is the first member, make them an admin
        let all_members = state.storage.list_members().await.map_err(internal)?;
        if all_members.len() == 1 {
            let _ = state.storage.add_member_role(&auth.user_id, "admin").await;
        }

        if let Some(msg) = event_json(
            Op::MemberJoin,
            serde_json::json!({
                "user_id": user.id,
                "pubkey": user.pubkey,
                "display_name": user.display_name,
                "avatar_hash": user.avatar_hash,
                "is_bot": user.is_bot,
                "joined_at": Utc::now(),
                "roles": [],
            }),
        ) {
            state.gateway.broadcast_all(&msg);
        }
    }

    let member = state
        .storage
        .get_member_by_pubkey(&user.pubkey)
        .await
        .map_err(internal)?
        .ok_or_else(|| not_found("member not found"))?;

    Ok(Json(JoinMemberResponse {
        member: member_to_response(&state, member).await.map_err(internal)?,
    }))
}

pub(super) async fn leave_member(
    State(state): State<AppState>,
    auth: MemberUser,
) -> Result<StatusCode, (StatusCode, Json<ErrorBody>)> {
    let user = state
        .storage
        .get_user_by_id(&auth.user_id)
        .await
        .map_err(internal)?
        .ok_or_else(|| not_found("user not found"))?;

    state
        .storage
        .remove_all_member_roles(&auth.user_id)
        .await
        .map_err(internal)?;
    state
        .storage
        .remove_member(&auth.user_id)
        .await
        .map_err(storage_err)?;

    if let Some(msg) = event_json(
        Op::MemberLeave,
        serde_json::json!({
            "user_id": user.id,
            "pubkey": user.pubkey,
            "left_at": Utc::now(),
        }),
    ) {
        state.gateway.broadcast_all(&msg);
    }

    Ok(StatusCode::NO_CONTENT)
}

pub(super) async fn list_members(
    State(state): State<AppState>,
    _auth: MemberUser,
) -> ApiResult<ListMembersResponse> {
    let mut members = Vec::new();
    for member in state.storage.list_members().await.map_err(internal)? {
        members.push(member_to_response(&state, member).await.map_err(internal)?);
    }

    Ok(Json(ListMembersResponse { members }))
}

pub(super) async fn get_member(
    State(state): State<AppState>,
    _auth: MemberUser,
    Path(pubkey): Path<String>,
) -> ApiResult<MemberWithRoles> {
    let member = state
        .storage
        .get_member_by_pubkey(&pubkey)
        .await
        .map_err(internal)?
        .ok_or_else(|| not_found("member not found"))?;

    let response = member_to_response(&state, member).await.map_err(internal)?;
    Ok(Json(response))
}

pub(super) async fn kick_member(
    State(state): State<AppState>,
    auth: UserPermissions,
    Path(pubkey): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ErrorBody>)> {
    if !auth.has(KICK_MEMBERS) {
        return Err(forbidden("missing kick_members permission"));
    }

    let member = state
        .storage
        .get_member_by_pubkey(&pubkey)
        .await
        .map_err(internal)?
        .ok_or_else(|| not_found("member not found"))?;

    let target_roles = state
        .storage
        .list_member_roles(&member.user_id)
        .await
        .map_err(internal)?;
    let target_highest = target_roles.iter().map(|role| role.position).max().unwrap_or(0);

    ensure_higher_than(&auth, target_highest)?;

    state
        .storage
        .remove_all_member_roles(&member.user_id)
        .await
        .map_err(internal)?;
    state
        .storage
        .remove_member(&member.user_id)
        .await
        .map_err(storage_err)?;

    if let Some(msg) = event_json(
        Op::MemberKick,
        serde_json::json!({
            "user_id": member.user_id,
            "pubkey": member.pubkey,
            "kicked_by": auth.user_id,
            "kicked_at": Utc::now(),
        }),
    ) {
        state.gateway.broadcast_all(&msg);
    }

    Ok(StatusCode::NO_CONTENT)
}

pub(super) async fn ban_member(
    State(state): State<AppState>,
    auth: UserPermissions,
    Path(pubkey): Path<String>,
    Json(req): Json<BanRequest>,
) -> ApiResult<BanResponse> {
    if !auth.has(BAN_MEMBERS) {
        return Err(forbidden("missing ban_members permission"));
    }

    if !is_valid_pubkey(&pubkey) {
        return Err(bad_request("invalid public key format"));
    }

    if let Some(member) = state
        .storage
        .get_member_by_pubkey(&pubkey)
        .await
        .map_err(internal)?
    {
        let target_roles = state
            .storage
            .list_member_roles(&member.user_id)
            .await
            .map_err(internal)?;
        let target_highest = target_roles.iter().map(|role| role.position).max().unwrap_or(0);

        ensure_higher_than(&auth, target_highest)?;

        state
            .storage
            .remove_all_member_roles(&member.user_id)
            .await
            .map_err(internal)?;
        let _ = state.storage.remove_member(&member.user_id).await;
    }

    let ban = state
        .storage
        .add_ban(&pubkey, &auth.user_id, req.reason.as_deref())
        .await
        .map_err(storage_err)?;

    if let Some(msg) = event_json(
        Op::MemberBan,
        serde_json::json!({
            "pubkey": ban.pubkey,
            "banned_by": ban.banned_by,
            "reason": ban.reason,
            "banned_at": ban.banned_at,
        }),
    ) {
        state.gateway.broadcast_all(&msg);
    }

    Ok(Json(BanResponse::from(ban)))
}

pub(super) async fn unban_member(
    State(state): State<AppState>,
    auth: UserPermissions,
    Path(pubkey): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ErrorBody>)> {
    if !auth.has(BAN_MEMBERS) {
        return Err(forbidden("missing ban_members permission"));
    }

    state.storage.remove_ban(&pubkey).await.map_err(storage_err)?;
    Ok(StatusCode::NO_CONTENT)
}

pub(super) async fn list_bans(
    State(state): State<AppState>,
    auth: UserPermissions,
) -> ApiResult<ListBansResponse> {
    if !auth.has(BAN_MEMBERS) {
        return Err(forbidden("missing ban_members permission"));
    }

    let bans = state
        .storage
        .list_bans()
        .await
        .map_err(internal)?
        .into_iter()
        .map(BanResponse::from)
        .collect();

    Ok(Json(ListBansResponse { bans }))
}

pub(super) async fn list_allowlist(
    State(state): State<AppState>,
    auth: UserPermissions,
) -> ApiResult<ListAllowlistResponse> {
    if !auth.has(MANAGE_SERVER) {
        return Err(forbidden("missing manage_server permission"));
    }

    let entries = state
        .storage
        .list_allowlist_entries()
        .await
        .map_err(internal)?
        .into_iter()
        .map(AllowlistEntryResponse::from)
        .collect();

    Ok(Json(ListAllowlistResponse { entries }))
}

pub(super) async fn add_allowlist(
    State(state): State<AppState>,
    auth: UserPermissions,
    Json(req): Json<AllowlistRequest>,
) -> ApiResult<AllowlistEntryResponse> {
    if !auth.has(MANAGE_SERVER) {
        return Err(forbidden("missing manage_server permission"));
    }

    if !is_valid_pubkey(&req.pubkey) {
        return Err(bad_request("invalid public key format"));
    }

    let entry = state
        .storage
        .add_allowlist_entry(&req.pubkey, &auth.user_id)
        .await
        .map_err(storage_err)?;

    Ok(Json(AllowlistEntryResponse::from(entry)))
}

pub(super) async fn remove_allowlist(
    State(state): State<AppState>,
    auth: UserPermissions,
    Path(pubkey): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ErrorBody>)> {
    if !auth.has(MANAGE_SERVER) {
        return Err(forbidden("missing manage_server permission"));
    }

    state
        .storage
        .remove_allowlist_entry(&pubkey)
        .await
        .map_err(storage_err)?;
    Ok(StatusCode::NO_CONTENT)
}

pub(super) async fn list_pending_bots(
    State(state): State<AppState>,
    auth: UserPermissions,
) -> ApiResult<ListPendingBotsResponse> {
    if !auth.has(MANAGE_SERVER) {
        return Err(forbidden("missing manage_server permission"));
    }

    let bots = state
        .storage
        .list_pending_bots()
        .await
        .map_err(internal)?
        .into_iter()
        .map(BotApprovalResponse::from)
        .collect();

    Ok(Json(ListPendingBotsResponse { bots }))
}

pub(super) async fn approve_bot(
    State(state): State<AppState>,
    auth: UserPermissions,
    Path(pubkey): Path<String>,
    Json(req): Json<ApproveBotRequest>,
) -> ApiResult<BotApprovalResponse> {
    if !auth.has(MANAGE_SERVER) {
        return Err(forbidden("missing manage_server permission"));
    }

    let approval = state
        .storage
        .approve_bot(&pubkey, &auth.user_id, req.note.as_deref())
        .await
        .map_err(storage_err)?;

    // Auto-join: add the bot as a member so it's immediately visible in the member list.
    if let Some(user) = state.storage.get_user_by_pubkey(&pubkey).await.map_err(internal)? {
        if !state.storage.is_member(&user.id).await.map_err(internal)? {
            let _ = state.storage.add_member(&user.id).await;
            let _ = state.storage.add_member_role(&user.id, "everyone").await;

            if let Some(msg) = event_json(
                Op::MemberJoin,
                serde_json::json!({
                    "user_id": user.id,
                    "pubkey": user.pubkey,
                    "is_bot": user.is_bot,
                    "joined_at": Utc::now(),
                    "roles": [],
                }),
            ) {
                state.gateway.broadcast_all(&msg);
            }
        }
    }

    Ok(Json(BotApprovalResponse::from(approval)))
}

pub(super) async fn revoke_bot(
    State(state): State<AppState>,
    auth: UserPermissions,
    Path(pubkey): Path<String>,
) -> ApiResult<BotApprovalResponse> {
    if !auth.has(MANAGE_SERVER) {
        return Err(forbidden("missing manage_server permission"));
    }

    let approval = state
        .storage
        .revoke_bot(&pubkey)
        .await
        .map_err(storage_err)?;

    Ok(Json(BotApprovalResponse::from(approval)))
}

