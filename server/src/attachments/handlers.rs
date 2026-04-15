use axum::extract::{Path, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use axum_extra::extract::Multipart;
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::attachments::models::AttachmentResponse;
use crate::permissions::{
    effective_permissions, has_permission, UserPermissions, ATTACH_FILES, SEND_MESSAGES,
};
use crate::storage::traits::CreateAttachmentParams;
use crate::AppState;

#[derive(Debug, Serialize)]
pub struct ErrorBody {
    error: String,
}

type ApiResult<T> = Result<(StatusCode, Json<T>), (StatusCode, Json<ErrorBody>)>;

fn bad_request(msg: impl Into<String>) -> (StatusCode, Json<ErrorBody>) {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorBody { error: msg.into() }),
    )
}

fn unprocessable(msg: impl Into<String>) -> (StatusCode, Json<ErrorBody>) {
    (
        StatusCode::UNPROCESSABLE_ENTITY,
        Json(ErrorBody { error: msg.into() }),
    )
}

fn not_found(msg: impl Into<String>) -> (StatusCode, Json<ErrorBody>) {
    (
        StatusCode::NOT_FOUND,
        Json(ErrorBody { error: msg.into() }),
    )
}

fn forbidden(msg: impl Into<String>) -> (StatusCode, Json<ErrorBody>) {
    (
        StatusCode::FORBIDDEN,
        Json(ErrorBody { error: msg.into() }),
    )
}

fn internal(e: crate::storage::StorageError) -> (StatusCode, Json<ErrorBody>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorBody {
            error: e.to_string(),
        }),
    )
}

fn compute_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

fn detect_mime(data: &[u8], content_type: Option<&str>) -> String {
    // Try magic bytes first for images.
    if data.len() >= 8 {
        if data.starts_with(b"\x89PNG") {
            return "image/png".to_string();
        }
        if data.starts_with(b"\xFF\xD8\xFF") {
            return "image/jpeg".to_string();
        }
        if data.starts_with(b"GIF87a") || data.starts_with(b"GIF89a") {
            return "image/gif".to_string();
        }
        if data.len() >= 12 && &data[0..4] == b"RIFF" && &data[8..12] == b"WEBP" {
            return "image/webp".to_string();
        }
        if data.starts_with(b"%PDF") {
            return "application/pdf".to_string();
        }
    }
    content_type
        .map(|s| s.to_string())
        .unwrap_or_else(|| "application/octet-stream".to_string())
}

fn image_dimensions(data: &[u8], mime: &str) -> (Option<i32>, Option<i32>) {
    if !mime.starts_with("image/") {
        return (None, None);
    }
    match image::load_from_memory(data) {
        Ok(img) => (Some(img.width() as i32), Some(img.height() as i32)),
        Err(_) => (None, None),
    }
}

/// POST /api/v1/channels/:channel_id/attachments
pub async fn upload_attachment(
    State(state): State<AppState>,
    auth: UserPermissions,
    Path(channel_id): Path<String>,
    mut multipart: Multipart,
) -> ApiResult<Vec<AttachmentResponse>> {
    // Feature flag check.
    if !state.config.features.file_uploads {
        return Err(forbidden("file uploads are disabled on this server"));
    }

    // Channel existence.
    let channel_exists = state
        .storage
        .get_channel(&channel_id)
        .await
        .map_err(internal)?
        .is_some();
    if !channel_exists {
        return Err(not_found("channel not found"));
    }

    // Permission check.
    let perms = effective_permissions(state.storage.as_ref(), &auth.user_id, Some(&channel_id))
        .await
        .map_err(internal)?;
    if !has_permission(perms, SEND_MESSAGES) || !has_permission(perms, ATTACH_FILES) {
        return Err(forbidden("missing send_messages or attach_files permission"));
    }

    let max_size = state.config.content.max_file_size;
    let allowed_types = &state.config.content.allowed_file_types;
    let mut results = Vec::new();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| bad_request(format!("invalid multipart body: {e}")))?
    {
        let filename = field
            .file_name()
            .unwrap_or("unnamed")
            .to_string();
        let content_type = field.content_type().map(|s| s.to_string());
        let data = field
            .bytes()
            .await
            .map_err(|e| bad_request(format!("failed to read upload: {e}")))?;

        if data.is_empty() {
            return Err(unprocessable("file must not be empty"));
        }

        if data.len() as u64 > max_size {
            return Err(unprocessable(format!(
                "file exceeds maximum size of {} bytes",
                max_size
            )));
        }

        let mime = detect_mime(&data, content_type.as_deref());

        if !allowed_types.is_empty() && !allowed_types.iter().any(|t| t == &mime) {
            return Err(unprocessable(format!(
                "file type {mime} is not allowed"
            )));
        }

        let content_hash = compute_hash(&data);
        let (width, height) = image_dimensions(&data, &mime);

        // Store media bytes (content-addressable, deduped).
        state
            .storage
            .put(&content_hash, &mime, data.len() as u64, &auth.user_id, &data)
            .await
            .map_err(internal)?;

        // Create attachment record.
        let attachment = state
            .storage
            .create_attachment(CreateAttachmentParams {
                channel_id: &channel_id,
                uploader_id: &auth.user_id,
                filename: &filename,
                content_hash: &content_hash,
                size: data.len() as i64,
                mime_type: &mime,
                width,
                height,
            })
            .await
            .map_err(internal)?;

        results.push(AttachmentResponse::from(attachment));
    }

    if results.is_empty() {
        return Err(bad_request("no files provided"));
    }

    Ok((StatusCode::CREATED, Json(results)))
}

/// GET /api/v1/media/:hash — serve a stored media blob.
pub async fn serve_media(
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_is_deterministic() {
        let a = compute_hash(b"hello");
        let b = compute_hash(b"hello");
        assert_eq!(a, b);
        assert_ne!(a, compute_hash(b"world"));
    }

    #[test]
    fn detect_png_mime() {
        let mut data = vec![0u8; 100];
        data[0] = 0x89;
        data[1] = b'P';
        data[2] = b'N';
        data[3] = b'G';
        assert_eq!(detect_mime(&data, None), "image/png");
    }

    #[test]
    fn detect_jpeg_mime() {
        let mut data = vec![0u8; 100];
        data[0] = 0xFF;
        data[1] = 0xD8;
        data[2] = 0xFF;
        assert_eq!(detect_mime(&data, None), "image/jpeg");
    }

    #[test]
    fn detect_mime_fallback_to_content_type() {
        let data = vec![0u8; 100];
        assert_eq!(detect_mime(&data, Some("text/plain")), "text/plain");
    }

    #[test]
    fn detect_mime_fallback_to_octet_stream() {
        let data = vec![0u8; 100];
        assert_eq!(detect_mime(&data, None), "application/octet-stream");
    }
}
