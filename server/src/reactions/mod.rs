mod handlers;

use axum::routing::{delete, put};
use axum::Router;

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/channels/:channel_id/messages/:message_id/reactions/:emoji",
            put(handlers::put_reaction).get(handlers::list_reactors),
        )
        .route(
            "/channels/:channel_id/messages/:message_id/reactions",
            delete(handlers::delete_own_reaction),
        )
        .route(
            "/channels/:channel_id/messages/:message_id/reactions/users/:user_id",
            delete(handlers::delete_user_reaction),
        )
}
