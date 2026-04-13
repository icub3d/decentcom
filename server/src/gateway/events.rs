use chrono::Utc;
use serde::Serialize;
use shared::gateway::{EventEnvelope, Op};

pub fn event_json<T: Serialize>(op: Op, data: T) -> Option<String> {
    serde_json::to_string(&EventEnvelope {
        op,
        d: data,
        t: Utc::now().timestamp(),
    })
    .ok()
}
