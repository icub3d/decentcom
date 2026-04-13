mod handlers;
pub mod models;
pub mod validation;

use axum::routing::{get, patch, post};
use axum::Router;

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/channels", get(handlers::list_channels).post(handlers::create_channel))
        .route(
            "/channels/:channel_id",
            get(handlers::get_channel)
                .patch(handlers::update_channel)
                .delete(handlers::delete_channel),
        )
        .route("/categories", post(handlers::create_category))
        .route(
            "/categories/:category_id",
            patch(handlers::update_category).delete(handlers::delete_category),
        )
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    use crate::channels::models::{CategoryResponse, ChannelResponse, ListChannelsResponse};
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

    async fn authed_token(state: &AppState) -> String {
        use base64::engine::general_purpose::STANDARD as BASE64;
        use base64::Engine;
        use ed25519_dalek::Signer;

        let signing_key = ed25519_dalek::SigningKey::from_bytes(&[42u8; 32]);
        let pubkey = bs58::encode(signing_key.verifying_key().as_bytes()).into_string();

        // Obtain challenge
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

        // Sign and verify
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
    async fn channel_create_requires_auth() {
        let state = test_state().await;
        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/channels")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"name":"general"}"#))
            .unwrap();
        let resp = app(state).oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn channel_full_lifecycle() {
        let state = test_state().await;
        let token = authed_token(&state).await;

        // Create a category
        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/categories")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {token}"))
            .body(Body::from(r#"{"name":"Text Channels","position":0}"#))
            .unwrap();
        let resp = app(state.clone()).oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let cat: CategoryResponse = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(cat.name, "Text Channels");

        // Create a channel in that category
        let body = serde_json::json!({
            "name": "general",
            "category_id": cat.id,
            "position": 0
        });
        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/channels")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {token}"))
            .body(Body::from(body.to_string()))
            .unwrap();
        let resp = app(state.clone()).oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let ch: ChannelResponse = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(ch.name, "general");
        assert_eq!(ch.category_id.as_deref(), Some(cat.id.as_str()));

        // List channels and categories
        let req = Request::builder()
            .method("GET")
            .uri("/api/v1/channels")
            .header("authorization", format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap();
        let resp = app(state.clone()).oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let list: ListChannelsResponse = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(list.channels.len(), 1);
        assert_eq!(list.categories.len(), 1);

        // Update the channel
        let req = Request::builder()
            .method("PATCH")
            .uri(format!("/api/v1/channels/{}", ch.id))
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {token}"))
            .body(Body::from(r#"{"name":"random"}"#))
            .unwrap();
        let resp = app(state.clone()).oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let updated: ChannelResponse = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(updated.name, "random");

        // Delete the channel
        let req = Request::builder()
            .method("DELETE")
            .uri(format!("/api/v1/channels/{}", ch.id))
            .header("authorization", format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap();
        let resp = app(state.clone()).oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NO_CONTENT);

        // 404 on get after delete
        let req = Request::builder()
            .method("GET")
            .uri(format!("/api/v1/channels/{}", ch.id))
            .header("authorization", format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap();
        let resp = app(state.clone()).oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn invalid_channel_name_returns_400() {
        let state = test_state().await;
        let token = authed_token(&state).await;

        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/channels")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {token}"))
            .body(Body::from(r#"{"name":"Has Spaces"}"#))
            .unwrap();
        let resp = app(state).oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }
}
