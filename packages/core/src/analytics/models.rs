use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DailySummary {
    pub date: NaiveDate,
    pub total_tracked_ms: u64,
    pub total_foreground_ms: u64,
    pub total_interaction_ms: u64,
    pub total_idle_ms: u64,
    pub total_media_ms: u64,
    pub total_productive_ms: u64,
    pub total_neutral_ms: u64,
    pub total_distracting_ms: u64,
    pub total_uncategorized_ms: u64,
    pub context_switches: u32,
    pub top_applications: Vec<ApplicationSummary>,
    pub top_domains: Vec<DomainSummary>,
    pub longest_sessions: Vec<SessionSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WeeklySummary {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub total_tracked_ms: u64,
    pub total_foreground_ms: u64,
    pub total_interaction_ms: u64,
    pub total_idle_ms: u64,
    pub total_media_ms: u64,
    pub total_productive_ms: u64,
    pub total_neutral_ms: u64,
    pub total_distracting_ms: u64,
    pub total_uncategorized_ms: u64,
    pub context_switches: u32,
    pub top_applications: Vec<ApplicationSummary>,
    pub top_domains: Vec<DomainSummary>,
    pub longest_sessions: Vec<SessionSummary>,
    pub daily_summaries: Vec<DailySummary>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApplicationSummary {
    pub application_id: String,
    pub application_name: Option<String>,
    pub total_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DomainSummary {
    pub domain: String,
    pub total_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionSummary {
    pub session_id: String,
    pub application_id: String,
    pub application_name: Option<String>,
    pub domain: Option<String>,
    pub duration_ms: u64,
    pub classification: crate::rules::models::Classification,
    pub activity_type: crate::rules::models::ActivityType,
}
