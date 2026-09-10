use rusqlite::{params, Connection, Result};
use zero_core::rules::models::{Rule, RuleScope, MatchField, MatchType, Classification, ActivityType};

pub struct RulesRepository<'a> {
    conn: &'a Connection,
}

impl<'a> RulesRepository<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn insert(&self, rule: &Rule) -> Result<()> {
        let scope_str = match rule.scope {
            RuleScope::System => "system",
            RuleScope::Account => "account",
            RuleScope::Device => "device",
        };
        let match_field_str = serde_json::to_string(&rule.match_field).unwrap().trim_matches('"').to_string();
        let match_type_str = serde_json::to_string(&rule.match_type).unwrap().trim_matches('"').to_string();
        let classification_str = serde_json::to_string(&rule.classification).unwrap().trim_matches('"').to_string();
        let activity_type_str = serde_json::to_string(&rule.activity_type).unwrap().trim_matches('"').to_string();

        self.conn.execute(
            "INSERT INTO rules (
                id, user_id, scope, priority, enabled, match_field, match_type, match_value, classification, activity_type, created_at, updated_at, version
            ) VALUES (
                ?1, NULL, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, datetime('now'), datetime('now'), 1
            )",
            params![
                rule.id,
                scope_str,
                rule.priority,
                rule.enabled,
                match_field_str,
                match_type_str,
                rule.match_value,
                classification_str,
                activity_type_str
            ],
        )?;
        Ok(())
    }

    pub fn get_all(&self) -> Result<Vec<Rule>> {
        let mut stmt = self.conn.prepare("SELECT id, scope, priority, enabled, match_field, match_type, match_value, classification, activity_type FROM rules")?;
        let rules_iter = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            let scope_str: String = row.get(1)?;
            let priority: i32 = row.get(2)?;
            let enabled: bool = row.get(3)?;
            let match_field_str: String = row.get(4)?;
            let match_type_str: String = row.get(5)?;
            let match_value: String = row.get(6)?;
            let classification_str: String = row.get(7)?;
            let activity_type_str: String = row.get(8)?;

            let scope = match scope_str.as_str() {
                "system" => RuleScope::System,
                "account" => RuleScope::Account,
                "device" => RuleScope::Device,
                _ => RuleScope::System,
            };

            let match_field: MatchField = serde_json::from_str(&format!("\"{}\"", match_field_str)).unwrap_or(MatchField::ApplicationName);
            let match_type: MatchType = serde_json::from_str(&format!("\"{}\"", match_type_str)).unwrap_or(MatchType::Exact);
            let classification: Classification = serde_json::from_str(&format!("\"{}\"", classification_str)).unwrap_or(Classification::Neutral);
            let activity_type: ActivityType = serde_json::from_str(&format!("\"{}\"", activity_type_str)).unwrap_or(ActivityType::Unknown);

            Ok(Rule {
                id,
                scope,
                priority,
                enabled,
                match_field,
                match_type,
                match_value,
                classification,
                activity_type,
            })
        })?;

        let mut rules = Vec::new();
        for rule in rules_iter {
            rules.push(rule?);
        }
        Ok(rules)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use crate::migrations::run_migrations;

    #[test]
    fn test_rules_repository() {
        let mut conn = Connection::open_in_memory().unwrap();
        run_migrations(&mut conn).unwrap();

        let repo = RulesRepository::new(&conn);

        let rule = Rule {
            id: "r1".to_string(),
            scope: RuleScope::System,
            priority: 10,
            enabled: true,
            match_field: MatchField::Domain,
            match_type: MatchType::DomainSubtree,
            match_value: "youtube.com".to_string(),
            classification: Classification::Distracting,
            activity_type: ActivityType::Media,
        };

        repo.insert(&rule).unwrap();

        let rules = repo.get_all().unwrap();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].id, "r1");
        assert_eq!(rules[0].classification, Classification::Distracting);
        assert_eq!(rules[0].activity_type, ActivityType::Media);
    }
}
