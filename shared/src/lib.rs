pub mod auth;
pub mod gateway;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServerInfo {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HealthStatus {
    pub status: String,
}

impl HealthStatus {
    pub fn ok() -> Self {
        Self {
            status: "ok".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_status_roundtrip() {
        let s = HealthStatus::ok();
        let json = serde_json::to_string(&s).unwrap();
        let back: HealthStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(s, back);
    }
}
