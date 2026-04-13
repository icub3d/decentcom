use std::time::Duration;

use axum::extract::ws::{CloseFrame, Message, WebSocket};
use futures_util::StreamExt;
use shared::gateway::{ClientCommand, EmptyData, HelloChannel, HelloData, HelloUser, Op};
use tokio::sync::mpsc;
use tokio::time::Instant;

use crate::gateway::events::event_json;
use crate::AppState;

const AUTH_FAILED_CLOSE_CODE: u16 = 4001;
const HEARTBEAT_TIMEOUT_CLOSE_CODE: u16 = 4002;

pub async fn reject_unauthorized(mut socket: WebSocket) {
    let _ = socket
        .send(Message::Close(Some(CloseFrame {
            code: AUTH_FAILED_CLOSE_CODE,
            reason: "unauthorized".into(),
        })))
        .await;
}

pub async fn run(
    mut socket: WebSocket,
    state: AppState,
    connection_id: String,
    user_id: String,
) {
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();
    state
        .gateway
        .register(connection_id.clone(), user_id.clone(), tx.clone());

    let hello = match build_hello_payload(&state, &user_id).await {
        Some(payload) => payload,
        None => {
            state.gateway.unregister(&connection_id);
            return;
        }
    };

    if let Some(hello_event) = event_json(Op::Hello, hello) {
        let _ = tx.send(hello_event);
    }
    if let Some(ready_event) = event_json(Op::Ready, EmptyData {}) {
        let _ = tx.send(ready_event);
    }

    let heartbeat_interval = Duration::from_millis(state.config.gateway.heartbeat_interval_ms);
    let ack_timeout = Duration::from_millis(state.config.gateway.heartbeat_ack_timeout_ms);

    let mut heartbeat_timer = tokio::time::interval(heartbeat_interval);
    heartbeat_timer.tick().await;

    let mut timeout_timer = tokio::time::interval(Duration::from_secs(1));
    timeout_timer.tick().await;

    let mut awaiting_ack = false;
    let mut ack_deadline = Instant::now() + ack_timeout;

    loop {
        tokio::select! {
            maybe_outgoing = rx.recv() => {
                let Some(outgoing) = maybe_outgoing else {
                    break;
                };
                if socket.send(Message::Text(outgoing)).await.is_err() {
                    break;
                }
            }
            maybe_incoming = socket.next() => {
                match maybe_incoming {
                    Some(Ok(Message::Text(text))) => {
                        match serde_json::from_str::<ClientCommand>(&text) {
                            Ok(ClientCommand::Subscribe(data)) => {
                                state.gateway.subscribe(&connection_id, &data.channel_id);
                            }
                            Ok(ClientCommand::Unsubscribe(data)) => {
                                state.gateway.unsubscribe(&connection_id, &data.channel_id);
                            }
                            Ok(ClientCommand::HeartbeatAck(_)) => {
                                awaiting_ack = false;
                            }
                            Ok(ClientCommand::Unknown) | Err(_) => {
                                // Ignore unknown/invalid commands to preserve compatibility.
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        break;
                    }
                    Some(Ok(_)) => {}
                    Some(Err(_)) => {
                        break;
                    }
                }
            }
            _ = heartbeat_timer.tick() => {
                if let Some(heartbeat) = event_json(Op::Heartbeat, EmptyData {}) {
                    if socket.send(Message::Text(heartbeat)).await.is_err() {
                        break;
                    }
                    awaiting_ack = true;
                    ack_deadline = Instant::now() + ack_timeout;
                }
            }
            _ = timeout_timer.tick() => {
                if awaiting_ack && Instant::now() >= ack_deadline {
                    let _ = socket
                        .send(Message::Close(Some(CloseFrame {
                            code: HEARTBEAT_TIMEOUT_CLOSE_CODE,
                            reason: "heartbeat timeout".into(),
                        })))
                        .await;
                    break;
                }
            }
        }
    }

    state.gateway.unregister(&connection_id);
}

async fn build_hello_payload(state: &AppState, user_id: &str) -> Option<HelloData> {
    let user = state.storage.get_user_by_id(user_id).await.ok().flatten()?;
    let channels = state.storage.list_channels().await.ok()?;

    let hello_channels = channels
        .into_iter()
        .map(|channel| HelloChannel {
            id: channel.id,
            name: channel.name,
        })
        .collect();

    Some(HelloData {
        server_name: state.config.server.name.clone(),
        user: HelloUser {
            id: user.id,
            pubkey: user.pubkey,
            display_name: user.display_name,
        },
        channels: hello_channels,
        heartbeat_interval_ms: state.config.gateway.heartbeat_interval_ms,
    })
}
