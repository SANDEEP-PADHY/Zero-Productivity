use crate::resolver::{NormalizedIdentity, ResolvedSession};
use crate::rules::models::{
    ActivityType, Classification, ClassifiedSession, MatchField, MatchType, PrivacyContext, Rule,
};

pub struct RulesEngine {
    rules: Vec<Rule>,
}

impl RulesEngine {
    pub fn new(mut rules: Vec<Rule>) -> Self {
        // Sort rules for deterministic evaluation.
        // Precedence: 
        // 1. Device > Account > System (RuleScope ordinal)
        // 2. Priority (higher integer wins)
        // 3. ID (fallback lexicographical sort for deterministic tie-breaking)
        rules.sort_by(|a, b| {
            a.scope.cmp(&b.scope).reverse() // Device (2) comes before System (0)
                .then_with(|| b.priority.cmp(&a.priority)) // Higher priority wins
                .then_with(|| a.id.cmp(&b.id)) // Tie-breaker
        });
        
        Self { rules }
    }

    pub fn evaluate(
        &self,
        session: &ResolvedSession,
        privacy: &PrivacyContext,
    ) -> ClassifiedSession {
        for rule in &self.rules {
            if !rule.enabled {
                continue;
            }
            if self.matches(rule, session, privacy) {
                return ClassifiedSession {
                    resolved_session: session.clone(),
                    classification: rule.classification,
                    activity_type: rule.activity_type,
                    winning_rule_id: Some(rule.id.clone()),
                    reason: Some(format!("Matched {:?} = {:?}", rule.match_field, rule.match_value)),
                };
            }
        }

        // Default behavior if no rules match
        let default_activity_type = match &session.identity {
            NormalizedIdentity::Application(_) => ActivityType::Application,
            NormalizedIdentity::Browser(b) => {
                if b.domain.is_some() || b.url.is_some() {
                    ActivityType::Website
                } else {
                    ActivityType::Browser
                }
            }
        };

        ClassifiedSession {
            resolved_session: session.clone(),
            classification: Classification::Neutral,
            activity_type: default_activity_type,
            winning_rule_id: None,
            reason: Some("Default fallback".to_string()),
        }
    }

