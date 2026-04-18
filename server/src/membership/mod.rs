mod handlers;
pub mod models;

use axum::routing::{delete, get, post};
use axum::Router;

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/members/join", post(handlers::join_member))
        .route("/members/me", delete(handlers::leave_member))
        .route("/members", get(handlers::list_members))
        .route("/members/:pubkey", get(handlers::get_member))
        .route("/members/:pubkey/kick", post(handlers::kick_member))
        .route("/members/:pubkey/ban", post(handlers::ban_member))
        .route("/bans", get(handlers::list_bans))
        .route("/bans/:pubkey", delete(handlers::unban_member))
        .route(
            "/allowlist",
            get(handlers::list_allowlist).post(handlers::add_allowlist),
        )
        .route("/allowlist/:pubkey", delete(handlers::remove_allowlist))
        .route("/admin/bots/pending", get(handlers::list_pending_bots))
        .route("/admin/bots/:pubkey/approve", post(handlers::approve_bot))
        .route("/admin/bots/:pubkey/revoke", post(handlers::revoke_bot))
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
    use crate::storage::{DynStorage, SqliteStorage};
    use crate::{app, AppState};

    async fn test_state(mode: MembershipMode) -> AppState {
        let mut cfg = ServerConfig::default();
        cfg.membership.mode = mode;
        let storage: DynStorage = Arc::new(SqliteStorage::in_memory().await.unwrap());
        AppState {
            config: Arc::new(cfg),
            storage,
            challenge_store: crate::auth::challenge_store(),
            gateway: crate::gateway::gateway_handle(),
        }
    }

    async fn authed_session(state: &AppState, seed: u8) -> (String, String) {
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

        (verify.token, pubkey)
    }

    #[tokio::test]
    async fn join_open_mode_succeeds() {
        let state = test_state(MembershipMode::Open).await;
        let (token, _pubkey) = authed_session(&state, 91).await;

        let session = state.storage.get_session(&token).await.unwrap().unwrap();
        let user_id = session.user_id;

        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/members/join")
            .header("authorization", format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap();
        let resp = app(state.clone()).oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // Verify the first member is an admin
        let is_admin = state.storage.user_has_role(&user_id, "admin").await.unwrap();
        assert!(is_admin, "first member should be granted admin role");
    }

    #[tokio::test]
    async fn join_invite_only_without_invite_is_rejected() {
        let state = test_state(MembershipMode::InviteOnly).await;
        let (_owner_token, _) = authed_session(&state, 92).await;
        let (token, _) = authed_session(&state, 93).await;

        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/members/join")
            .header("authorization", format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap();
        let resp = app(state).oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn allowlist_mode_requires_entry() {
        let state = test_state(MembershipMode::Allowlist).await;
        let (admin_token, _) = authed_session(&state, 93).await;
        let (member_token, member_pubkey) = authed_session(&state, 94).await;

        let join_req = Request::builder()
            .method("POST")
            .uri("/api/v1/members/join")
            .header("authorization", format!("Bearer {member_token}"))
            .body(Body::empty())
            .unwrap();
        let join_resp = app(state.clone()).oneshot(join_req).await.unwrap();
        assert_eq!(join_resp.status(), StatusCode::FORBIDDEN);

        let allow_req = Request::builder()
            .method("POST")
            .uri("/api/v1/allowlist")
            .header("authorization", format!("Bearer {admin_token}"))
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({ "pubkey": member_pubkey }).to_string(),
            ))
            .unwrap();
        let allow_resp = app(state.clone()).oneshot(allow_req).await.unwrap();
        assert_eq!(allow_resp.status(), StatusCode::OK);

        let join_req_2 = Request::builder()
            .method("POST")
            .uri("/api/v1/members/join")
            .header("authorization", format!("Bearer {member_token}"))
            .body(Body::empty())
            .unwrap();
        let join_resp_2 = app(state).oneshot(join_req_2).await.unwrap();
        assert_eq!(join_resp_2.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn kick_and_ban_require_permissions() {
        let state = test_state(MembershipMode::Open).await;
        let (_admin_token, _) = authed_session(&state, 95).await;
        let (member_token, member_pubkey) = authed_session(&state, 96).await;

        let kick_req = Request::builder()
            .method("POST")
            .uri(format!("/api/v1/members/{member_pubkey}/kick"))
            .header("authorization", format!("Bearer {member_token}"))
            .body(Body::empty())
            .unwrap();
        let kick_resp = app(state.clone()).oneshot(kick_req).await.unwrap();
        assert_eq!(kick_resp.status(), StatusCode::FORBIDDEN);

        let ban_req = Request::builder()
            .method("POST")
            .uri(format!("/api/v1/members/{member_pubkey}/ban"))
            .header("authorization", format!("Bearer {member_token}"))
            .header("content-type", "application/json")
            .body(Body::from(r#"{"reason":"test"}"#))
            .unwrap();
        let ban_resp = app(state).oneshot(ban_req).await.unwrap();
        assert_eq!(ban_resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn list_members_returns_entries() {
        let state = test_state(MembershipMode::Open).await;
        let (admin_token, _) = authed_session(&state, 97).await;
        let (_member_token, _member_pubkey) = authed_session(&state, 98).await;

        let req = Request::builder()
            .method("GET")
            .uri("/api/v1/members")
            .header("authorization", format!("Bearer {admin_token}"))
            .body(Body::empty())
            .unwrap();
        let resp = app(state).oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(json["members"].as_array().unwrap().len() >= 2);
    }

    #[tokio::test]
    async fn kick_removes_membership_and_allows_rejoin() {
        let state = test_state(MembershipMode::Open).await;
        let (admin_token, _) = authed_session(&state, 99).await;
        let (member_token, member_pubkey) = authed_session(&state, 100).await;

        // Join
        let join_req = Request::builder()
            .method("POST")
            .uri("/api/v1/members/join")
            .header("authorization", format!("Bearer {member_token}"))
            .body(Body::empty())
            .unwrap();
        let _ = app(state.clone()).oneshot(join_req).await.unwrap();

        // Kick
        let kick_req = Request::builder()
            .method("POST")
            .uri(format!("/api/v1/members/{member_pubkey}/kick"))
            .header("authorization", format!("Bearer {admin_token}"))
            .body(Body::empty())
            .unwrap();
        let kick_resp = app(state.clone()).oneshot(kick_req).await.unwrap();
        assert_eq!(kick_resp.status(), StatusCode::NO_CONTENT);

        // Check not member
        let check_req = Request::builder()
            .method("GET")
            .uri(format!("/api/v1/members/{member_pubkey}"))
            .header("authorization", format!("Bearer {admin_token}"))
            .body(Body::empty())
            .unwrap();
        let check_resp = app(state.clone()).oneshot(check_req).await.unwrap();
        assert_eq!(check_resp.status(), StatusCode::NOT_FOUND);

        // Rejoin
        let rejoin_req = Request::builder()
            .method("POST")
            .uri("/api/v1/members/join")
            .header("authorization", format!("Bearer {member_token}"))
            .body(Body::empty())
            .unwrap();
        let rejoin_resp = app(state).oneshot(rejoin_req).await.unwrap();
        assert_eq!(rejoin_resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn ban_removes_membership_and_blocks_rejoin_until_unbanned() {
        let state = test_state(MembershipMode::Open).await;
        let (admin_token, _) = authed_session(&state, 101).await;
        let (member_token, member_pubkey) = authed_session(&state, 102).await;

        // Join
        let join_req = Request::builder()
            .method("POST")
            .uri("/api/v1/members/join")
            .header("authorization", format!("Bearer {member_token}"))
            .body(Body::empty())
            .unwrap();
        let _ = app(state.clone()).oneshot(join_req).await.unwrap();

        // Ban
        let ban_req = Request::builder()
            .method("POST")
            .uri(format!("/api/v1/members/{member_pubkey}/ban"))
            .header("authorization", format!("Bearer {admin_token}"))
            .header("content-type", "application/json")
            .body(Body::from(r#"{"reason":"bye"}"#))
            .unwrap();
        let ban_resp = app(state.clone()).oneshot(ban_req).await.unwrap();
        assert_eq!(ban_resp.status(), StatusCode::OK);

        // Check rejoin blocked
        let rejoin_req = Request::builder()
            .method("POST")
            .uri("/api/v1/members/join")
            .header("authorization", format!("Bearer {member_token}"))
            .body(Body::empty())
            .unwrap();
        let rejoin_resp = app(state.clone()).oneshot(rejoin_req).await.unwrap();
        assert_eq!(rejoin_resp.status(), StatusCode::FORBIDDEN);

        // Unban
        let unban_req = Request::builder()
            .method("DELETE")
            .uri(format!("/api/v1/bans/{member_pubkey}"))
            .header("authorization", format!("Bearer {admin_token}"))
            .body(Body::empty())
            .unwrap();
        let unban_resp = app(state.clone()).oneshot(unban_req).await.unwrap();
        assert_eq!(unban_resp.status(), StatusCode::NO_CONTENT);

        // Rejoin succeeds
        let rejoin_req_2 = Request::builder()
            .method("POST")
            .uri("/api/v1/members/join")
            .header("authorization", format!("Bearer {member_token}"))
            .body(Body::empty())
            .unwrap();
        let rejoin_resp_2 = app(state).oneshot(rejoin_req_2).await.unwrap();
        assert_eq!(rejoin_resp_2.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn hierarchy_blocks_kick_of_equal_role() {
        let state = test_state(MembershipMode::Open).await;
        
        // User 1: Owner (admin)
        let (_owner_token, _) = authed_session(&state, 103).await;
        // User 2: Member (everyone)
        let (user2_token, _) = authed_session(&state, 104).await;
        // User 3: Member (everyone)
        let (user3_token, user3_pubkey) = authed_session(&state, 105).await;

        // Both join
        let join_req_2 = Request::builder()
            .method("POST")
            .uri("/api/v1/members/join")
            .header("authorization", format!("Bearer {user2_token}"))
            .body(Body::empty())
            .unwrap();
        let _ = app(state.clone()).oneshot(join_req_2).await.unwrap();

        let join_req_3 = Request::builder()
            .method("POST")
            .uri("/api/v1/members/join")
            .header("authorization", format!("Bearer {user3_token}"))
            .body(Body::empty())
            .unwrap();
        let _ = app(state.clone()).oneshot(join_req_3).await.unwrap();

        // Grant KICK_MEMBERS to @everyone so User 2 can attempt the kick
        state.storage.update_role("everyone", None, None, Some(crate::permissions::KICK_MEMBERS), None).await.unwrap();

        // User 2 tries to kick User 3. Both have only @everyone (position 0).
        let kick_req = Request::builder()
            .method("POST")
            .uri(format!("/api/v1/members/{user3_pubkey}/kick"))
            .header("authorization", format!("Bearer {user2_token}"))
            .body(Body::empty())
            .unwrap();
        let kick_resp = app(state).oneshot(kick_req).await.unwrap();
        assert_eq!(kick_resp.status(), StatusCode::FORBIDDEN);
        let body = kick_resp.into_body().collect().await.unwrap().to_bytes();
        assert!(String::from_utf8_lossy(&body).contains("equal or higher role"));
    }
}
