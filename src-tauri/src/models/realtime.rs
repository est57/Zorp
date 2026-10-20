use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

/// How a saved request is used: a normal HTTP call or a long-lived connection.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum RequestMode {
    #[default]
    Http,
    Websocket,
    Sse,
}

#[derive(Debug, Clone, Copy, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum RealtimeKind {
    Open,
    /// Received from the server.
    Message,
    /// Sent by the user.
    Sent,
    Close,
    Error,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RealtimeEvent {
    pub connection_id: Uuid,
    pub kind: RealtimeKind,
    pub data: String,
    /// SSE event name; null for WebSocket and unnamed SSE events.
    pub event: Option<String>,
    pub binary: bool,
    #[ts(type = "number")]
    pub at: i64,
}