    fn matches(&self, rule: &Rule, session: &ResolvedSession, privacy: &PrivacyContext) -> bool {
        let value_to_match = match rule.match_field {
            MatchField::ApplicationId => {
                match &session.identity {
                    NormalizedIdentity::Application(app) => Some(app.app_id.as_str()),
                    NormalizedIdentity::Browser(_) => None,
                }
            }
            MatchField::ApplicationName => {
                match &session.identity {
                    NormalizedIdentity::Application(app) => Some(app.normalized_name.as_str()),
                    NormalizedIdentity::Browser(_) => None,
                }
            }
            MatchField::BrowserName => {
                match &session.identity {
                    NormalizedIdentity::Application(_) => None,
                    NormalizedIdentity::Browser(browser) => Some(browser.browser_name.as_str()),
                }
            }
            MatchField::Domain => {
                match &session.identity {
                    NormalizedIdentity::Application(_) => None,
                    NormalizedIdentity::Browser(browser) => browser.domain.as_deref(),
                }
            }
            MatchField::Url => {
                if !privacy.url_collection_enabled {
                    return false;
                }
                match &session.identity {
                    NormalizedIdentity::Application(_) => None,
                    NormalizedIdentity::Browser(browser) => browser.url.as_deref(),
                }
            }
            MatchField::Title => {
                if !privacy.title_collection_enabled {
                    return false;
                }
                session.window_title.as_deref()
            }
        };

        let value_to_match = match value_to_match {
            Some(v) => v,
            None => return false,
        };

        let target = &rule.match_value;
        let is_case_insensitive = matches!(rule.match_field, MatchField::ApplicationName | MatchField::BrowserName | MatchField::Domain);

        let v_match = if is_case_insensitive {
            value_to_match.to_lowercase()
        } else {
            value_to_match.to_string()
        };
        let target_match = if is_case_insensitive {
            target.to_lowercase()
        } else {
            target.to_string()
        };

        match rule.match_type {
            MatchType::Exact => v_match == target_match,
            MatchType::Prefix => v_match.starts_with(&target_match),
            MatchType::Suffix => v_match.ends_with(&target_match),
            MatchType::Contains => v_match.contains(&target_match),
            MatchType::DomainExact => {
                // e.g. rule: example.com, actual: example.com (match), actual: sub.example.com (no match)
                v_match == target_match
            }
            MatchType::DomainSubtree => {
                // e.g. rule: example.com, actual: example.com (match), actual: sub.example.com (match)
                // actual: notexample.com (no match)
                if v_match == target_match {
                    true
                } else {
                    v_match.ends_with(&format!(".{}", target_match))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::{NormalizedApplication, NormalizedBrowser};
    use crate::rules::models::RuleScope;
    use crate::tracking::session::FinalizationReason;
    use chrono::Utc;

    fn mock_session_app(app_id: &str, app_name: &str) -> ResolvedSession {
        ResolvedSession {
            session_id: "test".to_string(),
            start_utc: Utc::now(),
            end_utc: Utc::now(),
            duration_ms: 1000,
            foreground_ms: 1000,
            interaction_ms: 0,
            media_ms: 0,
            idle_ms: 0,
            identity: NormalizedIdentity::Application(NormalizedApplication {
                app_id: app_id.to_string(),
                normalized_name: app_name.to_string(),
                raw_name: app_name.to_string(),
                path: None,
            }),
            window_title: Some("Some title".to_string()),
            finalization_reason: FinalizationReason::Unknown,
            metadata: serde_json::Value::Null,
        }
    }

    fn mock_session_browser(browser: &str, domain: Option<&str>, url: Option<&str>, title: Option<&str>) -> ResolvedSession {
        ResolvedSession {
            session_id: "test".to_string(),
            start_utc: Utc::now(),
            end_utc: Utc::now(),
            duration_ms: 1000,
            foreground_ms: 1000,
            interaction_ms: 0,
            media_ms: 0,
            idle_ms: 0,
            identity: NormalizedIdentity::Browser(NormalizedBrowser {
                browser_name: browser.to_string(),
                raw_name: browser.to_string(),
                domain: domain.map(|s| s.to_string()),
                url: url.map(|s| s.to_string()),
                title: title.map(|s| s.to_string()),
            }),
            window_title: title.map(|s| s.to_string()),
            finalization_reason: FinalizationReason::Unknown,
            metadata: serde_json::Value::Null,
        }
    }

    #[test]
    fn test_empty_rules_fallback() {
        let engine = RulesEngine::new(vec![]);
        let session = mock_session_app("app:code", "code");
        let result = engine.evaluate(&session, &PrivacyContext::default());
        assert_eq!(result.classification, Classification::Neutral);
        assert_eq!(result.activity_type, ActivityType::Application);
        assert_eq!(result.winning_rule_id, None);
    }

    #[test]
    fn test_bare_browser_fallback() {
        let engine = RulesEngine::new(vec![]);
        let session_with_domain = mock_session_browser("chrome", Some("example.com"), None, None);
        let result = engine.evaluate(&session_with_domain, &PrivacyContext::default());
        assert_eq!(result.activity_type, ActivityType::Website);

        let session_no_domain = mock_session_browser("chrome", None, None, None);
        let result2 = engine.evaluate(&session_no_domain, &PrivacyContext::default());
        assert_eq!(result2.activity_type, ActivityType::Browser);
    }

    #[test]
    fn test_application_exact_match() {
        let rule = Rule {
            id: "r1".to_string(),
            scope: RuleScope::System,
            priority: 0,
            enabled: true,
            match_field: MatchField::ApplicationName,
            match_type: MatchType::Exact,
            match_value: "code".to_string(),
            classification: Classification::Productive,
            activity_type: ActivityType::Application,
        };
        let engine = RulesEngine::new(vec![rule]);
        
        let session_match = mock_session_app("app:code", "code");
        let session_no_match = mock_session_app("app:slack", "slack");
        
        assert_eq!(engine.evaluate(&session_match, &PrivacyContext::default()).classification, Classification::Productive);
        assert_eq!(engine.evaluate(&session_no_match, &PrivacyContext::default()).classification, Classification::Neutral);
    }

    #[test]
    fn test_domain_subtree_match() {
        let rule = Rule {
            id: "r1".to_string(),
            scope: RuleScope::System,
            priority: 0,
            enabled: true,
            match_field: MatchField::Domain,
            match_type: MatchType::DomainSubtree,
            match_value: "example.com".to_string(),
            classification: Classification::Distracting,
            activity_type: ActivityType::Website,
        };
        let engine = RulesEngine::new(vec![rule]);
        
        let session1 = mock_session_browser("chrome", Some("example.com"), None, None);
        let session2 = mock_session_browser("chrome", Some("sub.example.com"), None, None);
        let session3 = mock_session_browser("chrome", Some("notexample.com"), None, None);
        
        assert_eq!(engine.evaluate(&session1, &PrivacyContext::default()).classification, Classification::Distracting);
        assert_eq!(engine.evaluate(&session2, &PrivacyContext::default()).classification, Classification::Distracting);
        assert_eq!(engine.evaluate(&session3, &PrivacyContext::default()).classification, Classification::Neutral);
    }

    #[test]
    fn test_precedence_and_conflict() {
        let r1 = Rule {
            id: "r1".to_string(),
            scope: RuleScope::System,
            priority: 0,
            enabled: true,
            match_field: MatchField::Domain,
            match_type: MatchType::DomainSubtree,
            match_value: "example.com".to_string(),
            classification: Classification::Distracting,
            activity_type: ActivityType::Website,
        };
        let r2 = Rule {
            id: "r2".to_string(),
            scope: RuleScope::Device, // Higher scope
            priority: 0,
            enabled: true,
            match_field: MatchField::Domain,
            match_type: MatchType::Exact,
            match_value: "example.com".to_string(),
            classification: Classification::Productive,
            activity_type: ActivityType::Website,
        };
        let engine = RulesEngine::new(vec![r1, r2]);
        let session = mock_session_browser("chrome", Some("example.com"), None, None);
        
        let result = engine.evaluate(&session, &PrivacyContext::default());
        assert_eq!(result.classification, Classification::Productive);
        assert_eq!(result.winning_rule_id, Some("r2".to_string()));
    }

    #[test]
    fn test_privacy_disabled() {
        let r1 = Rule {
            id: "r1".to_string(),
            scope: RuleScope::System,
            priority: 0,
            enabled: true,
            match_field: MatchField::Title,
            match_type: MatchType::Contains,
            match_value: "secret".to_string(),
            classification: Classification::Productive,
            activity_type: ActivityType::Application,
        };
        let engine = RulesEngine::new(vec![r1]);
        let _session = mock_session_app("app:code", "code");
        
        // With title collection enabled, it matches (title is "Some title" - oops, doesn't contain secret, let's make it match)
        let mut session_secret = mock_session_app("app:code", "code");
        session_secret.window_title = Some("secret project".to_string());
        
        let result_enabled = engine.evaluate(&session_secret, &PrivacyContext { title_collection_enabled: true, url_collection_enabled: true });
        assert_eq!(result_enabled.classification, Classification::Productive);
        
        let result_disabled = engine.evaluate(&session_secret, &PrivacyContext { title_collection_enabled: false, url_collection_enabled: true });
        assert_eq!(result_disabled.classification, Classification::Neutral);
        assert_eq!(result_disabled.reason.as_deref(), Some("Default fallback"));
    }

    #[test]
    fn test_media_activity_type() {
        let r1 = Rule {
            id: "r1".to_string(),
            scope: RuleScope::System,
            priority: 0,
            enabled: true,
            match_field: MatchField::Domain,
            match_type: MatchType::DomainSubtree,
            match_value: "youtube.com".to_string(),
            classification: Classification::Neutral,
            activity_type: ActivityType::Media,
        };
        let engine = RulesEngine::new(vec![r1]);
        let session = mock_session_browser("chrome", Some("youtube.com"), None, None);
        let result = engine.evaluate(&session, &PrivacyContext::default());
        assert_eq!(result.activity_type, ActivityType::Media);
    }
}
