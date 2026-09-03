use rusqlite::{Connection, Result, params};
use chrono::{Utc, Duration};

/// Deletes locally retained data beyond the configured retention period.
/// MUST NEVER delete data that is still pending sync.
pub fn cleanup_expired_records(conn: &mut Connection) -> Result<usize> {
    let now = Utc::now();
    
    // 90 days for sessions
    let sessions_threshold = (now - Duration::try_days(90).unwrap()).to_rfc3339();
    
    let tx = conn.transaction()?;
    
    let deleted_sessions = tx.execute(
        "DELETE FROM activity_sessions 
         WHERE created_at < ?1 
         AND id NOT IN (
             SELECT record_id FROM sync_queue WHERE record_type = 'activity_sessions'
         )",
        params![sessions_threshold],
    )?;

    tx.commit()?;
    Ok(deleted_sessions)
}
