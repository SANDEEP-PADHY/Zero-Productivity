use rusqlite::{Connection, Result, params, OptionalExtension};
use zero_core::tracking::session::FinalizedSession;
use serde_json::json;
use chrono::Utc;
use uuid::Uuid;

pub fn insert_session(conn: &mut Connection, device_id: &str, session: &FinalizedSession) -> Result<()> {
    // Check for idempotency first
    let exists: Option<String> = conn.query_row(
        "SELECT id FROM activity_sessions WHERE id = ?1",
        [&session.session_id],
        |row| row.get(0),
    ).optional()?;

    if exists.is_some() {
        return Ok(()); // Already inserted, safely ignore.
    }

    let metadata = json!({
        "app_path": session.app_path,
        "process_id": session.process_id,
        "finalization_reason": session.finalization_reason,
    }).to_string();

    let now = Utc::now().to_rfc3339();
    let started_at = session.start_utc.to_rfc3339();
    let ended_at = session.end_utc.to_rfc3339();

    let tx = conn.transaction()?;

    tx.execute(
        "INSERT INTO activity_sessions (
            id, device_id, source, application_name, title,
            started_at, ended_at, duration_ms, metadata, created_at
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10
        )",
        params![
            session.session_id,
            device_id,
            "windows",
            session.app_name,
            session.window_title,
            started_at,
            ended_at,
            session.duration_ms as i64,
            metadata,
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
            session.session_id,
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
