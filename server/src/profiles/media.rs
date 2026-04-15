use axum::extract::{Path, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};

use crate::AppState;

/// GET /media/:hash — serve a stored media blob.
pub(super) async fn get_media(
    State(state): State<AppState>,
    Path(hash): Path<String>,
) -> Result<Response, StatusCode> {
    let (mime_type, bytes) = state
        .storage
        .get_by_content_hash(&hash)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok((
        [
            (header::CONTENT_TYPE, mime_type),
            (
                header::CACHE_CONTROL,
                "public, max-age=31536000, immutable".to_string(),
            ),
        ],
        bytes,
    )
        .into_response())
}
