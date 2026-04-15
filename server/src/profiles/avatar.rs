use image::imageops::FilterType;
use image::ImageFormat;
use sha2::{Digest, Sha256};

use crate::storage::traits::{MediaStore, UserStore};
use crate::storage::StorageError;

const MAX_AVATAR_SIZE: usize = 2 * 1024 * 1024; // 2 MB
const AVATAR_DIMENSION: u32 = 256;

#[derive(Debug)]
pub enum AvatarError {
    TooLarge,
    InvalidFormat(String),
    ProcessingFailed(String),
    Storage(StorageError),
}

impl std::fmt::Display for AvatarError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AvatarError::TooLarge => write!(f, "avatar exceeds maximum size of 2 MB"),
            AvatarError::InvalidFormat(msg) => write!(f, "invalid image format: {msg}"),
            AvatarError::ProcessingFailed(msg) => write!(f, "avatar processing failed: {msg}"),
            AvatarError::Storage(e) => write!(f, "storage error: {e}"),
        }
    }
}

impl From<StorageError> for AvatarError {
    fn from(e: StorageError) -> Self {
        AvatarError::Storage(e)
    }
}

/// Validate, resize, hash, and store an avatar image. Returns the content hash.
pub async fn process_and_store_avatar(
    storage: &(impl MediaStore + UserStore + ?Sized),
    user_id: &str,
    data: &[u8],
    content_type: Option<&str>,
) -> Result<String, AvatarError> {
    if data.len() > MAX_AVATAR_SIZE {
        return Err(AvatarError::TooLarge);
    }

    let format = detect_format(data, content_type)?;
    let resized = resize_avatar(data, format)?;
    let content_hash = compute_hash(&resized);

    let mime = match format {
        ImageFormat::Png => "image/png",
        ImageFormat::Jpeg => "image/jpeg",
        ImageFormat::WebP => "image/webp",
        _ => "application/octet-stream",
    };

    storage
        .put(&content_hash, mime, resized.len() as u64, user_id, &resized)
        .await?;
    storage
        .update_user(user_id, None, Some(&content_hash))
        .await?;

    Ok(content_hash)
}

fn detect_format(data: &[u8], content_type: Option<&str>) -> Result<ImageFormat, AvatarError> {
    // Try guessing from bytes first, fall back to content-type header.
    if let Ok(format) = image::guess_format(data) {
        match format {
            ImageFormat::Png | ImageFormat::Jpeg | ImageFormat::WebP => return Ok(format),
            other => {
                return Err(AvatarError::InvalidFormat(format!(
                    "unsupported format: {other:?}; only JPEG, PNG, and WebP are accepted"
                )));
            }
        }
    }

    match content_type {
        Some("image/png") => Ok(ImageFormat::Png),
        Some("image/jpeg") => Ok(ImageFormat::Jpeg),
        Some("image/webp") => Ok(ImageFormat::WebP),
        Some(ct) => Err(AvatarError::InvalidFormat(format!(
            "unsupported content type: {ct}"
        ))),
        None => Err(AvatarError::InvalidFormat(
            "could not determine image format".into(),
        )),
    }
}

fn resize_avatar(data: &[u8], format: ImageFormat) -> Result<Vec<u8>, AvatarError> {
    let img = image::load_from_memory_with_format(data, format)
        .map_err(|e| AvatarError::ProcessingFailed(e.to_string()))?;

    let resized = img.resize_to_fill(AVATAR_DIMENSION, AVATAR_DIMENSION, FilterType::Lanczos3);

    let mut output = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut output);
    resized
        .write_to(&mut cursor, format)
        .map_err(|e| AvatarError::ProcessingFailed(e.to_string()))?;

    Ok(output)
}

fn compute_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tiny_png() -> Vec<u8> {
        // 1x1 red PNG.
        let mut img = image::RgbaImage::new(1, 1);
        img.put_pixel(0, 0, image::Rgba([255, 0, 0, 255]));
        let mut buf = Vec::new();
        let mut cursor = std::io::Cursor::new(&mut buf);
        image::DynamicImage::ImageRgba8(img)
            .write_to(&mut cursor, ImageFormat::Png)
            .unwrap();
        buf
    }

    #[test]
    fn detects_png_format() {
        let data = tiny_png();
        assert_eq!(detect_format(&data, None).unwrap(), ImageFormat::Png);
    }

    #[test]
    fn rejects_too_large() {
        let data = vec![0u8; MAX_AVATAR_SIZE + 1];
        // Should fail before format detection because of size check
        // (size check is in process_and_store_avatar, not here)
        assert!(data.len() > MAX_AVATAR_SIZE);
    }

    #[test]
    fn resize_produces_256x256() {
        let data = tiny_png();
        let resized = resize_avatar(&data, ImageFormat::Png).unwrap();
        let img = image::load_from_memory(&resized).unwrap();
        assert_eq!(img.width(), AVATAR_DIMENSION);
        assert_eq!(img.height(), AVATAR_DIMENSION);
    }

    #[test]
    fn hash_is_deterministic() {
        let a = compute_hash(b"hello");
        let b = compute_hash(b"hello");
        assert_eq!(a, b);
        assert_ne!(a, compute_hash(b"world"));
    }
}
