mod supabase;

#[cfg(test)]
mod tests;

use std::path::PathBuf;
use std::time::Duration;
use tokio::time::sleep;
use database::connection;
use serde_json::{json, Value};
use chrono::Utc;
use rusqlite::OptionalExtension;

const DEFAULT_SYNC_INTERVAL: Duration = Duration::from_secs(15 * 60); // 15 minutes
const MIN_BACKOFF: Duration = Duration::from_secs(30);
const MAX_BACKOFF: Duration = Duration::from_secs(3600); // 1 hour

pub struct SyncWorker {
    db_path: PathBuf,
    client: supabase::SupabaseClient,
    current_backoff: Duration,
}

impl SyncWorker {
    pub fn new(db_path: PathBuf) -> Self {
        let api_url = std::env::var("SUPABASE_URL").unwrap_or_else(|_| "http://127.0.0.1:8000".to_string());
        let anon_key = std::env::var("SUPABASE_ANON_KEY").unwrap_or_else(|_| "dummy_anon_key".to_string());
        
        Self {
            db_path,
            client: supabase::SupabaseClient::new(api_url, anon_key),
            current_backoff: MIN_BACKOFF,
        }
    }

    pub async fn run(&mut self) {
        loop {
            match self.sync_cycle().await {
                Ok(true) => {
                    // Success, reset backoff and sleep for default interval
                    self.current_backoff = MIN_BACKOFF;
                    sleep(DEFAULT_SYNC_INTERVAL).await;
                }
                Ok(false) => {
                    // No-op / Unauthenticated
                    sleep(DEFAULT_SYNC_INTERVAL).await;
                }
                Err(e) => {
                    eprintln!("Sync Worker Error: {:?}", e);
                    // Apply exponential backoff
                    sleep(self.current_backoff).await;
                    self.current_backoff = std::cmp::min(self.current_backoff * 2, MAX_BACKOFF);
                }
            }
        }
    }

    async fn sync_cycle(&mut self) -> Result<bool, String> {
        let auth_state = crate::auth::api::AuthState::new();
        
        let access_token = match auth_state.get_access_token() {
            Ok(token) => token,
            Err(_) => return Ok(false), // Not logged in or no token
        };

        self.client.set_jwt(access_token);

        // Verify auth (if token expired, we might get 401 later, but we can do a preemptive check or just react to 401)
        if !self.client.has_auth() {
            return Ok(false);
        }

        let db_path = self.db_path.clone();
        let device_id = tokio::task::spawn_blocking(move || {
            let conn = connection::open_database(&db_path)
                .map_err(|e| format!("Failed to open DB for sync: {:?}", e))?;
            database::repositories::device::get_or_create_local_device(&conn)
                .map_err(|e| format!("Failed to get device ID: {:?}", e))
        }).await.map_err(|e| e.to_string())??;

        // Local -> Cloud (Upload activity sessions)
        if let Err(e) = self.upload_activity_sessions().await {
            if e.contains("401") || e.contains("Unauthorized") {
                if let Ok(refresh_token) = auth_state.get_refresh_token() {
                    if let Ok(new_access_token) = self.client.refresh_token(&refresh_token).await {
                        let _ = auth_state.set_access_token(&new_access_token);
                        // Retry upload once
                        self.upload_activity_sessions().await?;
                    } else {
                        return Err(e);
                    }
                } else {
                    return Err(e);
                }
            } else {
                return Err(e);
            }
        }

        // Cloud -> Local (Download settings & tombstones)
        if let Err(e) = self.download_updates(&device_id).await {
            if e.contains("401") || e.contains("Unauthorized") {
                if let Ok(refresh_token) = auth_state.get_refresh_token() {
                    if let Ok(new_access_token) = self.client.refresh_token(&refresh_token).await {
                        let _ = auth_state.set_access_token(&new_access_token);
                        self.download_updates(&device_id).await?;
                    } else {
                        return Err(e);
                    }
                } else {
                    return Err(e);
                }
            } else {
                return Err(e);
            }
        }

        Ok(true)
    }

