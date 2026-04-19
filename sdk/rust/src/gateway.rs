//! Low-level WebSocket gateway client.
//!
//! Handles the connection lifecycle, automatic heartbeat acknowledgements,
//! and surfaces typed server events as a stream. Wire types live in
//! `shared::gateway`; this module implements the client-side protocol only.

use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use shared::gateway::{EmptyData, HelloData, Op, SubscriptionData};
use tokio::sync::{mpsc, Mutex};
use tokio::task::JoinHandle;
use tokio_tungstenite::tungstenite::protocol::Message;
use url::Url;

use crate::error::{Error, Result};

type WsStream = tokio_tungstenite::WebSocketStream<
    tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
>;
type WsSink = futures_util::stream::SplitSink<WsStream, Message>;

/// A typed gateway event. The raw envelope is exposed so callers can
/// deserialize `data` into any payload type relevant to the op code (see
/// [`GatewayEvent::decode`]).
#[derive(Debug, Clone)]
pub struct GatewayEvent {
    pub op: Op,
    pub data: Value,
    pub t: i64,
}

impl GatewayEvent {
    /// Decode the `d` field as a specific payload type.
    pub fn decode<T: DeserializeOwned>(&self) -> Result<T> {
        serde_json::from_value(self.data.clone()).map_err(Error::Json)
    }
}

/// Builder for a [`GatewayClient`].
#[derive(Debug, Clone)]
pub struct GatewayBuilder {
    base_url: Url,
    token: Option<String>,
    path: String,
    auto_heartbeat_ack: bool,
    event_buffer: usize,
}

impl GatewayBuilder {
    pub(crate) fn new(base_url: Url, token: Option<String>) -> Self {
        Self {
            base_url,
            token,
            path: "/api/v1/gateway".to_string(),
            auto_heartbeat_ack: true,
            event_buffer: 256,
        }
    }

    /// Override the gateway path (defaults to `/api/v1/gateway`).
    pub fn path(mut self, path: impl Into<String>) -> Self {
        self.path = path.into();
        self
    }

    /// Disable automatic heartbeat acknowledgement. Callers must respond to
    /// `Op::Heartbeat` events themselves via [`GatewayClient::send_raw`].
    pub fn auto_heartbeat_ack(mut self, enabled: bool) -> Self {
        self.auto_heartbeat_ack = enabled;
        self
    }

    /// Size of the channel buffering decoded events.
    pub fn event_buffer(mut self, size: usize) -> Self {
        self.event_buffer = size.max(1);
        self
    }

    /// Connect to the gateway and spawn the background reader task.
    pub async fn connect(self) -> Result<GatewayClient> {
        let token = self.token.ok_or(Error::Unauthenticated)?;
        let url = build_ws_url(&self.base_url, &self.path, &token)?;

        let (stream, _response) = tokio_tungstenite::connect_async(url.as_str())
            .await
            .map_err(|e| Error::Gateway(e.to_string()))?;

        let (sink, mut source) = stream.split();
        let sink = Arc::new(Mutex::new(sink));
        let (tx, rx) = mpsc::channel::<GatewayEvent>(self.event_buffer);

        let sink_for_task = sink.clone();
        let auto_ack = self.auto_heartbeat_ack;

        let reader = tokio::spawn(async move {
            while let Some(msg) = source.next().await {
                let msg = match msg {
                    Ok(m) => m,
                    Err(_) => break,
                };
                let text = match msg {
                    Message::Text(t) => t,
                    Message::Binary(b) => match String::from_utf8(b) {
                        Ok(t) => t,
                        Err(_) => continue,
                    },
                    Message::Ping(_) | Message::Pong(_) | Message::Frame(_) => continue,
                    Message::Close(_) => break,
                };
                let value: Value = match serde_json::from_str(&text) {
                    Ok(v) => v,
                    Err(_) => continue,
                };
                let op: Op = match value.get("op").cloned().and_then(|v| {
                    serde_json::from_value::<Op>(v).ok()
                }) {
                    Some(op) => op,
                    None => continue,
                };
                let data = value.get("d").cloned().unwrap_or(Value::Null);
                let t = value.get("t").and_then(Value::as_i64).unwrap_or(0);

                if auto_ack && op == Op::Heartbeat {
                    let ack =
                        serde_json::json!({ "op": "HEARTBEAT_ACK", "d": {} }).to_string();
                    let mut sink = sink_for_task.lock().await;
                    if sink.send(Message::Text(ack)).await.is_err() {
                        break;
                    }
                }

                if tx.send(GatewayEvent { op, data, t }).await.is_err() {
                    break;
                }
            }
        });

        Ok(GatewayClient {
            sink,
            rx,
            reader: Some(reader),
        })
    }
}

