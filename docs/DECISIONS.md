# Architecture Decisions

| ID | Decision | Reason |
|---|---|---|
| ADR-001 | One Git monorepo | Shared core, docs, sync, coordinated releases |
| ADR-002 | SQLite locally | Reliable offline source of truth |
| ADR-003 | Self-hosted Supabase initially | Control, cost, home-lab architecture |
| ADR-004 | Event-driven collection | Precision with lower overhead |
| ADR-005 | Foreground-only desktop | Measures actual active context |
| ADR-006 | Media separate from interaction | Media can remain active while input is low |
| ADR-007 | Idle is a state | Preserve transparent timeline |
| ADR-008 | Device-specific raw sessions | Avoid corrupt cross-device ingestion |
| ADR-009 | User-controlled classification | Workflows differ by user |
| ADR-010 | Stitch + Antigravity | Separate visual design from implementation |
| ADR-011 | Bidirectional Downstream Sync | Support settings, rules, and tombstones propagating to offline devices |
| ADR-012 | Deletion Tombstones | Prevent offline devices from re-uploading deleted records |
| ADR-013 | Explicit Local Data Retention | Prevent unbounded SQLite growth (7 days obs, 90 days sessions), except for unsynced data |
| ADR-014 | Recovery Heartbeat | ~60s heartbeat bounds crash recovery without replacing event-driven architecture |
| ADR-015 | Continuous Midnight Sessions | Sessions represent continuous activity and are split at query time by Analytics, not during tracking |
| ADR-016 | Offline Authentication Resilience | Local tracking and queueing continues without data loss if token expires offline |
| ADR-017 | Deterministic Settings Conflict Resolution | Last-Write-Wins (LWW) based on server timestamps resolves simultaneous offline edits |
| ADR-018 | Linux X11 Support First | Avoid making complex Wayland compositor differences a blocker for the Windows MVP |
| ADR-019 | Android v1 Application Identity Only | Android browser URL tracking requires a separate architecture (Accessibility Services) not suited for v1 |
| ADR-020 | Strict Device-Specific Truth | Never silently deduplicate or merge raw sessions during ingestion; analytics handles overlaps |
