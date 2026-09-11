use rusqlite::{Connection, Result};
use chrono::{DateTime, Utc};
use zero_core::rules::models::{ClassifiedSession, Classification, ActivityType};
use zero_core::resolver::{ResolvedSession, NormalizedIdentity, NormalizedApplication, NormalizedBrowser};
use zero_core::tracking::session::FinalizationReason;

pub fn get_sessions_in_range(
    conn: &Connection, 
    start_utc: DateTime<Utc>, 
    end_utc: DateTime<Utc>, 
    device_ids: Option<Vec<String>>
) -> Result<Vec<ClassifiedSession>> {
    
    let mut sql = "
        SELECT 
            id, application_id, application_name, browser_name, domain, url, title,
            started_at, ended_at, duration_ms, foreground_ms, interaction_ms, media_ms, idle_ms, 
            classification, activity_type, metadata
        FROM activity_sessions 
        WHERE started_at < ?2 AND ended_at > ?1
    ".to_string();

    if let Some(devices) = &device_ids {
        if !devices.is_empty() {
            let placeholders: Vec<String> = devices.iter().map(|_| "?".to_string()).collect();
            sql.push_str(&format!(" AND device_id IN ({})", placeholders.join(", ")));
        }
    }

    sql.push_str(" ORDER BY started_at ASC");

    let mut stmt = conn.prepare(&sql)?;
    
    let start_str = start_utc.to_rfc3339();
    let end_str = end_utc.to_rfc3339();
    
    let mut p: Vec<&dyn rusqlite::ToSql> = vec![&start_str, &end_str];
    if let Some(devices) = &device_ids {
        if !devices.is_empty() {
            for device in devices {
                p.push(device);
            }
        }
    }

    let rows = stmt.query_map(rusqlite::params_from_iter(p), |row| {
        let session_id: String = row.get(0)?;
        let app_id: Option<String> = row.get(1)?;
        let app_name: Option<String> = row.get(2)?;
        let browser_name: Option<String> = row.get(3)?;
        let domain: Option<String> = row.get(4)?;
        let url: Option<String> = row.get(5)?;
        let title: Option<String> = row.get(6)?;
        
        let started_at: String = row.get(7)?;
        let ended_at: String = row.get(8)?;
        let duration_ms: i64 = row.get(9)?;
        let foreground_ms: Option<i64> = row.get(10)?;
        let interaction_ms: Option<i64> = row.get(11)?;
        let media_ms: Option<i64> = row.get(12)?;
        let idle_ms: Option<i64> = row.get(13)?;
        
        let classification_str: String = row.get(14)?;
        let activity_type_str: String = row.get(15)?;
        let metadata_str: Option<String> = row.get(16)?;

        let start = chrono::DateTime::parse_from_rfc3339(&started_at).unwrap().with_timezone(&Utc);
        let end = chrono::DateTime::parse_from_rfc3339(&ended_at).unwrap().with_timezone(&Utc);

        let identity = if let Some(browser) = browser_name {
            NormalizedIdentity::Browser(NormalizedBrowser {
                browser_name: browser,
                raw_name: app_name.unwrap_or_default(),
                domain,
                url,
                title: title.clone(),
            })
        } else {
            NormalizedIdentity::Application(NormalizedApplication {
                app_id: app_id.unwrap_or_default(),
                normalized_name: "".to_string(), // we don't store this directly, but it's okay for analytics
                raw_name: app_name.unwrap_or_default(),
                path: None, // Can be parsed from metadata if needed
            })
        };

        let metadata = metadata_str.map(|s| serde_json::from_str(&s).unwrap_or(serde_json::json!({})))
            .unwrap_or(serde_json::json!({}));

        let winning_rule_id = metadata.get("winning_rule_id").and_then(|v| v.as_str()).map(|s| s.to_string());
        let reason = metadata.get("reason").and_then(|v| v.as_str()).map(|s| s.to_string());

        let classification = match classification_str.as_str() {
            "productive" => Classification::Productive,
            "neutral" => Classification::Neutral,
            "distracting" => Classification::Distracting,
            _ => Classification::Unknown,
        };

        let activity_type = match activity_type_str.as_str() {
            "application" => ActivityType::Application,
            "website" => ActivityType::Website,
            "media" => ActivityType::Media,
            "browser" => ActivityType::Browser,
            "system" => ActivityType::System,
            _ => ActivityType::Unknown,
        };

        let raw_session = ClassifiedSession {
            resolved_session: ResolvedSession {
                session_id,
                start_utc: start,
                end_utc: end,
                duration_ms: duration_ms as u64,
                foreground_ms: foreground_ms.unwrap_or(duration_ms) as u64,
                interaction_ms: interaction_ms.unwrap_or(0) as u64,
                media_ms: media_ms.unwrap_or(0) as u64,
                idle_ms: idle_ms.unwrap_or(0) as u64,
                identity,
                window_title: title,
                finalization_reason: FinalizationReason::Unknown, // we don't use this in analytics
                metadata,
            },
            classification,
            activity_type,
            winning_rule_id,
            reason,
        };

        // Clamp the session to the requested range bounds so that analytics doesn't double-count overlapping time
        Ok(zero_core::analytics::engine::clamp_session(&raw_session, start_utc, end_utc))
    })?;

    let mut sessions = Vec::new();
    for row in rows {
        if let Some(clamped) = row? {
            sessions.push(clamped);
        }
    }

    Ok(sessions)
}
