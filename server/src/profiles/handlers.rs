use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use axum_extra::extract::Multipart;
use serde::{Deserialize, Serialize};
use shared::gateway::Op;

use crate::gateway::events::event_json;
use crate::permissions::MemberUser;
use crate::profiles::avatar::{process_and_store_avatar, AvatarError};
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

fn internal(e: crate::storage::StorageError) -> (StatusCode, Json<ErrorBody>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorBody {
            error: e.to_string(),
        }),
    )
}

fn avatar_err(e: AvatarError) -> (StatusCode, Json<ErrorBody>) {
    match e {
        AvatarError::TooLarge | AvatarError::InvalidFormat(_) => bad_request(e.to_string()),
        AvatarError::ProcessingFailed(msg) => (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(ErrorBody { error: msg }),
        ),
        AvatarError::Storage(se) => internal(se),
    }
}

// ---------- Response types ----------

#[derive(Debug, Clone, Serialize)]
pub struct ProfileResponse {
    pub user_id: String,
    pub pubkey: String,
    pub display_name: Option<String>,
    pub avatar_hash: Option<String>,
    pub joined_at: Option<String>,
    pub roles: Vec<RoleSummary>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RoleSummary {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
    pub position: i32,
}

// ---------- Request types ----------

#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub display_name: Option<String>,
}

// ---------- Handlers ----------

/// PATCH /profile — update own display name.
pub(super) async fn update_profile(
    State(state): State<AppState>,
    auth: MemberUser,
    Json(req): Json<UpdateProfileRequest>,
) -> ApiResult<ProfileResponse> {
    let display_name = req
        .display_name
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty());

    if let Some(name) = display_name {
        if name.chars().count() > 32 {
            return Err(bad_request(
                "display name must be at most 32 characters",
            ));
        }
    } else if req.display_name.is_some() {
        // Caller sent display_name but it was empty/whitespace.
        return Err(bad_request("display name must not be empty"));
    }

    let user = state
        .storage
        .update_user(&auth.user_id, display_name, None)
        .await
        .map_err(internal)?;

    broadcast_member_update(&state, &user.id, &user.pubkey, &user.display_name, &user.avatar_hash);

    Ok(Json(to_profile_response(&state, user).await))
}

/// PUT /profile/avatar — upload or replace own avatar.
pub(super) async fn upload_avatar(
    State(state): State<AppState>,
    auth: MemberUser,
    mut multipart: Multipart,
) -> ApiResult<ProfileResponse> {
    let field = multipart
        .next_field()
        .await
        .map_err(|e| bad_request(format!("invalid multipart body: {e}")))?
        .ok_or_else(|| bad_request("missing file field"))?;

    let content_type = field.content_type().map(|s| s.to_string());
    let data = field
        .bytes()
        .await
        .map_err(|e| bad_request(format!("failed to read upload: {e}")))?;

    process_and_store_avatar(
        state.storage.as_ref(),
        &auth.user_id,
        &data,
        content_type.as_deref(),
    )
    .await
    .map_err(avatar_err)?;

    let user = state
        .storage
        .get_user_by_id(&auth.user_id)
        .await
        .map_err(internal)?
        .ok_or_else(|| not_found("user not found"))?;

    broadcast_member_update(&state, &user.id, &user.pubkey, &user.display_name, &user.avatar_hash);

    Ok(Json(to_profile_response(&state, user).await))
}

/// DELETE /profile/avatar — remove own avatar.
pub(super) async fn delete_avatar(
    State(state): State<AppState>,
    auth: MemberUser,
) -> ApiResult<ProfileResponse> {
    let user = state
        .storage
        .clear_avatar_hash(&auth.user_id)
        .await
        .map_err(internal)?;

    broadcast_member_update(&state, &user.id, &user.pubkey, &user.display_name, &user.avatar_hash);

    Ok(Json(to_profile_response(&state, user).await))
}

/// GET /members/:pubkey/profile — view a member's profile.
pub(super) async fn get_member_profile(
    State(state): State<AppState>,
    _auth: MemberUser,
    Path(pubkey): Path<String>,
) -> ApiResult<ProfileResponse> {
    let user = state
        .storage
        .get_user_by_pubkey(&pubkey)
        .await
        .map_err(internal)?
        .ok_or_else(|| not_found("user not found"))?;

    let member = state
        .storage
        .get_member_by_pubkey(&pubkey)
        .await
        .map_err(internal)?;

    let roles: Vec<RoleSummary> = state
        .storage
        .list_member_roles(&user.id)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|r| RoleSummary {
            id: r.id,
            name: r.name,
            color: r.color,
            position: r.position,
        })
        .collect();

    Ok(Json(ProfileResponse {
        user_id: user.id,
        pubkey: user.pubkey,
        display_name: user.display_name,
        avatar_hash: user.avatar_hash,
        joined_at: member.map(|m| m.joined_at.to_rfc3339()),
        roles,
    }))
}

// ---------- Helpers ----------

async fn to_profile_response(
    state: &AppState,
    user: crate::storage::models::User,
) -> ProfileResponse {
    let member = state
        .storage
        .get_member_by_pubkey(&user.pubkey)
        .await
        .ok()
        .flatten();

    let roles: Vec<RoleSummary> = state
        .storage
        .list_member_roles(&user.id)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|r| RoleSummary {
            id: r.id,
            name: r.name,
            color: r.color,
            position: r.position,
        })
        .collect();

    ProfileResponse {
        user_id: user.id,
        pubkey: user.pubkey,
        display_name: user.display_name,
        avatar_hash: user.avatar_hash,
        joined_at: member.map(|m| m.joined_at.to_rfc3339()),
        roles,
    }
}

fn broadcast_member_update(
    state: &AppState,
    user_id: &str,
    pubkey: &str,
    display_name: &Option<String>,
    avatar_hash: &Option<String>,
) {
    if let Some(msg) = event_json(
        Op::MemberUpdate,
        serde_json::json!({
            "user_id": user_id,
            "pubkey": pubkey,
            "display_name": display_name,
            "avatar_hash": avatar_hash,
        }),
    ) {
        state.gateway.broadcast_all(&msg);
    }
}
