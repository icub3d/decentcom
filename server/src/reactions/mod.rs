mod handlers;

use axum::routing::{delete, put};
use axum::Router;

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/channels/:channel_id/messages/:message_id/reactions/:emoji",
            put(handlers::put_reaction).get(handlers::list_reactors),
        )
        .route(
            "/channels/:channel_id/messages/:message_id/reactions",
            delete(handlers::delete_own_reaction),
        )
        .route(
            "/channels/:channel_id/messages/:message_id/reactions/users/:user_id",
            delete(handlers::delete_user_reaction),
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

    async fn test_state_reactions_disabled() -> AppState {
        let storage: DynStorage = Arc::new(SqliteStorage::in_memory().await.unwrap());
        let mut cfg = ServerConfig::default();
        cfg.features.emoji_reactions = false;
        AppState {
            config: Arc::new(cfg),
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

    async fn create_channel_and_message(state: &AppState, token: &str) -> (String, String) {
        let channel = state
            .storage
            .create_channel("test", None, 0)
            .await
            .unwrap();

        let req = Request::builder()
            .method("POST")
            .uri(format!("/api/v1/channels/{}/messages", channel.id))
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {token}"))
            .body(Body::from(r#"{"content":"hello"}"#))
            .unwrap();
        let resp = app(state.clone()).oneshot(req).await.unwrap();
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let msg: MessageResponse = serde_json::from_slice(&bytes).unwrap();
        (channel.id, msg.id)
    }

    async fn put_reaction(
        state: &AppState,
        token: &str,
        channel_id: &str,
        message_id: &str,
        emoji: &str,
    ) -> StatusCode {
        let req = Request::builder()
            .method("PUT")
            .uri(format!(
                "/api/v1/channels/{channel_id}/messages/{message_id}/reactions/{emoji}"
            ))
            .header("authorization", format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap();
        app(state.clone()).oneshot(req).await.unwrap().status()
    }

    #[tokio::test]
    async fn add_reaction_appears_in_message_fetch() {
        let state = test_state().await;
        let token = authed_token(&state, 20).await;
        let (channel_id, message_id) = create_channel_and_message(&state, &token).await;

        let status = put_reaction(&state, &token, &channel_id, &message_id, "thumbsup").await;
        assert_eq!(status, StatusCode::OK);

        let req = Request::builder()
            .method("GET")
            .uri(format!("/api/v1/channels/{channel_id}/messages"))
            .header("authorization", format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap();
        let resp = app(state.clone()).oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let page: MessagePage = serde_json::from_slice(&bytes).unwrap();
        let msg = &page.messages[0];
        assert_eq!(msg.reactions.len(), 1);
        assert_eq!(msg.reactions[0].emoji, "thumbsup");
        assert_eq!(msg.reactions[0].count, 1);
        assert!(msg.reactions[0].me);
    }

    #[tokio::test]
    async fn toggle_same_emoji_removes_reaction() {
        let state = test_state().await;
        let token = authed_token(&state, 21).await;
        let (channel_id, message_id) = create_channel_and_message(&state, &token).await;

        put_reaction(&state, &token, &channel_id, &message_id, "thumbsup").await;
        // Toggle off
        let status = put_reaction(&state, &token, &channel_id, &message_id, "thumbsup").await;
        assert_eq!(status, StatusCode::OK);

        let req = Request::builder()
            .method("GET")
            .uri(format!("/api/v1/channels/{channel_id}/messages/{message_id}"))
            .header("authorization", format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap();
        let resp = app(state.clone()).oneshot(req).await.unwrap();
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let msg: MessageResponse = serde_json::from_slice(&bytes).unwrap();
        assert!(msg.reactions.is_empty());
    }

    #[tokio::test]
    async fn replace_reaction_with_different_emoji() {
        let state = test_state().await;
        let token = authed_token(&state, 22).await;
        let (channel_id, message_id) = create_channel_and_message(&state, &token).await;

        put_reaction(&state, &token, &channel_id, &message_id, "thumbsup").await;
        put_reaction(&state, &token, &channel_id, &message_id, "heart").await;

        let req = Request::builder()
            .method("GET")
            .uri(format!("/api/v1/channels/{channel_id}/messages/{message_id}"))
            .header("authorization", format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap();
        let resp = app(state.clone()).oneshot(req).await.unwrap();
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let msg: MessageResponse = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(msg.reactions.len(), 1);
        assert_eq!(msg.reactions[0].emoji, "heart");
    }

    #[tokio::test]
    async fn delete_own_reaction_removes_it() {
        let state = test_state().await;
        let token = authed_token(&state, 23).await;
        let (channel_id, message_id) = create_channel_and_message(&state, &token).await;

        put_reaction(&state, &token, &channel_id, &message_id, "thumbsup").await;

        let req = Request::builder()
            .method("DELETE")
            .uri(format!(
                "/api/v1/channels/{channel_id}/messages/{message_id}/reactions"
            ))
            .header("authorization", format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap();
        let resp = app(state.clone()).oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NO_CONTENT);

        let req = Request::builder()
            .method("GET")
            .uri(format!("/api/v1/channels/{channel_id}/messages/{message_id}"))
            .header("authorization", format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap();
        let resp = app(state.clone()).oneshot(req).await.unwrap();
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let msg: MessageResponse = serde_json::from_slice(&bytes).unwrap();
        assert!(msg.reactions.is_empty());
    }

    #[tokio::test]
    async fn reactions_disabled_returns_403() {
        let state = test_state_reactions_disabled().await;
        let token = authed_token(&state, 24).await;
        let (channel_id, message_id) = create_channel_and_message(&state, &token).await;

        let status = put_reaction(&state, &token, &channel_id, &message_id, "thumbsup").await;
        assert_eq!(status, StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn manage_messages_can_remove_other_user_reaction() {
        let state = test_state().await;
        let admin_token = authed_token(&state, 25).await;
        let user_token = authed_token(&state, 26).await;
        let (channel_id, message_id) = create_channel_and_message(&state, &admin_token).await;

        put_reaction(&state, &user_token, &channel_id, &message_id, "thumbsup").await;

        // Get user id from storage via pubkey
        let user_id = {
            let signing_key = ed25519_dalek::SigningKey::from_bytes(&[26u8; 32]);
            let pubkey = bs58::encode(signing_key.verifying_key().as_bytes()).into_string();
            state
                .storage
                .get_user_by_pubkey(&pubkey)
                .await
                .unwrap()
                .unwrap()
                .id
        };

        // Admin removes user's reaction
        let req = Request::builder()
            .method("DELETE")
            .uri(format!(
                "/api/v1/channels/{channel_id}/messages/{message_id}/reactions/users/{user_id}"
            ))
            .header("authorization", format!("Bearer {admin_token}"))
            .body(Body::empty())
            .unwrap();
        let resp = app(state.clone()).oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NO_CONTENT);

        let req = Request::builder()
            .method("GET")
            .uri(format!("/api/v1/channels/{channel_id}/messages/{message_id}"))
            .header("authorization", format!("Bearer {admin_token}"))
            .body(Body::empty())
            .unwrap();
        let resp = app(state.clone()).oneshot(req).await.unwrap();
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let msg: MessageResponse = serde_json::from_slice(&bytes).unwrap();
        assert!(msg.reactions.is_empty());
    }
}
