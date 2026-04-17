mod handlers;
mod models;

use axum::routing::{get, post, put};
use axum::Router;

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/channels/:channel_id/messages/:message_id/threads", post(handlers::create_thread))
        .route("/threads/:thread_id", get(handlers::get_thread))
        .route("/threads/:thread_id/messages", get(handlers::list_thread_messages).post(handlers::create_thread_message))
        .route("/threads/:thread_id/follow", put(handlers::follow_thread).delete(handlers::unfollow_thread))
        .route("/threads/:thread_id/read", put(handlers::mark_thread_read))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    use crate::config::ServerConfig;
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

    #[tokio::test]
    async fn create_thread_success() {
        let state = test_state().await;
        let token = authed_token(&state, 30).await;
        let uid = {
            let signing_key = ed25519_dalek::SigningKey::from_bytes(&[30u8; 32]);
            let pubkey = bs58::encode(signing_key.verifying_key().as_bytes()).into_string();
            state.storage.get_user_by_pubkey(&pubkey).await.unwrap().unwrap().id
        };
        
        let channel = state.storage.create_channel("test", None, 0).await.unwrap();
        let msg = state.storage.create_message(&channel.id, &uid, "root", None).await.unwrap();

        let req = Request::builder()
            .method("POST")
            .uri(format!("/api/v1/channels/{}/messages/{}/threads", channel.id, msg.id))
            .header("authorization", format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap();

        let resp = app(state.clone()).oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let thread: crate::threads::models::CreateThreadResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(thread.parent_message_id, msg.id);
    }

    #[tokio::test]
    async fn post_to_thread_success() {
        let state = test_state().await;
        let token = authed_token(&state, 31).await;
        let uid = {
            let signing_key = ed25519_dalek::SigningKey::from_bytes(&[31u8; 32]);
            let pubkey = bs58::encode(signing_key.verifying_key().as_bytes()).into_string();
            state.storage.get_user_by_pubkey(&pubkey).await.unwrap().unwrap().id
        };

        let channel = state.storage.create_channel("test", None, 0).await.unwrap();
        let msg = state.storage.create_message(&channel.id, &uid, "root", None).await.unwrap();
        let thread = state.storage.create_thread(&channel.id, &msg.id, &uid).await.unwrap();

        let req = Request::builder()
            .method("POST")
            .uri(format!("/api/v1/threads/{}/messages", thread.id))
            .header("authorization", format!("Bearer {token}"))
            .header("content-type", "application/json")
            .body(Body::from(r#"{"content":"reply"}"#))
            .unwrap();

        let resp = app(state.clone()).oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // Verify thread updated
        let t_fetched = state.storage.get_thread(&thread.id).await.unwrap().unwrap();
        assert_eq!(t_fetched.reply_count, 1);
    }
}
