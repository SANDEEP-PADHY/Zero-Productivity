# Sync Protocol

## Pipeline
`SQLite finalized records -> sync queue -> batch builder -> authenticated HTTPS -> Supabase -> PostgreSQL`

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

## Multi-device
Never merge overlapping device sessions during ingestion. Preserve raw device-specific truth and derive aggregate human-time later.
