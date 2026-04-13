pub mod challenge;
mod handlers;
pub mod middleware;

use std::time::Duration;

use axum::routing::{get, post};
use axum::Router;

use crate::auth::challenge::ChallengeStore;
use crate::AppState;

pub fn challenge_store() -> challenge::SharedChallengeStore {
    std::sync::Arc::new(ChallengeStore::new(Duration::from_secs(5 * 60)))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/challenge", post(handlers::challenge))
        .route("/verify", post(handlers::verify))
        .route("/me", get(handlers::me))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use base64::engine::general_purpose::STANDARD as BASE64;
    use base64::Engine;
    use ed25519_dalek::{Signer, SigningKey};
    use http_body_util::BodyExt;
    use shared::auth::{AuthMeResponse, ChallengeResponse, VerifyResponse};
    use tower::ServiceExt;

    use crate::config::ServerConfig;
    use crate::storage::{DynStorage, SqliteStorage};
    use crate::{app, AppState};

    async fn test_state() -> AppState {
        let storage: DynStorage = Arc::new(SqliteStorage::in_memory().await.unwrap());
        AppState {
            config: Arc::new(ServerConfig::default()),
            storage,
            challenge_store: super::challenge_store(),
        }
    }

    fn signing_key() -> SigningKey {
        SigningKey::from_bytes(&[7u8; 32])
    }

    fn public_key_b58(key: &SigningKey) -> String {
        bs58::encode(key.verifying_key().as_bytes()).into_string()
    }

    async fn request_challenge(app: &axum::Router, pubkey: &str) -> (StatusCode, String) {
        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/auth/challenge")
            .header("content-type", "application/json")
            .body(Body::from(format!(r#"{{"pubkey":"{pubkey}"}}"#)))
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        let status = resp.status();
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        (status, String::from_utf8(body.to_vec()).unwrap())
    }

    #[tokio::test]
    async fn valid_ed25519_signature_is_accepted_by_verify_handler() {
        let state = test_state().await;
        let router = app(state);

        let key = signing_key();
        let pubkey = public_key_b58(&key);

        let (status, body) = request_challenge(&router, &pubkey).await;
        assert_eq!(status, StatusCode::OK);
        let challenge: ChallengeResponse = serde_json::from_str(&body).unwrap();

        let sig = key.sign(challenge.challenge.as_bytes());
        let sig_b64 = BASE64.encode(sig.to_bytes());

        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/auth/verify")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({"pubkey": pubkey, "signature": sig_b64}).to_string(),
            ))
            .unwrap();
        let resp = router.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn invalid_signature_is_rejected_with_401() {
        let state = test_state().await;
        let router = app(state);
        let key = signing_key();
        let pubkey = public_key_b58(&key);

        let (_, body) = request_challenge(&router, &pubkey).await;
        let challenge: ChallengeResponse = serde_json::from_str(&body).unwrap();
        let mut bad_sig = key.sign(challenge.challenge.as_bytes()).to_bytes();
        bad_sig[0] ^= 0x01;

        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/auth/verify")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({"pubkey": pubkey, "signature": BASE64.encode(bad_sig)}).to_string(),
            ))
            .unwrap();
        let resp = router.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn verify_with_unknown_challenge_returns_401() {
        let state = test_state().await;
        let router = app(state);
        let key = signing_key();
        let pubkey = public_key_b58(&key);
        let sig = key.sign(b"not-a-real-challenge");

        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/auth/verify")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({"pubkey": pubkey, "signature": BASE64.encode(sig.to_bytes())}).to_string(),
            ))
            .unwrap();
        let resp = router.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn first_time_pubkey_creates_user() {
        let state = test_state().await;
        let storage = state.storage.clone();
        let router = app(state);

        let key = signing_key();
        let pubkey = public_key_b58(&key);
        let (_, body) = request_challenge(&router, &pubkey).await;
        let challenge: ChallengeResponse = serde_json::from_str(&body).unwrap();
        let sig = key.sign(challenge.challenge.as_bytes());

        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/auth/verify")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({"pubkey": pubkey, "signature": BASE64.encode(sig.to_bytes())}).to_string(),
            ))
            .unwrap();
        let resp = router.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let users = storage.list_users().await.unwrap();
        assert_eq!(users.len(), 1);
    }

    #[tokio::test]
    async fn returning_pubkey_does_not_create_duplicate_user() {
        let state = test_state().await;
        let storage = state.storage.clone();
        let router = app(state);

        let key = signing_key();
        let pubkey = public_key_b58(&key);

        for _ in 0..2 {
            let (_, body) = request_challenge(&router, &pubkey).await;
            let challenge: ChallengeResponse = serde_json::from_str(&body).unwrap();
            let sig = key.sign(challenge.challenge.as_bytes());

            let req = Request::builder()
                .method("POST")
                .uri("/api/v1/auth/verify")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({"pubkey": pubkey, "signature": BASE64.encode(sig.to_bytes())})
                        .to_string(),
                ))
                .unwrap();
            let resp = router.clone().oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::OK);
        }

        let users = storage.list_users().await.unwrap();
        assert_eq!(users.len(), 1);
    }

    #[tokio::test]
    async fn auth_user_extractor_returns_401_for_missing_invalid_expired_token() {
        let mut cfg = ServerConfig::default();
        cfg.auth.session_ttl_seconds = 1;
        let storage: DynStorage = Arc::new(SqliteStorage::in_memory().await.unwrap());
        let state = AppState {
            config: Arc::new(cfg),
            storage,
            challenge_store: super::challenge_store(),
        };
        let router = app(state);

        let req = Request::builder()
            .method("GET")
            .uri("/api/v1/auth/me")
            .body(Body::empty())
            .unwrap();
        let resp = router.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

        let req = Request::builder()
            .method("GET")
            .uri("/api/v1/auth/me")
            .header("authorization", "Bearer nope")
            .body(Body::empty())
            .unwrap();
        let resp = router.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

        let key = signing_key();
        let pubkey = public_key_b58(&key);
        let (_, body) = request_challenge(&router, &pubkey).await;
        let challenge: ChallengeResponse = serde_json::from_str(&body).unwrap();
        let sig = key.sign(challenge.challenge.as_bytes());

        let verify_req = Request::builder()
            .method("POST")
            .uri("/api/v1/auth/verify")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({"pubkey": pubkey, "signature": BASE64.encode(sig.to_bytes())}).to_string(),
            ))
            .unwrap();
        let verify_resp = router.clone().oneshot(verify_req).await.unwrap();
        assert_eq!(verify_resp.status(), StatusCode::OK);
        let body = verify_resp.into_body().collect().await.unwrap().to_bytes();
        let verify: VerifyResponse = serde_json::from_slice(&body).unwrap();

        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        let req = Request::builder()
            .method("GET")
            .uri("/api/v1/auth/me")
            .header("authorization", format!("Bearer {}", verify.token))
            .body(Body::empty())
            .unwrap();
        let resp = router.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn auth_user_extractor_returns_user_for_valid_token_and_me_endpoint_works() {
        let state = test_state().await;
        let router = app(state);
        let key = signing_key();
        let pubkey = public_key_b58(&key);

        let (_, body) = request_challenge(&router, &pubkey).await;
        let challenge: ChallengeResponse = serde_json::from_str(&body).unwrap();
        let sig = key.sign(challenge.challenge.as_bytes());

        let verify_req = Request::builder()
            .method("POST")
            .uri("/api/v1/auth/verify")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({"pubkey": pubkey, "signature": BASE64.encode(sig.to_bytes())}).to_string(),
            ))
            .unwrap();
        let verify_resp = router.clone().oneshot(verify_req).await.unwrap();
        assert_eq!(verify_resp.status(), StatusCode::OK);
        let verify_body = verify_resp.into_body().collect().await.unwrap().to_bytes();
        let verify: VerifyResponse = serde_json::from_slice(&verify_body).unwrap();

        let me_req = Request::builder()
            .method("GET")
            .uri("/api/v1/auth/me")
            .header("authorization", format!("Bearer {}", verify.token))
            .body(Body::empty())
            .unwrap();
        let me_resp = router.oneshot(me_req).await.unwrap();
        assert_eq!(me_resp.status(), StatusCode::OK);
        let me_body = me_resp.into_body().collect().await.unwrap().to_bytes();
        let me: AuthMeResponse = serde_json::from_slice(&me_body).unwrap();
        assert_eq!(me.user_id, verify.user_id);
    }

    #[tokio::test]
    async fn integration_full_auth_flow_challenge_sign_verify_and_me() {
        let state = test_state().await;
        let router = app(state);
        let key = signing_key();
        let pubkey = public_key_b58(&key);

        let (challenge_status, challenge_body) = request_challenge(&router, &pubkey).await;
        assert_eq!(challenge_status, StatusCode::OK);
        let challenge: ChallengeResponse = serde_json::from_str(&challenge_body).unwrap();
        let _nonce = BASE64.decode(&challenge.challenge).unwrap();

        let sig = key.sign(challenge.challenge.as_bytes());
        let verify_req = Request::builder()
            .method("POST")
            .uri("/api/v1/auth/verify")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({"pubkey": pubkey, "signature": BASE64.encode(sig.to_bytes())}).to_string(),
            ))
            .unwrap();
        let verify_resp = router.clone().oneshot(verify_req).await.unwrap();
        assert_eq!(verify_resp.status(), StatusCode::OK);
        let verify_body = verify_resp.into_body().collect().await.unwrap().to_bytes();
        let verify: VerifyResponse = serde_json::from_slice(&verify_body).unwrap();

        let me_req = Request::builder()
            .method("GET")
            .uri("/api/v1/auth/me")
            .header("authorization", format!("Bearer {}", verify.token))
            .body(Body::empty())
            .unwrap();
        let me_resp = router.oneshot(me_req).await.unwrap();
        assert_eq!(me_resp.status(), StatusCode::OK);
    }
}