    async fn download_updates(&self, device_id: &str) -> Result<(), String> {
        let db_path = self.db_path.clone();
        let device_id_str = device_id.to_string();
        
        let (last_sync_at, last_sync_id) = tokio::task::spawn_blocking(move || -> Result<(Option<String>, Option<String>), rusqlite::Error> {
            let conn = connection::open_database(&db_path)?;
            let mut stmt = conn.prepare("SELECT last_sync_at, last_sync_id FROM devices WHERE id = ?1")?;
            stmt.query_row([&device_id_str], |r| {
                Ok((r.get(0)?, r.get(1)?))
            }).optional().map(|r| r.unwrap_or((None, None)))
        }).await.map_err(|e| e.to_string())?.map_err(|e| e.to_string())?;

        let mut max_updated_at: Option<String> = None;
        let mut max_updated_id: Option<String> = None;

        // 1. Fetch settings
        if let Ok(settings) = self.client.get_settings(last_sync_at.as_deref(), last_sync_id.as_deref()).await {
            let db_path = self.db_path.clone();
            let settings_clone = settings.clone();
            tokio::task::spawn_blocking(move || {
                if let Ok(conn) = connection::open_database(&db_path) {
                    for setting in settings_clone {
                        if let (Some(id), Some(version), Some(mode), Some(payload)) = (
                            setting.get("id").and_then(|v| v.as_str()),
                            setting.get("version").and_then(|v| v.as_i64()),
                            setting.get("mode").and_then(|v| v.as_str()),
                            setting.get("payload")
                        ) {
                            let payload_str = serde_json::to_string(payload).unwrap_or_default();
                            let local_version: Option<i64> = conn.query_row(
                                "SELECT version FROM settings WHERE id = ?1", 
                                [id], 
                                |r| r.get(0)
                            ).optional().unwrap_or(None);

                            if local_version.is_none() || local_version.unwrap() < version {
                                conn.execute(
                                    "INSERT INTO settings (id, version, mode, payload, created_at, updated_at) 
                                     VALUES (?1, ?2, ?3, ?4, datetime('now'), datetime('now'))
                                     ON CONFLICT (id) DO UPDATE SET version=excluded.version, mode=excluded.mode, payload=excluded.payload, updated_at=datetime('now')",
                                    rusqlite::params![id, version, mode, payload_str]
                                ).unwrap_or_default();
                            }
                        }
                    }
                }
            }).await.map_err(|e| e.to_string())?;

            // Update max cursor
            for setting in &settings {
                if let (Some(updated_at), Some(id)) = (setting.get("updated_at").and_then(|v| v.as_str()), setting.get("id").and_then(|v| v.as_str())) {
                    if max_updated_at.is_none() || updated_at > max_updated_at.as_deref().unwrap() {
                        max_updated_at = Some(updated_at.to_string());
                        max_updated_id = Some(id.to_string());
                    } else if updated_at == max_updated_at.as_deref().unwrap() && (max_updated_id.is_none() || id > max_updated_id.as_deref().unwrap()) {
                        max_updated_id = Some(id.to_string());
                    }
                }
            }
        }

        // 2. Fetch tombstones
        if let Ok(tombstones) = self.client.get_tombstones(last_sync_at.as_deref(), last_sync_id.as_deref()).await {
            let db_path = self.db_path.clone();
            let tombstones_clone = tombstones.clone();
            tokio::task::spawn_blocking(move || {
                if let Ok(conn) = connection::open_database(&db_path) {
                    for tombstone in tombstones_clone {
                        if let (Some(target_id), Some(table_name)) = (
                            tombstone.get("target_record_id").and_then(|v| v.as_str()),
                            tombstone.get("table_name").and_then(|v| v.as_str())
                        ) {
                            match table_name {
                                "activity_sessions" => {
                                    conn.execute("DELETE FROM activity_sessions WHERE id = ?1", [target_id]).unwrap_or_default();
                                }
                                "settings" => {
                                    conn.execute("DELETE FROM settings WHERE id = ?1", [target_id]).unwrap_or_default();
                                }
                                _ => {
                                    // Ignore unknown table targets for security
                                }
                            }
                        }
                    }
                }
            }).await.map_err(|e| e.to_string())?;

            for tombstone in &tombstones {
                if let (Some(updated_at), Some(id)) = (tombstone.get("updated_at").and_then(|v| v.as_str()), tombstone.get("id").and_then(|v| v.as_str())) {
                    if max_updated_at.is_none() || updated_at > max_updated_at.as_deref().unwrap() {
                        max_updated_at = Some(updated_at.to_string());
                        max_updated_id = Some(id.to_string());
                    } else if updated_at == max_updated_at.as_deref().unwrap() && (max_updated_id.is_none() || id > max_updated_id.as_deref().unwrap()) {
                        max_updated_id = Some(id.to_string());
                    }
                }
            }
        }

        // 3. Update cursor
        if let Some(new_sync_time) = max_updated_at {
            let db_path = self.db_path.clone();
            let device_id_str = device_id.to_string();
            let new_sync_id = max_updated_id.unwrap_or_default();
            tokio::task::spawn_blocking(move || {
                if let Ok(conn) = connection::open_database(&db_path) {
                    conn.execute("UPDATE devices SET last_sync_at = ?1, last_sync_id = ?2 WHERE id = ?3", [new_sync_time, new_sync_id, device_id_str]).unwrap_or_default();
                }
            }).await.map_err(|e| e.to_string())?;
        }

        Ok(())
    }

