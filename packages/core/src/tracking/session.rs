use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FinalizationReason {
    ApplicationChanged,
    WorkstationLocked,
    Shutdown,
    Recovery,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FinalizedSession {
    pub session_id: String,
    pub start_utc: DateTime<Utc>,
    pub end_utc: DateTime<Utc>,
    pub duration_ms: u64,
    pub app_name: Option<String>,
    pub app_path: Option<String>,
    pub window_title: Option<String>,
    pub process_id: Option<u32>,
    pub finalization_reason: FinalizationReason,
}
