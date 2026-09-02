# Data Model

## Time
Store UTC wall-clock timestamps. Use monotonic clocks locally for duration arithmetic. Internal precision may be milliseconds. Sessions crossing midnight or other calendar boundaries must store precise UTC start and end bounds without splitting.

## Main entities

### devices
`id, user_id, device_name, platform, os_version, app_version, first_registered_at, last_sync_at, last_activity_sync_at, settings_mode, revoked_at, created_at, updated_at`

### activity_sessions
`id, user_id, device_id, source, activity_type, application_id, application_name, browser_name, domain, url, title, started_at, ended_at, duration_ms, foreground_ms, interaction_ms, media_ms, idle_ms, classification, confidence, metadata, created_at`

### observations (local-first)
`id, device_id, observed_at_utc, monotonic_ms, source, signal_type, payload, confidence`

### sync_queue
`id, device_id, record_type, record_id, payload_hash, state, attempt_count, next_attempt_at, last_error, created_at, updated_at`

### settings
`id, user_id, device_id nullable, version, mode, payload, created_at, updated_at`

### rules
`id, user_id, scope, priority, enabled, match_type, match_value, classification, action, created_at, updated_at, version`

### tombstones
`id, target_record_id, user_id, device_id nullable, deletion_timestamp, schema_version, sync_state, created_at`
Records deleted locally or remotely generate a tombstone to ensure offline devices drop the deleted record instead of re-uploading it.

## IDs
Use globally unique session IDs. Prefer UUIDv7 or equivalent.

## Local Retention Policies
Explicit retention policies govern the local SQLite database to prevent unbounded growth:
- **`observations`**: Default 7 days.
- **`sync_queue`**: Remove immediately after successful sync acknowledgment.
- **`activity_sessions`**: Default 90 days.
- **`tombstones`**: Retained long enough to ensure sync across offline devices (e.g., 90 days).

**Critical Rule:** Retention must never delete records that are still required for reliable synchronization. If a record has not successfully synchronized, it must not be deleted solely because its normal retention period expired.
