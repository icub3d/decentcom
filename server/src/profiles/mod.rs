pub mod avatar;
mod handlers;
mod media;

use axum::routing::{get, patch, put};
use axum::Router;

use crate::AppState;

pub fn profile_router() -> Router<AppState> {
    Router::new()
        .route("/profile", patch(handlers::update_profile))
        .route(
            "/profile/avatar",
            put(handlers::upload_avatar).delete(handlers::delete_avatar),
        )
        .route(
            "/members/:pubkey/profile",
            get(handlers::get_member_profile),
        )
}

pub fn media_router() -> Router<AppState> {
    Router::new().route("/media/:hash", get(media::get_media))
}
