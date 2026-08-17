use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryRecord {
    pub received_at: u64,
    pub text: String,
    pub code: Option<String>,
    pub source: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ReceiverStatus {
    Starting,
    Listening { port: u16 },
    Degraded { port: u16, message: String },
    Unavailable { port: u16, message: String },
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSnapshot {
    pub history: Vec<HistoryRecord>,
    pub receiver_status: ReceiverStatus,
    pub autostart_enabled: bool,
    pub storage_warning: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct PushPayload {
    pub text: Option<String>,
    pub code: Option<String>,
    pub source: Option<String>,
    pub time: Option<u64>,
}

#[derive(Clone, Debug, Serialize)]
pub struct PushResult {
    pub ok: bool,
    pub code: Option<String>,
    pub repeated: bool,
    pub message: String,
}
