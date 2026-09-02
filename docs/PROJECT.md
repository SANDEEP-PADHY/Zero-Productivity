# Project Specification

## Purpose
A local-first cross-platform activity tracker that records the user's active digital context with precise session boundaries, separates foreground/interaction/media/idle time, synchronizes finalized sessions, and provides configurable analytics.

## Platforms
- Windows desktop (first)
- Linux desktop
- Android
- Chromium browser extension (Firefox later)
- Web dashboard

## Core principles
1. Local-first and offline-capable.
2. Event-driven collection instead of one-second polling.
3. Foreground context is the primary activity signal.
4. Media state is separate from interaction state.
5. Idle is represented, not silently deleted.
6. User rules override defaults.
7. Sync is batched and idempotent.
8. Device truth is preserved; cross-device aggregates are derived.
9. Privacy and data minimization are first-class.
10. Documentation is the implementation source of truth.

## v1 non-goals
No invasive keylogging, screenshot surveillance by default, clipboard/password capture, public PostgreSQL exposure, or whole SQLite database synchronization.
