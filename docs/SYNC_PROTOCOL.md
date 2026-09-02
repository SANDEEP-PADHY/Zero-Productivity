# Sync Protocol

## Pipeline

The system requires bidirectional synchronization.

**Upstream:**
`SQLite finalized records -> sync queue -> batch builder -> authenticated HTTPS -> Supabase -> PostgreSQL`

**Downstream:**
`Supabase (PostgreSQL) -> HTTPS -> Device Sync -> Local SQLite`

## Default cadence
15 minutes, configurable to immediate/5m/15m/30m/1h/manual. Also flush on close, shutdown, sleep, network restoration, and explicit sync when feasible.

## Batch contract
```json
{
  "schema_version": 1,
  "batch_id": "uuid",
  "device_id": "uuid",
  "sessions": []
}
```

## Idempotency
Retries must not duplicate records. Enforce a unique session identity such as `(device_id, session_id)` or a globally unique session ID.

## Queue states
`pending -> uploading -> acknowledged` with `failed/dead_letter` paths. Use exponential backoff for temporary failures.

## Downstream Synchronization
Downstream synchronization pushes changes from the server to local devices. This includes:
- Account settings
- Device settings
- Rules
- Deletion tombstones
- Future server-originated configuration changes

**Cursor Mechanism:**
Clients use a `last_synced_at` (server timestamp) or monotonically increasing `version` cursor. The client includes this cursor in downstream requests to determine what changes it has already received, what changes are pending, and whether a server change is newer than its local version.

## Multi-device
Never merge overlapping device sessions during ingestion. Preserve raw device-specific truth and derive aggregate human-time later.
