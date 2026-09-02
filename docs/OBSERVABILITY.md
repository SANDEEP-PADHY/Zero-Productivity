# Observability

## Local logs
Collector lifecycle, state transitions, session boundaries, sync attempts/results, permission failures, recovery events. Avoid sensitive URLs/titles unless diagnostics explicitly allow them.

## Metrics
`sessions_created`, `sessions_closed`, `recovery_events`, `sync_batches`, `sync_failures`, `sync_latency`, `queued_records`, `collector_errors`.

## Backend
Monitor API errors, DB connections/size, auth failures, sync throughput, RLS failures, and Realtime usage if enabled.
