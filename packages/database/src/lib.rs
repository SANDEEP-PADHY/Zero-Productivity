pub mod connection;
pub mod migrations;
pub mod repositories;

pub fn init() {
    println!("Database initialized.");
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Utc, TimeZone};
    use rusqlite::{params, Connection};
    use zero_core::tracking::session::{FinalizedSession, FinalizationReason};
    use zero_core::resolver::{ResolvedSession, NormalizedIdentity, NormalizedApplication};
    use uuid::Uuid;

    fn setup_memory_db() -> rusqlite::Connection {
        let mut conn = connection::open_in_memory().unwrap();
        migrations::run_migrations(&mut conn).unwrap();
        conn
    }

    #[test]
    fn test_migrations_are_idempotent() {
        let mut conn = setup_memory_db();
        // Running a second time should not fail
        assert!(migrations::run_migrations(&mut conn).is_ok());
    }

    #[test]
    fn test_device_creation() {
        let conn = setup_memory_db();
        let device_id = repositories::device::get_or_create_local_device(&conn).unwrap();
        let device_id_2 = repositories::device::get_or_create_local_device(&conn).unwrap();
        assert_eq!(device_id, device_id_2, "Device ID should be stable");
    }

    #[test]
    fn test_insert_session_creates_sync_queue_transactionally() {
        let mut conn = setup_memory_db();
        let device_id = repositories::device::get_or_create_local_device(&conn).unwrap();

        let session = FinalizedSession {
            session_id: Uuid::now_v7().to_string(),
            start_utc: Utc::now(),
            end_utc: Utc::now(),
            duration_ms: 100,
            foreground_ms: 100,
            interaction_ms: 0,
            media_ms: 0,
            idle_ms: 0,
            app_name: None,
            app_path: None,
            window_title: None,
            process_id: None,
            window_handle: None,
            url: None,
            domain: None,
            finalization_reason: FinalizationReason::Unknown,
        };

        let resolved = zero_core::resolver::resolve_session(session);
        let rules_engine = zero_core::rules::RulesEngine::new(vec![]);
        let classified = rules_engine.evaluate(&resolved, &zero_core::rules::PrivacyContext::default());

        repositories::sessions::insert_session(&mut conn, &device_id, &classified).unwrap();

        // Verify session inserted
        let found = repositories::sessions::get_session_by_id(&conn, &resolved.session_id).unwrap();
        assert!(found.is_some());

        // Verify sync queue created
        let queue_count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM sync_queue WHERE record_id = ?1",
            [&resolved.session_id],
            |row| row.get(0)
        ).unwrap();
        assert_eq!(queue_count, 1);
    }

    #[test]
    fn test_insert_session_duplicate_is_idempotent() {
        let mut conn = Connection::open_in_memory().unwrap();
        migrations::run_migrations(&mut conn).unwrap();

        let device_id = repositories::device::get_or_create_local_device(&conn).unwrap();

        let resolved = ResolvedSession {
            session_id: "test-session".to_string(),
            start_utc: Utc::now(),
            end_utc: Utc::now(),
            duration_ms: 5000,
            foreground_ms: 5000,
            interaction_ms: 0,
            media_ms: 0,
            idle_ms: 0,
            identity: NormalizedIdentity::Application(NormalizedApplication {
                app_id: "app:test".to_string(),
                normalized_name: "test".to_string(),
                raw_name: "test".to_string(),
                path: None,
            }),
            window_title: None,
            finalization_reason: FinalizationReason::Unknown,
            metadata: serde_json::Value::Null,
        };
        let rules_engine = zero_core::rules::RulesEngine::new(vec![]);
        let classified = rules_engine.evaluate(&resolved, &zero_core::rules::PrivacyContext::default());

        repositories::sessions::insert_session(&mut conn, &device_id, &classified).unwrap();
        // Second insert must not error and must not duplicate sync queue
        repositories::sessions::insert_session(&mut conn, &device_id, &classified).unwrap();

        let queue_count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM sync_queue WHERE record_id = ?1",
            [&resolved.session_id],
            |row| row.get(0)
        ).unwrap();
        assert_eq!(queue_count, 1, "Should only have one sync queue record for idempotency");
    }

    #[test]
    fn test_retention_protects_unsynced_data() {
        let mut conn = setup_memory_db();
        let device_id = repositories::device::get_or_create_local_device(&conn).unwrap();

        let old_time = Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap();
        let session_id = Uuid::now_v7().to_string();

        conn.execute(
            "INSERT INTO activity_sessions (id, device_id, source, started_at, ended_at, duration_ms, created_at)
             VALUES (?1, ?2, 'win', ?3, ?3, 100, ?3)",
            params![session_id, device_id, old_time.to_rfc3339()],
        ).unwrap();

        // Insert into sync queue so it counts as "unsynced"
        conn.execute(
            "INSERT INTO sync_queue (id, device_id, record_type, record_id, state, created_at, updated_at)
             VALUES ('q1', ?1, 'activity_sessions', ?2, 'pending', ?3, ?3)",
            params![device_id, session_id, old_time.to_rfc3339()],
        ).unwrap();

        let deleted = repositories::retention::cleanup_expired_records(&mut conn).unwrap();
        assert_eq!(deleted, 0, "Should not delete unsynced record even if old");
    }
}
