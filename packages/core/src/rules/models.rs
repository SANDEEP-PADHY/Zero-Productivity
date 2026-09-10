// Removed chrono

use serde::{Deserialize, Serialize};
use crate::resolver::ResolvedSession;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Classification {
    Productive,
    Neutral,
    Distracting,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ActivityType {
    Application,
    Website,
    Media,
    Browser,
    System,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum RuleScope {
    System = 0,
    Account = 1,
    Device = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MatchType {
    Exact,
    Prefix,
    Suffix,
    Contains,
    DomainExact,
    DomainSubtree,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MatchField {
    ApplicationId,
    ApplicationName,
    BrowserName,
    Domain,
    Url,
    Title,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rule {
    pub id: String,
    pub scope: RuleScope,
    pub priority: i32,
    pub enabled: bool,
    pub match_field: MatchField,
    pub match_type: MatchType,
    pub match_value: String,
    pub classification: Classification,
    pub activity_type: ActivityType,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClassifiedSession {
    pub resolved_session: ResolvedSession,
    pub classification: Classification,
    pub activity_type: ActivityType,
    pub winning_rule_id: Option<String>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrivacyContext {
    pub url_collection_enabled: bool,
    pub title_collection_enabled: bool,
}

impl Default for PrivacyContext {
    fn default() -> Self {
        Self {
            url_collection_enabled: true,
            title_collection_enabled: true,
        }
    }
}
