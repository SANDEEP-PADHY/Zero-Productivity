# Data Model

## Time
Store UTC wall-clock timestamps. Use monotonic clocks locally for duration arithmetic. Internal precision may be milliseconds.

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
`id, user_id, scope, priority, enabled, match_type, match_value, classification, action, created_at, updated_at`

Use globally unique session IDs. Prefer UUIDv7 or equivalent.
