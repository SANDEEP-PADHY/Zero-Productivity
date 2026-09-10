pub mod auth;
pub mod collector;
pub mod sync_worker;
pub mod native_host;

pub fn run() {
    let (tx, rx) = std::sync::mpsc::channel();

    tauri::Builder::default()
        .manage(std::sync::Mutex::new(auth::api::AuthState::new()))
        .invoke_handler(tauri::generate_handler![
            auth::api::login,
            auth::api::logout
        ])
        .setup(move |app| {
            use tauri::Manager;
            zero_core::init();
            
            let app_data_dir = app.path().app_data_dir().expect("Failed to get app data dir");
            std::fs::create_dir_all(&app_data_dir).expect("Failed to create app data dir");
            
            let db_path = app_data_dir.join("data.db");
            let mut conn = database::connection::open_database(&db_path).expect("Failed to open database");
            database::migrations::run_migrations(&mut conn).expect("Failed to run migrations");
            
            // Startup recovery / retention cleanup
            if let Err(e) = database::repositories::retention::cleanup_expired_records(&mut conn) {
                eprintln!("Failed to execute retention cleanup: {:?}", e);
            }

            let device_id = database::repositories::device::get_or_create_local_device(&conn).expect("Failed to get device ID");
            
            println!("Orchestration initialized. Database at {:?}", db_path);

            let manager = collector::CollectorManager::new();
            manager.start();
            let receiver = manager.take_receiver().expect("Failed to take receiver");

            // Spawn Sync Worker asynchronously
            let sync_db_path = db_path.clone();
            tauri::async_runtime::spawn(async move {
                let mut worker = sync_worker::SyncWorker::new(sync_db_path);
                worker.run().await;
            });

            // Spawn IPC Server asynchronously
            let ipc_sender = manager.sender();
            tauri::async_runtime::spawn(async move {
                collector::ipc::start_ipc_server(ipc_sender).await;
            });

            // Manage the collector manager so its Drop triggers WM_QUIT on shutdown
            app.manage(manager);

            let handle = std::thread::spawn(move || {
                use zero_core::tracking::engine::TrackingEngine;
                use zero_core::resolver;
                
                // Load rules for this thread
                let mut conn = database::connection::open_database(&db_path).expect("Failed to open local DB in thread");
                let rules_repo = database::repositories::rules::RulesRepository::new(&conn);
                let rules = rules_repo.get_all().unwrap_or_default();
                let rules_engine = zero_core::rules::RulesEngine::new(rules);
                let privacy_context = zero_core::rules::PrivacyContext::default();

                let clock = collector::clock::WindowsClock;
                let mut engine = TrackingEngine::new(clock);

                loop {
                    match receiver.recv() {
                        Ok(obs) => {
                            if let Some(finalized) = engine.handle_observation(obs) {
                                let resolved = resolver::resolve_session(finalized);
                                let classified = rules_engine.evaluate(&resolved, &privacy_context);
                                if let Err(e) = database::repositories::sessions::insert_session(&mut conn, &device_id, &classified) {
                                    eprintln!("Failed to persist session {}: {:?}", resolved.session_id, e);
                                }
                            }
                        }
                        Err(_) => {
                            // Channel disconnected due to graceful shutdown
                            if let Some(finalized) = engine.shutdown() {
                                let resolved = resolver::resolve_session(finalized);
                                let classified = rules_engine.evaluate(&resolved, &privacy_context);
                                if let Err(e) = database::repositories::sessions::insert_session(&mut conn, &device_id, &classified) {
                                    eprintln!("Failed to persist final session {}: {:?}", resolved.session_id, e);
                                }
                            }
                            break;
                        }
                    }
                }
            });

            tx.send(handle).expect("Failed to send thread handle");

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app_handle, _event| {});

    // Guarantee the orchestration thread finishes its final SQLite write before the process exits.
    // Tauri drops managed state (including CollectorManager) when `run` exits.
    if let Ok(handle) = rx.recv() {
        let _ = handle.join();
    }
}

#[cfg(test)]
mod tests {
    use database::connection::open_in_memory;
    use database::migrations::run_migrations;
    use database::repositories::{device, sessions, retention};
    use zero_core::tracking::engine::TrackingEngine;
    use zero_core::tracking::clock::{FakeClock, Clock};
    use zero_core::models::Observation;
    use zero_core::resolver;
    use chrono::TimeZone;

    #[test]
    fn test_local_orchestration_pipeline() {
        let mut conn = open_in_memory().unwrap();
        run_migrations(&mut conn).unwrap();
        
        let device_id = device::get_or_create_local_device(&conn).unwrap();
        
        let ts = chrono::Utc.with_ymd_and_hms(2026, 9, 3, 10, 0, 0).unwrap();
        let clock = FakeClock::new(ts, 1000);
        let mut engine = TrackingEngine::new(clock.clone());

        // 1. First observation reaches engine
        let obs1 = Observation::new_foreground(ts, 1000, None, Some("app_a.exe".into()), None, 1, 10, "win".into());
        let finalized1 = engine.handle_observation(obs1);
        assert!(finalized1.is_none(), "Session is active, not finalized");

        // 2. Clock advances, switch app
        clock.advance_ms(5000);
        let obs2 = Observation::new_foreground(clock.now_utc(), clock.now_monotonic_ms(), None, Some("app_b.exe".into()), None, 2, 20, "win".into());
        
        // 3. Engine finalizes old session
        let finalized_a = engine.handle_observation(obs2).expect("Expected finalized session for app_a");
        assert_eq!(finalized_a.app_name.as_deref(), Some("app_a.exe"));
        assert_eq!(finalized_a.duration_ms, 5000);

        let resolved_a = resolver::resolve_session(finalized_a.clone());
        assert_eq!(resolved_a.duration_ms, finalized_a.duration_ms);
        assert_eq!(resolved_a.start_utc, finalized_a.start_utc);
        assert_eq!(resolved_a.end_utc, finalized_a.end_utc);

        let rules_engine = zero_core::rules::RulesEngine::new(vec![]);
        let classified_a = rules_engine.evaluate(&resolved_a, &zero_core::rules::PrivacyContext::default());
        assert_eq!(classified_a.resolved_session.duration_ms, resolved_a.duration_ms);
        assert_eq!(classified_a.resolved_session.start_utc, resolved_a.start_utc);
        assert_eq!(classified_a.resolved_session.end_utc, resolved_a.end_utc);

        // 4. Persist to SQLite
        sessions::insert_session(&mut conn, &device_id, &classified_a).expect("Failed to insert");

        // Verify it exists in both tables
        let count: i32 = conn.query_row("SELECT COUNT(*) FROM activity_sessions WHERE id = ?1", [&resolved_a.session_id], |r| r.get(0)).unwrap();
        assert_eq!(count, 1);
        let sync_count: i32 = conn.query_row("SELECT COUNT(*) FROM sync_queue WHERE record_id = ?1", [&resolved_a.session_id], |r| r.get(0)).unwrap();
        assert_eq!(sync_count, 1);
        
        let (db_duration, db_started_at, db_ended_at): (i64, String, String) = conn.query_row(
            "SELECT duration_ms, started_at, ended_at FROM activity_sessions WHERE id = ?1", 
            [&resolved_a.session_id], 
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?))
        ).unwrap();
        
        assert_eq!(db_duration as u64, classified_a.resolved_session.duration_ms);
        assert_eq!(db_started_at, classified_a.resolved_session.start_utc.to_rfc3339());
        assert_eq!(db_ended_at, classified_a.resolved_session.end_utc.to_rfc3339());

        // 5. Graceful shutdown test
        clock.advance_ms(2000);
        let finalized_b = engine.shutdown().expect("Expected finalized session for app_b on shutdown");
        assert_eq!(finalized_b.duration_ms, 2000);

        let resolved_b = resolver::resolve_session(finalized_b);
        let classified_b = rules_engine.evaluate(&resolved_b, &zero_core::rules::PrivacyContext::default());
        sessions::insert_session(&mut conn, &device_id, &classified_b).expect("Failed to insert on shutdown");

        // 6. Retention protects them because they are in sync_queue
        let deleted = retention::cleanup_expired_records(&mut conn).unwrap();
        assert_eq!(deleted, 0);
    }
}

