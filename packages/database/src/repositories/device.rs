use rusqlite::{Connection, Result, OptionalExtension};
use uuid::Uuid;
use chrono::Utc;

/// Gets the existing local device ID, or creates a new one if it doesn't exist.
/// This acts as the stable identity for this specific machine.
pub fn get_or_create_local_device(conn: &Connection) -> Result<String> {
    // We assume the first device row is the local device since this is an offline-first DB.
    let existing: Option<String> = conn.query_row(
        "SELECT id FROM devices LIMIT 1",
        [],
        |row| row.get(0),
    ).optional()?;

    if let Some(id) = existing {
        return Ok(id);
    }

    let new_id = Uuid::now_v7().to_string();
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO devices (
            id, platform, first_registered_at, created_at, updated_at
        ) VALUES (?1, ?2, ?3, ?4, ?5)",
        [
            &new_id,
            "windows", // Hardcoded for this MVP stage
            &now,
            &now,
            &now,
        ],
    )?;

    Ok(new_id)
}
