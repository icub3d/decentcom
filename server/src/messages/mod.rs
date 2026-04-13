mod handlers;
pub mod models;

use axum::routing::{get, post};
use axum::Router;

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/channels/:channel_id/messages",
            post(handlers::create_message).get(handlers::list_messages),
        )
        .route(
            "/channels/:channel_id/messages/:message_id",
            get(handlers::get_message)
                .patch(handlers::update_message)
                .delete(handlers::delete_message),
        )
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    use crate::config::ServerConfig;
    use crate::messages::models::{MessagePage, MessageResponse};
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

    async fn authed_token(state: &AppState, seed: u8) -> String {
        use base64::engine::general_purpose::STANDARD as BASE64;
        use base64::Engine;
        use ed25519_dalek::Signer;

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
        vr.token
    }

    async fn create_channel_id(state: &AppState) -> String {
        state
            .storage
            .create_channel("general", None, 0)
            .await
            .unwrap()
            .id
    }

    #[tokio::test]
    async fn send_fetch_and_pagination() {
        let state = test_state().await;
        let token = authed_token(&state, 11).await;
        let channel_id = create_channel_id(&state).await;

        for n in 0..3 {
            let req = Request::builder()
                .method("POST")
                .uri(format!("/api/v1/channels/{channel_id}/messages"))
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::from(format!("{{\"content\":\"msg {n}\"}}")))
                .unwrap();
            let resp = app(state.clone()).oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::OK);
        }

        let req = Request::builder()
            .method("GET")
            .uri(format!("/api/v1/channels/{channel_id}/messages?limit=2"))
            .header("authorization", format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap();
        let resp = app(state.clone()).oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let page: MessagePage = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(page.messages.len(), 2);
        assert!(page.has_more);

        let before = page.messages.last().unwrap().id.clone();
        let req = Request::builder()
            .method("GET")
            .uri(format!(
                "/api/v1/channels/{channel_id}/messages?before={before}&limit=2"
            ))
            .header("authorization", format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap();
        let resp = app(state.clone()).oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let page2: MessagePage = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(page2.messages.len(), 1);
        assert!(!page2.has_more);
    }

    #[tokio::test]
    async fn edit_and_delete_authorization() {
        let state = test_state().await;
        let owner_token = authed_token(&state, 12).await;
        let other_token = authed_token(&state, 13).await;
        let channel_id = create_channel_id(&state).await;

        let create_req = Request::builder()
            .method("POST")
            .uri(format!("/api/v1/channels/{channel_id}/messages"))
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {owner_token}"))
            .body(Body::from(r#"{"content":"hello"}"#))
            .unwrap();
        let create_resp = app(state.clone()).oneshot(create_req).await.unwrap();
        let create_bytes = create_resp.into_body().collect().await.unwrap().to_bytes();
        let created: MessageResponse = serde_json::from_slice(&create_bytes).unwrap();

        let unauthorized_edit = Request::builder()
            .method("PATCH")
            .uri(format!(
                "/api/v1/channels/{channel_id}/messages/{}",
                created.id
            ))
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {other_token}"))
            .body(Body::from(r#"{"content":"hijack"}"#))
            .unwrap();
        let resp = app(state.clone()).oneshot(unauthorized_edit).await.unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);

        let edit_req = Request::builder()
            .method("PATCH")
            .uri(format!(
                "/api/v1/channels/{channel_id}/messages/{}",
                created.id
            ))
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {owner_token}"))
            .body(Body::from(r#"{"content":"edited"}"#))
            .unwrap();
        let edit_resp = app(state.clone()).oneshot(edit_req).await.unwrap();
        assert_eq!(edit_resp.status(), StatusCode::OK);
        let edit_bytes = edit_resp.into_body().collect().await.unwrap().to_bytes();
        let edited: MessageResponse = serde_json::from_slice(&edit_bytes).unwrap();
        assert_eq!(edited.content, "edited");
        assert!(edited.edited_at.is_some());

        let unauthorized_delete = Request::builder()
            .method("DELETE")
            .uri(format!(
                "/api/v1/channels/{channel_id}/messages/{}",
                created.id
            ))
            .header("authorization", format!("Bearer {other_token}"))
            .body(Body::empty())
            .unwrap();
        let resp = app(state.clone()).oneshot(unauthorized_delete).await.unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);

        let delete_req = Request::builder()
            .method("DELETE")
            .uri(format!(
                "/api/v1/channels/{channel_id}/messages/{}",
                created.id
            ))
            .header("authorization", format!("Bearer {owner_token}"))
            .body(Body::empty())
            .unwrap();
        let resp = app(state.clone()).oneshot(delete_req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NO_CONTENT);

        let get_req = Request::builder()
            .method("GET")
            .uri(format!(
                "/api/v1/channels/{channel_id}/messages/{}",
                created.id
            ))
            .header("authorization", format!("Bearer {owner_token}"))
            .body(Body::empty())
            .unwrap();
        let resp = app(state.clone()).oneshot(get_req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let deleted_msg: MessageResponse = serde_json::from_slice(&bytes).unwrap();
        assert!(deleted_msg.deleted);
    }

    #[tokio::test]
    async fn validation_and_not_found_cases() {
        let state = test_state().await;
        let token = authed_token(&state, 14).await;

        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/channels/does-not-exist/messages")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {token}"))
            .body(Body::from(r#"{"content":"hello"}"#))
            .unwrap();
        let resp = app(state.clone()).oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);

        let channel_id = create_channel_id(&state).await;

        let empty_req = Request::builder()
            .method("POST")
            .uri(format!("/api/v1/channels/{channel_id}/messages"))
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {token}"))
            .body(Body::from(r#"{"content":"   "}"#))
            .unwrap();
        let resp = app(state.clone()).oneshot(empty_req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);

        let long = "a".repeat(4001);
        let long_req = Request::builder()
            .method("POST")
            .uri(format!("/api/v1/channels/{channel_id}/messages"))
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {token}"))
            .body(Body::from(format!("{{\"content\":\"{long}\"}}")))
            .unwrap();
        let resp = app(state.clone()).oneshot(long_req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);

        let empty_list_req = Request::builder()
            .method("GET")
            .uri(format!("/api/v1/channels/{channel_id}/messages"))
            .header("authorization", format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap();
        let resp = app(state.clone()).oneshot(empty_list_req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let page: MessagePage = serde_json::from_slice(&bytes).unwrap();
        assert!(page.messages.is_empty());
        assert!(!page.has_more);
    }
}
