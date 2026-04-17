mod handlers;
pub mod models;

use axum::routing::{get, post};
use axum::Router;
use rand::Rng;

use crate::AppState;

const INVITE_CODE_LEN: usize = 8;
const INVITE_CODE_ALPHABET: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/invites", post(handlers::create_invite).get(handlers::list_invites))
        .route("/invites/:code", get(handlers::preview_invite).delete(handlers::revoke_invite))
        .route("/invites/:code/join", post(handlers::join_invite))
}

pub fn generate_code() -> String {
    let mut rng = rand::rng();
    let mut code = String::with_capacity(INVITE_CODE_LEN);
    for _ in 0..INVITE_CODE_LEN {
        let index = rng.random_range(0..INVITE_CODE_ALPHABET.len());
        code.push(INVITE_CODE_ALPHABET[index] as char);
    }
    code
}

pub fn build_invite_link(server_address: &str, code: &str) -> String {
    let address = if server_address.starts_with("http://") || server_address.starts_with("https://") {
        server_address.to_string()
    } else {
        format!("http://{server_address}")
    };
    format!("{address}/invite/{code}")
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use base64::engine::general_purpose::STANDARD as BASE64;
    use base64::Engine;
    use ed25519_dalek::Signer;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    use crate::config::{MembershipMode, ServerConfig};
    use crate::invites::models::{InvitePreviewResponse, InviteResponse, JoinInviteResponse, ListInvitesResponse};
    use crate::storage::{DynStorage, SqliteStorage};
    use crate::{app, AppState};

    use super::*;

    async fn test_state() -> AppState {
        let storage: DynStorage = Arc::new(SqliteStorage::in_memory().await.unwrap());
        AppState {
            config: Arc::new(ServerConfig::default()),
            storage,
            challenge_store: crate::auth::challenge_store(),
            gateway: crate::gateway::gateway_handle(),
        }
    }

    async fn invite_only_state() -> AppState {
        let mut cfg = ServerConfig::default();
        cfg.membership.mode = MembershipMode::InviteOnly;
        let storage: DynStorage = Arc::new(SqliteStorage::in_memory().await.unwrap());
        AppState {
            config: Arc::new(cfg),
            storage,
            challenge_store: crate::auth::challenge_store(),
            gateway: crate::gateway::gateway_handle(),
        }
    }

    async fn authed_session(state: &AppState, seed: u8) -> (String, String, String) {
        let signing_key = ed25519_dalek::SigningKey::from_bytes(&[seed; 32]);
        let pubkey = bs58::encode(signing_key.verifying_key().as_bytes()).into_string();

        let challenge_req = Request::builder()
            .method("POST")
            .uri("/api/v1/auth/challenge")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({ "pubkey": pubkey }).to_string(),
            ))
            .unwrap();
        let challenge_resp = app(state.clone()).oneshot(challenge_req).await.unwrap();
        let challenge_body = challenge_resp.into_body().collect().await.unwrap().to_bytes();
        let challenge: shared::auth::ChallengeResponse =
            serde_json::from_slice(&challenge_body).unwrap();

        let signature = BASE64.encode(signing_key.sign(challenge.challenge.as_bytes()).to_bytes());
        let verify_req = Request::builder()
            .method("POST")
            .uri("/api/v1/auth/verify")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({ "pubkey": pubkey, "signature": signature }).to_string(),
            ))
            .unwrap();
        let verify_resp = app(state.clone()).oneshot(verify_req).await.unwrap();
        let verify_body = verify_resp.into_body().collect().await.unwrap().to_bytes();
        let verify: shared::auth::VerifyResponse = serde_json::from_slice(&verify_body).unwrap();

        (verify.token, verify.user_id, pubkey)
    }

    #[test]
    fn generated_code_is_base62_and_correct_length() {
        let code = generate_code();
        assert_eq!(code.len(), INVITE_CODE_LEN);
        assert!(code
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric()));
    }

    #[test]
    fn invite_link_uses_http_prefix_when_missing() {
        let link = build_invite_link("127.0.0.1:8080", "abcd1234");
        assert_eq!(link, "http://127.0.0.1:8080/invite/abcd1234");
    }

    #[tokio::test]
    async fn create_and_list_invites() {
        let state = test_state().await;
        let (admin_token, _, _) = authed_session(&state, 71).await;

        let create_req = Request::builder()
            .method("POST")
            .uri("/api/v1/invites")
            .header("authorization", format!("Bearer {admin_token}"))
            .header("content-type", "application/json")
            .body(Body::from(r#"{"max_uses":3,"expires_in_seconds":3600}"#))
            .unwrap();
        let create_resp = app(state.clone()).oneshot(create_req).await.unwrap();
        assert_eq!(create_resp.status(), StatusCode::OK);
        let create_body = create_resp.into_body().collect().await.unwrap().to_bytes();
        let created: InviteResponse = serde_json::from_slice(&create_body).unwrap();
        assert_eq!(created.max_uses, 3);

        let list_req = Request::builder()
            .method("GET")
            .uri("/api/v1/invites")
            .header("authorization", format!("Bearer {admin_token}"))
            .body(Body::empty())
            .unwrap();
        let list_resp = app(state.clone()).oneshot(list_req).await.unwrap();
        assert_eq!(list_resp.status(), StatusCode::OK);
        let list_body = list_resp.into_body().collect().await.unwrap().to_bytes();
        let listed: ListInvitesResponse = serde_json::from_slice(&list_body).unwrap();
        assert!(listed.invites.iter().any(|invite| invite.code == created.code));
    }

    #[tokio::test]
    async fn invite_preview_is_public() {
        let state = test_state().await;
        let (admin_token, _, _) = authed_session(&state, 72).await;

        let create_req = Request::builder()
            .method("POST")
            .uri("/api/v1/invites")
            .header("authorization", format!("Bearer {admin_token}"))
            .header("content-type", "application/json")
            .body(Body::from(r#"{}"#))
            .unwrap();
        let create_resp = app(state.clone()).oneshot(create_req).await.unwrap();
        let create_body = create_resp.into_body().collect().await.unwrap().to_bytes();
        let created: InviteResponse = serde_json::from_slice(&create_body).unwrap();

        let preview_req = Request::builder()
            .method("GET")
            .uri(format!("/api/v1/invites/{}", created.code))
            .body(Body::empty())
            .unwrap();
        let preview_resp = app(state.clone()).oneshot(preview_req).await.unwrap();
        assert_eq!(preview_resp.status(), StatusCode::OK);
        let preview_body = preview_resp.into_body().collect().await.unwrap().to_bytes();
        let preview: InvitePreviewResponse = serde_json::from_slice(&preview_body).unwrap();
        assert_eq!(preview.code, created.code);
    }

    #[tokio::test]
    async fn user_without_manage_invites_cannot_create_or_list() {
        let state = test_state().await;
        let (_admin_token, _, _) = authed_session(&state, 73).await;
        let (user_token, _, _) = authed_session(&state, 74).await;

        let create_req = Request::builder()
            .method("POST")
            .uri("/api/v1/invites")
            .header("authorization", format!("Bearer {user_token}"))
            .header("content-type", "application/json")
            .body(Body::from(r#"{}"#))
            .unwrap();
        let create_resp = app(state.clone()).oneshot(create_req).await.unwrap();
        assert_eq!(create_resp.status(), StatusCode::FORBIDDEN);

        let list_req = Request::builder()
            .method("GET")
            .uri("/api/v1/invites")
            .header("authorization", format!("Bearer {user_token}"))
            .body(Body::empty())
            .unwrap();
        let list_resp = app(state.clone()).oneshot(list_req).await.unwrap();
        assert_eq!(list_resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn single_use_invite_fails_on_second_join() {
        let state = test_state().await;
        let (admin_token, _, _) = authed_session(&state, 75).await;
        let (member_token, _, _) = authed_session(&state, 76).await;
        let (member2_token, _, _) = authed_session(&state, 77).await;

        let create_req = Request::builder()
            .method("POST")
            .uri("/api/v1/invites")
            .header("authorization", format!("Bearer {admin_token}"))
            .header("content-type", "application/json")
            .body(Body::from(r#"{"max_uses":1}"#))
            .unwrap();
        let create_resp = app(state.clone()).oneshot(create_req).await.unwrap();
        let create_body = create_resp.into_body().collect().await.unwrap().to_bytes();
        let created: InviteResponse = serde_json::from_slice(&create_body).unwrap();

        let join_req = Request::builder()
            .method("POST")
            .uri(format!("/api/v1/invites/{}/join", created.code))
            .header("authorization", format!("Bearer {member_token}"))
            .body(Body::empty())
            .unwrap();
        let join_resp = app(state.clone()).oneshot(join_req).await.unwrap();
        assert_eq!(join_resp.status(), StatusCode::OK);

        let join_req_2 = Request::builder()
            .method("POST")
            .uri(format!("/api/v1/invites/{}/join", created.code))
            .header("authorization", format!("Bearer {member2_token}"))
            .body(Body::empty())
            .unwrap();
        let join_resp_2 = app(state.clone()).oneshot(join_req_2).await.unwrap();
        assert_eq!(join_resp_2.status(), StatusCode::GONE);
    }

    #[tokio::test]
    async fn invite_join_assigns_grant_role() {
        let state = test_state().await;
        let (admin_token, _, _) = authed_session(&state, 78).await;
        let (member_token, member_user_id, _) = authed_session(&state, 79).await;

        let role_create_req = Request::builder()
            .method("POST")
            .uri("/api/v1/roles")
            .header("authorization", format!("Bearer {admin_token}"))
            .header("content-type", "application/json")
            .body(Body::from(r#"{"name":"vip","permissions":1,"position":5}"#))
            .unwrap();
        let role_create_resp = app(state.clone()).oneshot(role_create_req).await.unwrap();
        assert_eq!(role_create_resp.status(), StatusCode::OK);
        let role_body = role_create_resp.into_body().collect().await.unwrap().to_bytes();
        let created_role: crate::roles::models::RoleResponse =
            serde_json::from_slice(&role_body).unwrap();

        let create_invite_req = Request::builder()
            .method("POST")
            .uri("/api/v1/invites")
            .header("authorization", format!("Bearer {admin_token}"))
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({ "grant_role_id": created_role.id }).to_string(),
            ))
            .unwrap();
        let create_invite_resp = app(state.clone()).oneshot(create_invite_req).await.unwrap();
        let invite_body = create_invite_resp.into_body().collect().await.unwrap().to_bytes();
        let created_invite: InviteResponse = serde_json::from_slice(&invite_body).unwrap();

        let join_req = Request::builder()
            .method("POST")
            .uri(format!("/api/v1/invites/{}/join", created_invite.code))
            .header("authorization", format!("Bearer {member_token}"))
            .body(Body::empty())
            .unwrap();
        let join_resp = app(state.clone()).oneshot(join_req).await.unwrap();
        assert_eq!(join_resp.status(), StatusCode::OK);

        let join_body = join_resp.into_body().collect().await.unwrap().to_bytes();
        let joined: JoinInviteResponse = serde_json::from_slice(&join_body).unwrap();
        assert!(joined.member.roles.iter().any(|role_id| role_id == "everyone"));
        assert!(joined
            .member
            .roles
            .iter()
            .any(|role_id| role_id == &created_role.id));

        let roles = state.storage.list_member_roles(&member_user_id).await.unwrap();
        assert!(roles.iter().any(|role| role.id == created_role.id));
    }

    #[tokio::test]
    async fn revoked_invite_cannot_be_used() {
        let state = test_state().await;
        let (admin_token, _, _) = authed_session(&state, 80).await;
        let (member_token, _, _) = authed_session(&state, 81).await;

        let create_req = Request::builder()
            .method("POST")
            .uri("/api/v1/invites")
            .header("authorization", format!("Bearer {admin_token}"))
            .header("content-type", "application/json")
            .body(Body::from(r#"{}"#))
            .unwrap();
        let create_resp = app(state.clone()).oneshot(create_req).await.unwrap();
        let create_body = create_resp.into_body().collect().await.unwrap().to_bytes();
        let created: InviteResponse = serde_json::from_slice(&create_body).unwrap();

        let revoke_req = Request::builder()
            .method("DELETE")
            .uri(format!("/api/v1/invites/{}", created.code))
            .header("authorization", format!("Bearer {admin_token}"))
            .body(Body::empty())
            .unwrap();
        let revoke_resp = app(state.clone()).oneshot(revoke_req).await.unwrap();
        assert_eq!(revoke_resp.status(), StatusCode::NO_CONTENT);

        let join_req = Request::builder()
            .method("POST")
            .uri(format!("/api/v1/invites/{}/join", created.code))
            .header("authorization", format!("Bearer {member_token}"))
            .body(Body::empty())
            .unwrap();
        let join_resp = app(state.clone()).oneshot(join_req).await.unwrap();
        assert_eq!(join_resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn invite_only_mode_allows_auth_then_join() {
        let state = invite_only_state().await;
        let (admin_token, _, _) = authed_session(&state, 82).await;
        let (member_token, _, _) = authed_session(&state, 83).await;

        let create_req = Request::builder()
            .method("POST")
            .uri("/api/v1/invites")
            .header("authorization", format!("Bearer {admin_token}"))
            .header("content-type", "application/json")
            .body(Body::from(r#"{}"#))
            .unwrap();
        let create_resp = app(state.clone()).oneshot(create_req).await.unwrap();
        let create_body = create_resp.into_body().collect().await.unwrap().to_bytes();
        let created: InviteResponse = serde_json::from_slice(&create_body).unwrap();

        let join_req = Request::builder()
            .method("POST")
            .uri(format!("/api/v1/invites/{}/join", created.code))
            .header("authorization", format!("Bearer {member_token}"))
            .body(Body::empty())
            .unwrap();
        let join_resp = app(state.clone()).oneshot(join_req).await.unwrap();
        assert_eq!(join_resp.status(), StatusCode::OK);
    }
}
