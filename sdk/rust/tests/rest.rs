//! Integration tests for the REST client using a mocked HTTP server.
//!
//! These tests exercise the full request pipeline — URL construction, auth
//! header injection, JSON encoding/decoding, and response handling — against
//! `wiremock` stubs.

use decentcom_sdk::rest::models::{
    Channel, CreateChannelRequest, CreateMessageRequest, ListChannelsResponse, MessagePage,
};
use decentcom_sdk::{Client, Identity};
use serde_json::json;
use wiremock::matchers::{body_json, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn make_client(server: &MockServer) -> Client {
    Client::new(server.uri()).expect("client")
}

#[tokio::test]
async fn authenticate_performs_challenge_response() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/auth/challenge"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "challenge": "sign-me",
            "expires_at": "2026-04-18T00:00:00Z",
        })))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/api/v1/auth/verify"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "token": "session-token-xyz",
            "user_id": "u1",
            "pubkey": "pk1",
            "expires_at": "2026-05-18T00:00:00Z",
        })))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let (identity, _mnemonic) = Identity::generate().expect("generate");
    let verified = client.authenticate(&identity).await.expect("authenticate");
    assert_eq!(verified.token, "session-token-xyz");
    assert_eq!(client.session_token().as_deref(), Some("session-token-xyz"));
    assert_eq!(client.user_id().as_deref(), Some("u1"));
}

#[tokio::test]
async fn list_channels_sends_bearer_and_parses_response() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/channels"))
        .and(header("authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "channels": [
                {
                    "id": "c1",
                    "name": "general",
                    "topic": null,
                    "category": null,
                    "position": 0,
                    "created_at": "2026-04-18T00:00:00Z",
                    "updated_at": "2026-04-18T00:00:00Z",
                    "type": "text"
                }
            ]
        })))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    client.set_session(Some("test-token".into()), Some("u1".into()));
    let channels = client.channels().list().await.expect("list");
    assert_eq!(channels.len(), 1);
    assert_eq!(channels[0].name, "general");
}

#[tokio::test]
async fn create_channel_posts_json_body() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/channels"))
        .and(header("authorization", "Bearer t"))
        .and(body_json(json!({ "name": "chat", "category": "General" })))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "id": "c2",
            "name": "chat",
            "topic": null,
            "category": "General",
            "position": 1,
            "created_at": "2026-04-18T00:00:00Z",
            "updated_at": "2026-04-18T00:00:00Z",
            "type": "text"
        })))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    client.set_session(Some("t".into()), Some("u1".into()));
    let req = CreateChannelRequest {
        name: "chat".into(),
        category: Some("General".into()),
        position: None,
    };
    let channel = client.channels().create(&req).await.expect("create");
    assert_eq!(channel.id, "c2");
    assert_eq!(channel.category.as_deref(), Some("General"));
}

#[tokio::test]
async fn list_messages_forwards_query_params() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/channels/c1/messages"))
        .and(query_param("limit", "10"))
        .and(query_param("before", "msg-5"))
        .and(header("authorization", "Bearer t"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "messages": [],
            "has_more": false
        })))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    client.set_session(Some("t".into()), Some("u1".into()));
    let query = decentcom_sdk::rest::models::ListMessagesQuery {
        before: Some("msg-5".into()),
        after: None,
        limit: Some(10),
    };
    let page: MessagePage = client
        .messages()
        .list("c1", &query)
        .await
        .expect("list messages");
    assert!(page.messages.is_empty());
    assert!(!page.has_more);
}

#[tokio::test]
async fn send_message_posts_body_with_empty_attachments() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/channels/c1/messages"))
        .and(body_json(json!({ "content": "hi" })))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "id": "m1",
            "channel_id": "c1",
            "author_id": "u1",
            "content": "hi",
            "created_at": "2026-04-18T00:00:00Z",
            "edited_at": null,
            "deleted": false,
            "attachments": [],
            "reactions": [],
            "thread": null
        })))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    client.set_session(Some("t".into()), Some("u1".into()));
    let msg = client
        .messages()
        .send_text("c1", "hi")
        .await
        .expect("send text");
    assert_eq!(msg.id, "m1");
    assert_eq!(msg.content, "hi");
}

#[tokio::test]
async fn api_error_returns_parsed_message() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/channels"))
        .respond_with(ResponseTemplate::new(403).set_body_json(json!({
            "error": "missing read_messages permission"
        })))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    client.set_session(Some("t".into()), Some("u1".into()));
    let err = client.channels().list().await.expect_err("should error");
    match err {
        decentcom_sdk::Error::Api { status, message } => {
            assert_eq!(status, 403);
            assert_eq!(message, "missing read_messages permission");
        }
        other => panic!("expected Api error, got {other:?}"),
    }
}

#[tokio::test]
async fn unauthenticated_calls_fail_without_token() {
    let server = MockServer::start().await;
    let client = make_client(&server).await;
    let err = client.channels().list().await.expect_err("should error");
    assert!(matches!(err, decentcom_sdk::Error::Unauthenticated));
}

#[tokio::test]
async fn invite_preview_uses_anonymous_request() {
    let server = MockServer::start().await;

    // Note: no authorization header on this mock — we assert that preview
    // works even when the client has no session token.
    Mock::given(method("GET"))
        .and(path("/api/v1/invites/ABC123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "code": "ABC123",
            "server_name": "Test",
            "server_icon": null,
            "member_count": 42,
            "expires_at": null
        })))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let preview = client.invites().preview("ABC123").await.expect("preview");
    assert_eq!(preview.code, "ABC123");
    assert_eq!(preview.member_count, 42);
}

#[tokio::test]
async fn media_url_builds_absolute_path() {
    let server = MockServer::start().await;
    let client = make_client(&server).await;
    let url = client.attachments().media_url("abc123");
    assert_eq!(url, format!("{}/api/v1/media/abc123", server.uri()));
}

#[tokio::test]
async fn delete_returns_no_content() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/api/v1/channels/c1"))
        .and(header("authorization", "Bearer t"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    client.set_session(Some("t".into()), Some("u1".into()));
    client.channels().delete("c1").await.expect("delete");
}

// Dummy use to satisfy imports in a shared way.
#[allow(dead_code)]
fn _types_are_public() {
    let _: Option<Channel> = None;
    let _: Option<ListChannelsResponse> = None;
    let _: Option<CreateMessageRequest> = None;
}
