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
