use serde::Serialize;
use ts_rs::TS;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, TS)]
#[serde(tag = "type", rename_all = "snake_case")]
#[ts(export)]
pub enum ResponseBody {
    Text { text: String },
    Base64 { data: String, mime: String },
    TooLarge { preview: String, path: String },
}

#[derive(Debug, Clone, Default, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Timing {
    pub ttfb_ms: u32,
    pub download_ms: u32,
    pub total_ms: u32,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ResponseData {
    pub status: u16,
    pub status_text: String,
    pub headers: Vec<(String, String)>,
    pub body: ResponseBody,
    #[ts(type = "number")]
    pub size_bytes: u64,
    pub timing: Timing,
    pub http_version: String,
    pub url: String,
}

#[derive(Debug, Clone, Copy, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum ProgressPhase {
    Sending,
    Downloading,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ProgressEvent {
    pub request_id: Uuid,
    pub phase: ProgressPhase,
    #[ts(type = "number")]
    pub bytes: u64,
}
