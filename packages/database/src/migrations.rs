use rusqlite::{Connection, Result};

pub fn run_migrations(conn: &mut Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
        [],
    )?;

    let current_version: i32 = conn.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
        [],
        |row| row.get(0),
    )?;

    let migrations = get_migrations();

    for (version, sql) in migrations {
        if version > current_version {
            let tx = conn.transaction()?;
            tx.execute_batch(sql)?;
            tx.execute(
                "INSERT INTO schema_migrations (version) VALUES (?)",
                [version],
            )?;
            tx.commit()?;
        }
    }

    Ok(())
}

fn get_migrations() -> Vec<(i32, &'static str)> {
    vec![
        (1, MIGRATION_V1),
        (2, MIGRATION_V2),
        (3, MIGRATION_V3),
    ]
}

const MIGRATION_V1: &str = r#"
CREATE TABLE devices (
    id TEXT PRIMARY KEY,
    user_id TEXT,
    device_name TEXT,
    platform TEXT,
    os_version TEXT,
    app_version TEXT,
    first_registered_at TEXT,
    last_sync_at TEXT,
    last_activity_sync_at TEXT,
    settings_mode TEXT,
    revoked_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE activity_sessions (
    id TEXT PRIMARY KEY,
    user_id TEXT,
    device_id TEXT NOT NULL,
    source TEXT NOT NULL,
    activity_type TEXT,
    application_id TEXT,
    application_name TEXT,
    browser_name TEXT,
    domain TEXT,
    url TEXT,
    title TEXT,
    started_at TEXT NOT NULL,
    ended_at TEXT NOT NULL,
    duration_ms INTEGER NOT NULL,
    foreground_ms INTEGER,
    interaction_ms INTEGER,
    media_ms INTEGER,
    idle_ms INTEGER,
    classification TEXT,
    confidence REAL,
    metadata TEXT,
    created_at TEXT NOT NULL,
    FOREIGN KEY(device_id) REFERENCES devices(id) ON DELETE CASCADE,
    UNIQUE(device_id, id)
);

CREATE TABLE observations (
    id TEXT PRIMARY KEY,
    device_id TEXT NOT NULL,
    observed_at_utc TEXT NOT NULL,
    monotonic_ms INTEGER NOT NULL,
    source TEXT NOT NULL,
    signal_type TEXT NOT NULL,
    payload TEXT,
    confidence REAL,
    FOREIGN KEY(device_id) REFERENCES devices(id) ON DELETE CASCADE
);

CREATE TABLE sync_queue (
    id TEXT PRIMARY KEY,
    device_id TEXT NOT NULL,
    record_type TEXT NOT NULL,
    record_id TEXT NOT NULL,
    payload_hash TEXT,
    state TEXT NOT NULL,
    attempt_count INTEGER NOT NULL DEFAULT 0,
    next_attempt_at TEXT,
    last_error TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY(device_id) REFERENCES devices(id) ON DELETE CASCADE
);

CREATE TABLE tombstones (
    id TEXT PRIMARY KEY,
    target_record_id TEXT NOT NULL,
    user_id TEXT,
    device_id TEXT,
    deletion_timestamp TEXT NOT NULL,
    schema_version INTEGER,
    sync_state TEXT NOT NULL,
    created_at TEXT NOT NULL
);
"#;

const MIGRATION_V2: &str = r#"
ALTER TABLE devices ADD COLUMN last_sync_id TEXT;
"#;

const MIGRATION_V3: &str = r#"
CREATE TABLE rules (
    id TEXT PRIMARY KEY,
    user_id TEXT,
    scope TEXT NOT NULL,
    priority INTEGER NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT 1,
    match_field TEXT NOT NULL,
    match_type TEXT NOT NULL,
    match_value TEXT NOT NULL,
    classification TEXT NOT NULL,
    activity_type TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 1
);
"#;
