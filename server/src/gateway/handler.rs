use axum::extract::ws::WebSocketUpgrade;
use axum::extract::{Query, State};
use axum::http::header::AUTHORIZATION;
use axum::http::HeaderMap;
use axum::response::IntoResponse;
use axum::Json;
use axum::{http::StatusCode, response::Response};
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;

use crate::gateway::connection;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct GatewayQuery {
    token: Option<String>,
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    error: String,
}

pub async fn ws_upgrade(
    State(state): State<AppState>,
    Query(query): Query<GatewayQuery>,
    headers: HeaderMap,
    ws: WebSocketUpgrade,
) -> Response {
    let Some(token) = extract_token(&headers, &query) else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(ErrorBody {
                error: "missing session token".to_string(),
            }),
        )
            .into_response();
    };

    ws.on_upgrade(move |socket| async move {
        let Some(user_id) = validate_session(&state, &token).await else {
            connection::reject_unauthorized(socket).await;
            return;
        };

        let connection_id = ulid::Ulid::new().to_string();
        connection::run(socket, state, connection_id, user_id).await;
    })
}

fn extract_token(headers: &HeaderMap, query: &GatewayQuery) -> Option<String> {
    if let Some(header) = headers.get(AUTHORIZATION) {
        if let Ok(text) = header.to_str() {
            if let Some(token) = text.strip_prefix("Bearer ") {
                if !token.is_empty() {
                    return Some(token.to_string());
                }
            }
        }
    }

    query
        .token
        .as_ref()
        .filter(|token| !token.is_empty())
        .cloned()
}

async fn validate_session(state: &AppState, token: &str) -> Option<String> {
    let session = state.storage.get_session(token).await.ok().flatten()?;
    if session.expires_at <= Utc::now() {
        let _ = state.storage.delete_session(token).await;
        return None;
    }
    if !state.storage.is_member(&session.user_id).await.ok()? {
        return None;
    }
    Some(session.user_id)
}
