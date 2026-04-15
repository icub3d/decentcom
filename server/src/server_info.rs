use axum::extract::State;
use axum::{routing::get, Json, Router};
use shared::ServerInfo;

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/server/info", get(get_server_info))
}

async fn get_server_info(State(state): State<AppState>) -> Json<ServerInfo> {
    Json(ServerInfo {
        name: state.config.server.name.clone(),
        description: state.config.server.description.clone(),
    })
}
