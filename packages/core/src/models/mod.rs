use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrowserStatePayload {
    pub url: Option<String>,
    pub domain: Option<String>,
    pub title: Option<String>,
    pub window_id: Option<u64>,
    pub tab_id: Option<u64>,
    pub is_focused: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObservationKind {
    ForegroundChange,
    SessionLocked,
    SessionUnlocked,
    SystemSuspended,
    SystemResumed,
    Heartbeat,
    BrowserState(BrowserStatePayload),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Observation {
    pub timestamp: DateTime<Utc>,
    pub monotonic_ms: u64,
    pub kind: ObservationKind,
    pub window_title: Option<String>,
    pub app_name: Option<String>,
    pub app_path: Option<String>,
    pub process_id: Option<u32>,
    pub window_handle: Option<u64>,
    pub platform: String,
}

impl Observation {
    #[allow(clippy::too_many_arguments)]
    pub fn new_foreground(
        timestamp: DateTime<Utc>,
        monotonic_ms: u64,
        window_title: Option<String>,
        app_name: Option<String>,
        app_path: Option<String>,
        process_id: u32,
        window_handle: u64,
        platform: String,
    ) -> Self {
        Self {
            timestamp,
            monotonic_ms,
            kind: ObservationKind::ForegroundChange,
            window_title,
            app_name,
            app_path,
            process_id: Some(process_id),
            window_handle: Some(window_handle),
            platform,
        }
    }

    pub fn new_locked(timestamp: DateTime<Utc>, monotonic_ms: u64, platform: String) -> Self {
        Self {
            timestamp,
            monotonic_ms,
            kind: ObservationKind::SessionLocked,
            window_title: None,
            app_name: None,
            app_path: None,
            process_id: None,
            window_handle: None,
            platform,
        }
    }

    pub fn new_unlocked(timestamp: DateTime<Utc>, monotonic_ms: u64, platform: String) -> Self {
        Self {
            timestamp,
            monotonic_ms,
            kind: ObservationKind::SessionUnlocked,
            window_title: None,
            app_name: None,
            app_path: None,
            process_id: None,
            window_handle: None,
            platform,
        }
    }

    pub fn new_suspended(timestamp: DateTime<Utc>, monotonic_ms: u64, platform: String) -> Self {
        Self {
            timestamp,
            monotonic_ms,
            kind: ObservationKind::SystemSuspended,
            window_title: None,
            app_name: None,
            app_path: None,
            process_id: None,
            window_handle: None,
            platform,
        }
    }

    pub fn new_resumed(timestamp: DateTime<Utc>, monotonic_ms: u64, platform: String) -> Self {
        Self {
            timestamp,
            monotonic_ms,
            kind: ObservationKind::SystemResumed,
            window_title: None,
            app_name: None,
            app_path: None,
            process_id: None,
            window_handle: None,
            platform,
        }
    }
    
    pub fn new_heartbeat(timestamp: DateTime<Utc>, monotonic_ms: u64, platform: String) -> Self {
        Self {
            timestamp,
            monotonic_ms,
            kind: ObservationKind::Heartbeat,
            window_title: None,
            app_name: None,
            app_path: None,
            process_id: None,
            window_handle: None,
            platform,
        }
    }

    pub fn new_browser_state(timestamp: DateTime<Utc>, monotonic_ms: u64, payload: BrowserStatePayload, platform: String) -> Self {
        Self {
            timestamp,
            monotonic_ms,
            kind: ObservationKind::BrowserState(payload),
            window_title: None,
            app_name: None,
            app_path: None,
            process_id: None,
            window_handle: None,
            platform,
        }
    }
}

#[cfg(test)]
mod tests;
