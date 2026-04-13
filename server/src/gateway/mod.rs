mod connection;
pub(crate) mod events;
mod handler;
mod registry;

use axum::routing::get;
use axum::Router;

use crate::AppState;

pub type GatewayHandle = registry::ConnectionRegistry;

pub fn gateway_handle() -> GatewayHandle {
    GatewayHandle::default()
}

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(handler::ws_upgrade))
}

#[cfg(test)]
mod tests {
    use super::registry::ConnectionRegistry;
    use super::GatewayHandle;

    #[test]
    fn gateway_handle_is_cloneable() {
        let handle: GatewayHandle = ConnectionRegistry::default();
        let _other = handle.clone();
    }
}
