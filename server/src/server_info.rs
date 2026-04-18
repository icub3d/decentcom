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
        membership_mode: serde_json::to_value(state.config.membership.mode)
            .ok()
            .and_then(|v| v.as_str().map(str::to_owned))
            .unwrap_or_default(),
    })
}