    async fn upload_activity_sessions(&self) -> Result<(), String> {
        let db_path = self.db_path.clone();
        
        let fetch_result = tokio::task::spawn_blocking(move || -> Result<(Vec<Value>, Vec<String>), String> {
            let conn = connection::open_database(&db_path)
                .map_err(|e| format!("Failed to open DB: {:?}", e))?;
                
            let mut stmt = conn.prepare(
                "SELECT id, record_id, attempt_count FROM sync_queue 
                 WHERE state = 'pending' AND record_type = 'activity_sessions' 
                 AND (next_attempt_at IS NULL OR next_attempt_at <= datetime('now'))
                 LIMIT 50"
            ).map_err(|e| e.to_string())?;

            let rows = stmt.query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i32>(2)?,
                ))
            }).map_err(|e| e.to_string())?;

            let mut batch = Vec::new();
            let mut queue_ids = Vec::new();

            for (q_id, r_id, _attempt) in rows.flatten() {
                let session: Option<Value> = conn.query_row(
                    "SELECT id, user_id, device_id, source, application_name, title, started_at, ended_at, duration_ms, metadata, created_at 
                     FROM activity_sessions WHERE id = ?1",
                    [&r_id],
                    |r| {
                        Ok(json!({
                            "id": r.get::<_, String>(0)?,
                            "user_id": r.get::<_, Option<String>>(1)?,
                            "device_id": r.get::<_, String>(2)?,
                            "source": r.get::<_, String>(3)?,
                            "application_name": r.get::<_, Option<String>>(4)?,
                            "title": r.get::<_, Option<String>>(5)?,
                            "started_at": r.get::<_, String>(6)?,
                            "ended_at": r.get::<_, String>(7)?,
                            "duration_ms": r.get::<_, i64>(8)?,
                            "metadata": r.get::<_, Option<String>>(9)?.map(|m: String| serde_json::from_str::<Value>(&m).unwrap_or(Value::Null)),
                            "created_at": r.get::<_, String>(10)?,
                        }))
                    }
                ).optional().unwrap_or(None);

                if let Some(s) = session {
                    batch.push(s);
                    queue_ids.push(q_id);
                }
            }
            Ok((batch, queue_ids))
        }).await.map_err(|e| e.to_string())?;

        let (batch, queue_ids) = fetch_result?;

        if batch.is_empty() {
            return Ok(());
        }

        // Upload batch via HTTP (async)
        let upload_result = self.client.upsert_activity_sessions(batch).await;
        
        let db_path = self.db_path.clone();
        tokio::task::spawn_blocking(move || {
            if let Ok(conn) = connection::open_database(&db_path) {
                match upload_result {
                    Ok(_) => {
                        for q_id in queue_ids {
                            conn.execute("DELETE FROM sync_queue WHERE id = ?1", [&q_id]).unwrap_or_default();
                        }
                    }
                    Err(e) => {
                        let now = Utc::now();
                        for q_id in queue_ids {
                            let attempt: i32 = conn.query_row("SELECT attempt_count FROM sync_queue WHERE id = ?1", [&q_id], |r| r.get(0)).unwrap_or(0);
                            let backoff_secs = std::cmp::min(30 * (2i64.pow(attempt as u32)), 3600);
                            let next_attempt = now + chrono::Duration::seconds(backoff_secs);
                            conn.execute(
                                "UPDATE sync_queue SET attempt_count = attempt_count + 1, next_attempt_at = ?1, last_error = ?2 WHERE id = ?3",
                                rusqlite::params![next_attempt.to_rfc3339(), e, q_id]
                            ).unwrap_or_default();
                        }
                    }
                }
            }
        }).await.map_err(|e| e.to_string())?;
        
        Ok(())
    }
}
