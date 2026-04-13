use async_trait::async_trait;
use axum::extract::FromRequestParts;
use axum::extract::FromRef;
use axum::http::{header::AUTHORIZATION, request::Parts};
use axum::response::{IntoResponse, Response};
use axum::{http::StatusCode, Json};
use chrono::Utc;
use serde::Serialize;

use crate::AppState;

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: String,
}

#[derive(Debug)]
pub struct AuthRejection {
    status: StatusCode,
    message: String,
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    error: String,
}

impl AuthRejection {
    fn unauthorized(message: &str) -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            message: message.to_string(),
        }
    }
}

impl IntoResponse for AuthRejection {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ErrorBody {
                error: self.message,
            }),
        )
            .into_response()
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AuthRejection;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let state = AppState::from_ref(state);

        let header = parts
            .headers
            .get(AUTHORIZATION)
            .ok_or_else(|| AuthRejection::unauthorized("missing authorization header"))?;

        let header = header
            .to_str()
            .map_err(|_| AuthRejection::unauthorized("invalid authorization header"))?;

        let token = header
            .strip_prefix("Bearer ")
            .ok_or_else(|| AuthRejection::unauthorized("invalid bearer token"))?;

        let session = state
            .storage
            .get_session(token)
            .await
            .map_err(|_| AuthRejection::unauthorized("invalid session token"))?
            .ok_or_else(|| AuthRejection::unauthorized("invalid session token"))?;

        if session.expires_at <= Utc::now() {
            let _ = state.storage.delete_session(token).await;
            return Err(AuthRejection::unauthorized("session token expired"));
        }

        Ok(AuthUser {
            user_id: session.user_id,
        })
    }
}
