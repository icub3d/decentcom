mod handlers;
pub mod models;

use axum::routing::{get, patch, put};
use axum::Router;

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/roles", get(handlers::list_roles).post(handlers::create_role))
        .route(
            "/roles/:role_id",
            patch(handlers::update_role).delete(handlers::delete_role),
        )
        .route(
            "/members/:pubkey/roles/:role_id",
            put(handlers::add_member_role).delete(handlers::remove_member_role),
        )
        .route(
            "/channels/:channel_id/overrides",
            get(handlers::list_channel_overrides),
        )
        .route(
            "/channels/:channel_id/overrides/:role_id",
            put(handlers::set_channel_override).delete(handlers::delete_channel_override),
        )
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

    use crate::config::ServerConfig;
    use crate::roles::models::{ListRolesResponse, RoleResponse};
    use crate::storage::{DynStorage, SqliteStorage};
    use crate::{app, AppState};

    async fn test_state() -> AppState {
        let storage: DynStorage = Arc::new(SqliteStorage::in_memory().await.unwrap());
        AppState {
            config: Arc::new(ServerConfig::default()),
            storage,
            challenge_store: crate::auth::challenge_store(),
            gateway: crate::gateway::gateway_handle(),
        }
    }

    async fn authed_token(state: &AppState, seed: u8) -> (String, String) {
        let signing_key = ed25519_dalek::SigningKey::from_bytes(&[seed; 32]);
        let pubkey = bs58::encode(signing_key.verifying_key().as_bytes()).into_string();

        let body = serde_json::json!({ "pubkey": pubkey });
        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/auth/challenge")
            .header("content-type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();
        let resp = app(state.clone()).oneshot(req).await.unwrap();
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let cr: shared::auth::ChallengeResponse = serde_json::from_slice(&bytes).unwrap();

        let sig = BASE64.encode(signing_key.sign(cr.challenge.as_bytes()).to_bytes());
        let vbody = serde_json::json!({ "pubkey": pubkey, "signature": sig });
        let req2 = Request::builder()
            .method("POST")
            .uri("/api/v1/auth/verify")
            .header("content-type", "application/json")
            .body(Body::from(vbody.to_string()))
            .unwrap();
        let resp2 = app(state.clone()).oneshot(req2).await.unwrap();
        let bytes2 = resp2.into_body().collect().await.unwrap().to_bytes();
        let vr: shared::auth::VerifyResponse = serde_json::from_slice(&bytes2).unwrap();
        (vr.token, pubkey)
    }

    #[tokio::test]
    async fn role_create_and_list() {
        let state = test_state().await;
        let (token, _) = authed_token(&state, 31).await;

        let create_req = Request::builder()
            .method("POST")
            .uri("/api/v1/roles")
            .header("authorization", format!("Bearer {token}"))
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"name":"mods","permissions":3,"position":100}"#,
            ))
            .unwrap();
        let create_resp = app(state.clone()).oneshot(create_req).await.unwrap();
        assert_eq!(create_resp.status(), StatusCode::OK);
        let create_body = create_resp.into_body().collect().await.unwrap().to_bytes();
        let created: RoleResponse = serde_json::from_slice(&create_body).unwrap();
        assert_eq!(created.name, "mods");

        let list_req = Request::builder()
            .method("GET")
            .uri("/api/v1/roles")
            .header("authorization", format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap();
        let list_resp = app(state.clone()).oneshot(list_req).await.unwrap();
        assert_eq!(list_resp.status(), StatusCode::OK);
        let list_body = list_resp.into_body().collect().await.unwrap().to_bytes();
        let listed: ListRolesResponse = serde_json::from_slice(&list_body).unwrap();
        assert!(listed.roles.iter().any(|r| r.id == created.id));
    }

    #[tokio::test]
    async fn user_without_manage_roles_cannot_create_role() {
        let state = test_state().await;
        let (_admin, _) = authed_token(&state, 41).await;
        let (user_token, _) = authed_token(&state, 42).await;

        let create_req = Request::builder()
            .method("POST")
            .uri("/api/v1/roles")
            .header("authorization", format!("Bearer {user_token}"))
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"name":"mods","permissions":3,"position":100}"#,
            ))
            .unwrap();
        let create_resp = app(state).oneshot(create_req).await.unwrap();
        assert_eq!(create_resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn unauthenticated_roles_requests_return_401() {
        let state = test_state().await;
        let req = Request::builder()
            .method("GET")
            .uri("/api/v1/roles")
            .body(Body::empty())
            .unwrap();
        let resp = app(state).oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn builtin_roles_cannot_be_deleted() {
        let state = test_state().await;
        let (token, _) = authed_token(&state, 51).await;

        for role_id in ["everyone", "admin"] {
            let req = Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/roles/{role_id}"))
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap();
            let resp = app(state.clone()).oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        }
    }

    #[tokio::test]
    async fn channel_override_can_deny_sending_messages() {
        let state = test_state().await;
        let (admin_token, _) = authed_token(&state, 61).await;
        let (member_token, _) = authed_token(&state, 62).await;

        let channel_id = state
            .storage
            .create_channel("general", None, 0)
            .await
            .unwrap()
            .id;

        let override_req = Request::builder()
            .method("PUT")
            .uri(format!("/api/v1/channels/{channel_id}/overrides/everyone"))
            .header("authorization", format!("Bearer {admin_token}"))
            .header("content-type", "application/json")
            .body(Body::from(r#"{"allow":0,"deny":1}"#))
            .unwrap();
        let override_resp = app(state.clone()).oneshot(override_req).await.unwrap();
        assert_eq!(override_resp.status(), StatusCode::OK);

        let send_req = Request::builder()
            .method("POST")
            .uri(format!("/api/v1/channels/{channel_id}/messages"))
            .header("authorization", format!("Bearer {member_token}"))
            .header("content-type", "application/json")
            .body(Body::from(r#"{"content":"blocked"}"#))
            .unwrap();
        let send_resp = app(state).oneshot(send_req).await.unwrap();
        assert_eq!(send_resp.status(), StatusCode::FORBIDDEN);
    }
}
