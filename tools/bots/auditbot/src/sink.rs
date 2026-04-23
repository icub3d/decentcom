use std::sync::Arc;

use serde_json::Value;
use tokio::sync::Mutex;
use tracing::warn;

use crate::config::{Config, SinkType};

/// Shared, async-safe event sink.
#[derive(Clone)]
pub struct Sink {
    inner: Arc<Inner>,
}

enum Inner {
    File {
        path: String,
        lock: Mutex<()>,
    },
    Webhook {
        url: String,
        client: reqwest::Client,
    },
}

impl Sink {
    pub fn from_config(cfg: &Config) -> anyhow::Result<Self> {
        let inner = match cfg.sink_type {
            SinkType::File => {
                let path = cfg
                    .sink_path
                    .clone()
                    .ok_or_else(|| anyhow::anyhow!("sink_path required for file sink"))?;
                Inner::File {
                    path,
                    lock: Mutex::new(()),
                }
            }
            SinkType::Webhook => {
                let url = cfg
                    .sink_url
                    .clone()
                    .ok_or_else(|| anyhow::anyhow!("sink_url required for webhook sink"))?;
                let client = reqwest::Client::builder()
                    .user_agent(concat!("decentcom-auditbot/", env!("CARGO_PKG_VERSION")))
                    .build()?;
                Inner::Webhook { url, client }
            }
        };
        Ok(Self { inner: Arc::new(inner) })
    }

    /// Write a structured event record to the sink.
    pub async fn write(&self, record: &Value) {
        match self.inner.as_ref() {
            Inner::File { path, lock } => {
                let _guard = lock.lock().await;
                let line = match serde_json::to_string(record) {
                    Ok(s) => s + "\n",
                    Err(e) => {
                        warn!("failed to serialize audit record: {e}");
                        return;
                    }
                };
                use std::io::Write;
                let result = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)
                    .and_then(|mut f| f.write_all(line.as_bytes()));
                if let Err(e) = result {
                    warn!("failed to write audit record to {path}: {e}");
                }
            }
            Inner::Webhook { url, client } => {
                if let Err(e) = client.post(url).json(record).send().await {
                    warn!("failed to POST audit record to {url}: {e}");
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{EventFilter, SinkType};
    use serde_json::json;

    #[tokio::test]
    async fn file_sink_appends_json_lines() {
        let dir = std::env::temp_dir();
        let path = dir.join(format!("auditbot-test-{}.ndjson", std::process::id()));
        let path_str = path.to_string_lossy().to_string();

        let cfg = Config {
            server_url: "http://localhost".to_string(),
            mnemonic: "test".to_string(),
            display_name: None,
            sink_type: SinkType::File,
            sink_path: Some(path_str.clone()),
            sink_url: None,
            filter: EventFilter::default(),
        };

        let sink = Sink::from_config(&cfg).unwrap();
        sink.write(&json!({"event": "test1"})).await;
        sink.write(&json!({"event": "test2"})).await;

        let content = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("test1"));
        assert!(lines[1].contains("test2"));

        let _ = std::fs::remove_file(&path);
    }
}
