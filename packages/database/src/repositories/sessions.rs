use rusqlite::{Connection, Result, params, OptionalExtension};
use zero_core::rules::models::ClassifiedSession;
use zero_core::resolver::NormalizedIdentity;
use chrono::Utc;
use uuid::Uuid;

pub fn insert_session(conn: &mut Connection, device_id: &str, session: &ClassifiedSession) -> Result<()> {
    // Check for idempotency first
    let exists: Option<String> = conn.query_row(
        "SELECT id FROM activity_sessions WHERE id = ?1",
        [&session.resolved_session.session_id],
        |row| row.get(0),
    ).optional()?;

    if exists.is_some() {
        return Ok(()); // Already inserted, safely ignore.
    }

    let now = Utc::now().to_rfc3339();
    let started_at = session.resolved_session.start_utc.to_rfc3339();
    let ended_at = session.resolved_session.end_utc.to_rfc3339();

    let (app_id, app_name, browser_name, domain, url) = match &session.resolved_session.identity {
        NormalizedIdentity::Application(app) => (
            Some(app.app_id.clone()),
            Some(app.raw_name.clone()),
            None, None, None
        ),
        NormalizedIdentity::Browser(browser) => (
            None, None,
            Some(browser.browser_name.clone()),
            browser.domain.clone(),
            browser.url.clone()
        ),
    };

    let mut metadata = session.resolved_session.metadata.clone();
    if let Some(metadata_obj) = metadata.as_object_mut() {
        if let Some(winning_rule_id) = &session.winning_rule_id {
            metadata_obj.insert("winning_rule_id".to_string(), serde_json::Value::String(winning_rule_id.clone()));
        }
        if let Some(reason) = &session.reason {
            metadata_obj.insert("reason".to_string(), serde_json::Value::String(reason.clone()));
        }
    }

    let classification_str = serde_json::to_string(&session.classification).unwrap().trim_matches('"').to_string();
    let activity_type_str = serde_json::to_string(&session.activity_type).unwrap().trim_matches('"').to_string();

    let tx = conn.transaction()?;

    tx.execute(
        "INSERT INTO activity_sessions (
            id, device_id, source, application_id, application_name, browser_name, domain, url, title,
            started_at, ended_at, duration_ms, classification, activity_type, metadata, created_at
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16
        )",
        params![
            session.resolved_session.session_id,
            device_id,
            "windows",
            app_id,
            app_name,
            browser_name,
            domain,
            url,
            session.resolved_session.window_title,
            started_at,
            ended_at,
            session.resolved_session.duration_ms as i64,
            classification_str,
            activity_type_str,
            metadata.to_string(),
            now
        ],
    )?;

    // Enqueue for upstream sync
    let queue_id = Uuid::now_v7().to_string();
    tx.execute(
        "INSERT INTO sync_queue (
            id, device_id, record_type, record_id, state, created_at, updated_at
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7
        )",
        params![
            queue_id,
            device_id,
            "activity_sessions",
            session.resolved_session.session_id,
            "pending",
            now,
            now
        ],
    )?;

    tx.commit()?;
    Ok(())
}

pub fn get_session_by_id(conn: &Connection, session_id: &str) -> Result<Option<String>> {
    // Basic existence check to verify tests, we can expand full mapping if needed
    conn.query_row(
        "SELECT id FROM activity_sessions WHERE id = ?1",
        [session_id],
        |row| row.get(0),
    ).optional()
}

