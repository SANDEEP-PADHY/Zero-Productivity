# Testing Strategy

## Unit
Timestamp arithmetic, session boundaries, idle/media transitions, rule precedence, settings precedence, sync idempotency.

## Integration
OS/browser event -> session; SQLite -> queue; queue -> API; API -> PostgreSQL; auth -> RLS.

## Failure tests
Network loss, crash, sleep, lock/unlock, clock changes, duplicate upload, revoked device, malformed payload, schema mismatch.

## Acceptance example
09:00 ChatGPT; 09:05 YouTube video; 09:20 Shorts; 09:25 idle; 09:30 VS Code. Expected exact boundaries, semantic separation, correct idle state, and new VS Code session.
