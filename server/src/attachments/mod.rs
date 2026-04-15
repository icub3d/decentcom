pub mod handlers;
pub mod models;

use axum::routing::{get, post};
use axum::Router;

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/channels/:channel_id/attachments",
            post(handlers::upload_attachment),
        )
        .route("/media/:hash", get(handlers::serve_media))
}