fn build_ws_url(base: &Url, path: &str, token: &str) -> Result<Url> {
    let mut url = base.clone();
    let target_scheme = match url.scheme() {
        "https" => "wss",
        "http" => "ws",
        "wss" | "ws" => url.scheme(),
        _ => return Err(Error::Protocol("unsupported base URL scheme".into())),
    }
    .to_string();
    url.set_scheme(&target_scheme)
        .map_err(|_| Error::Protocol("unsupported base URL scheme".into()))?;
    url.set_path(path);
    url.set_query(None);
    url.query_pairs_mut().append_pair("token", token);
    Ok(url)
}

/// A connected WebSocket gateway client with a background reader task.
pub struct GatewayClient {
    sink: Arc<Mutex<WsSink>>,
    rx: mpsc::Receiver<GatewayEvent>,
    reader: Option<JoinHandle<()>>,
}

impl GatewayClient {
    /// Receive the next decoded gateway event, or `None` when the connection
    /// has closed.
    pub async fn next_event(&mut self) -> Option<GatewayEvent> {
        self.rx.recv().await
    }

    /// Wait for the first HELLO event and return its decoded payload.
    pub async fn wait_for_hello(&mut self) -> Result<HelloData> {
        loop {
            match self.next_event().await {
                Some(ev) if ev.op == Op::Hello => return ev.decode::<HelloData>(),
                Some(_) => continue,
                None => {
                    return Err(Error::Gateway(
                        "connection closed before hello".to_string(),
                    ))
                }
            }
        }
    }

    /// Subscribe to the given channel's event feed.
    pub async fn subscribe(&self, channel_id: impl Into<String>) -> Result<()> {
        self.send_command(
            "SUBSCRIBE",
            &SubscriptionData {
                channel_id: channel_id.into(),
            },
        )
        .await
    }

    /// Unsubscribe from the given channel.
    pub async fn unsubscribe(&self, channel_id: impl Into<String>) -> Result<()> {
        self.send_command(
            "UNSUBSCRIBE",
            &SubscriptionData {
                channel_id: channel_id.into(),
            },
        )
        .await
    }

    /// Send an explicit heartbeat ACK (only needed when automatic ACKs are
    /// disabled via [`GatewayBuilder::auto_heartbeat_ack`]).
    pub async fn heartbeat_ack(&self) -> Result<()> {
        self.send_command("HEARTBEAT_ACK", &EmptyData {}).await
    }

    /// Send a raw JSON command frame. Most callers should prefer the typed
    /// helpers above.
    pub async fn send_raw(&self, value: &Value) -> Result<()> {
        let text = serde_json::to_string(value)?;
        let mut sink = self.sink.lock().await;
        sink.send(Message::Text(text))
            .await
            .map_err(|e| Error::Gateway(e.to_string()))
    }

    async fn send_command<T: Serialize>(&self, op: &str, data: &T) -> Result<()> {
        let value = serde_json::json!({ "op": op, "d": data });
        self.send_raw(&value).await
    }

    /// Close the WebSocket cleanly and stop the reader task.
    pub async fn close(mut self) -> Result<()> {
        {
            let mut sink = self.sink.lock().await;
            let _ = sink.send(Message::Close(None)).await;
            let _ = sink.close().await;
        }
        if let Some(reader) = self.reader.take() {
            reader.abort();
        }
        Ok(())
    }
}

impl Drop for GatewayClient {
    fn drop(&mut self) {
        if let Some(reader) = self.reader.take() {
            reader.abort();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ws_url_http_becomes_ws() {
        let base = Url::parse("http://localhost:3000/").unwrap();
        let url = build_ws_url(&base, "/api/v1/gateway", "abc123").unwrap();
        assert_eq!(url.scheme(), "ws");
        assert_eq!(url.path(), "/api/v1/gateway");
        assert_eq!(url.query(), Some("token=abc123"));
    }

    #[test]
    fn ws_url_https_becomes_wss() {
        let base = Url::parse("https://example.com/").unwrap();
        let url = build_ws_url(&base, "/api/v1/gateway", "t").unwrap();
        assert_eq!(url.scheme(), "wss");
    }

    #[test]
    fn event_decode_roundtrip() {
        let data = serde_json::json!({ "channel_id": "c1" });
        let event = GatewayEvent {
            op: Op::ChannelCreate,
            data,
            t: 0,
        };
        let decoded: SubscriptionData = event.decode().unwrap();
        assert_eq!(decoded.channel_id, "c1");
    }
}
