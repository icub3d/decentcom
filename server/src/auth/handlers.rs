use std::time::Duration;

use axum::extract::State;
use axum::{http::StatusCode, Json};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use rand::RngCore;
use serde::Serialize;
use shared::auth::{
    AuthMeResponse, ChallengeRequest, ChallengeResponse, VerifyRequest, VerifyResponse,
};

use crate::auth::middleware::AuthUser;
use crate::AppState;

#[derive(Debug, Serialize)]
pub(super) struct ErrorBody {
    error: String,
}

type ApiResult<T> = Result<Json<T>, (StatusCode, Json<ErrorBody>)>;

pub(super) async fn challenge(
    State(state): State<AppState>,
    Json(req): Json<ChallengeRequest>,
) -> ApiResult<ChallengeResponse> {
    if parse_pubkey(&req.pubkey).is_err() {
        return Err(bad_request("invalid public key format"));
    }

    let mut nonce = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut nonce);

    state.challenge_store.insert(req.pubkey, nonce.to_vec());

    Ok(Json(ChallengeResponse {
        challenge: BASE64.encode(nonce),
    }))
}

pub(super) async fn verify(
    State(state): State<AppState>,
    Json(req): Json<VerifyRequest>,
) -> ApiResult<VerifyResponse> {
    let verifying_key =
        parse_pubkey(&req.pubkey).map_err(|_| bad_request("invalid public key format"))?;
    let signature =
        parse_signature(&req.signature).map_err(|_| bad_request("invalid signature format"))?;

    let nonce = state
        .challenge_store
        .take(&req.pubkey)
        .ok_or_else(|| unauthorized("challenge not found or expired"))?;

    // The client signs the base64 challenge string it receives.
    let challenge_text = BASE64.encode(nonce);
    verifying_key
        .verify(challenge_text.as_bytes(), &signature)
        .map_err(|_| unauthorized("signature verification failed"))?;

    let mut created_user = false;
    let user = match state
        .storage
        .get_user_by_pubkey(&req.pubkey)
        .await
        .map_err(internal)?
    {
        Some(existing) => existing,
        None => {
            created_user = true;
            state
                .storage
                .create_user(&req.pubkey, None)
                .await
                .map_err(internal)?
        }
    };

    if created_user {
        let _ = state.storage.add_member_role(&user.id, "everyone").await;
        let users = state.storage.list_users().await.map_err(internal)?;
        if users.len() == 1 {
            let _ = state.storage.add_member_role(&user.id, "admin").await;
        }
    }

    let session = state
        .storage
        .create_session(
            &user.id,
            Duration::from_secs(state.config.auth.session_ttl_seconds),
        )
        .await
        .map_err(internal)?;

    Ok(Json(VerifyResponse {
        token: session.token,
        user_id: user.id,
        expires_at: session.expires_at.to_rfc3339(),
    }))
}

pub(super) async fn me(auth_user: AuthUser) -> Json<AuthMeResponse> {
    Json(AuthMeResponse {
        user_id: auth_user.user_id,
    })
}

fn parse_pubkey(pubkey: &str) -> Result<VerifyingKey, ()> {
    let bytes = bs58::decode(pubkey).into_vec().map_err(|_| ())?;
    if bytes.len() != 32 {
        return Err(());
    }
    let key_bytes: [u8; 32] = bytes.as_slice().try_into().map_err(|_| ())?;
    VerifyingKey::from_bytes(&key_bytes).map_err(|_| ())
}

fn parse_signature(signature: &str) -> Result<Signature, ()> {
    let bytes = BASE64.decode(signature).map_err(|_| ())?;
    Signature::from_slice(&bytes).map_err(|_| ())
}

fn bad_request(msg: &str) -> (StatusCode, Json<ErrorBody>) {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorBody {
            error: msg.to_string(),
        }),
    )
}

fn unauthorized(msg: &str) -> (StatusCode, Json<ErrorBody>) {
    (
        StatusCode::UNAUTHORIZED,
        Json(ErrorBody {
            error: msg.to_string(),
        }),
    )
}

fn internal<E: std::fmt::Display>(err: E) -> (StatusCode, Json<ErrorBody>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorBody {
            error: format!("internal error: {err}"),
        }),
    )
}
