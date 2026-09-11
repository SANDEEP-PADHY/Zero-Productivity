use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::tracking::session::FinalizationReason;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalizedApplication {
    pub app_id: String,
    pub normalized_name: String,
    pub raw_name: String,
    pub path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalizedBrowser {
    pub browser_name: String,
    pub raw_name: String,
    pub domain: Option<String>,
    pub url: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NormalizedIdentity {
    Application(NormalizedApplication),
    Browser(NormalizedBrowser),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedSession {
    pub session_id: String,
    pub start_utc: DateTime<Utc>,
    pub end_utc: DateTime<Utc>,
    pub duration_ms: u64,
    pub foreground_ms: u64,
    pub interaction_ms: u64,
    pub media_ms: u64,
    pub idle_ms: u64,
    pub identity: NormalizedIdentity,
    pub window_title: Option<String>,
    pub finalization_reason: FinalizationReason,
    pub metadata: serde_json::Value,
}
