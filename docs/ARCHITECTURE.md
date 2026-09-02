# Architecture

```text
Windows/Linux ─┐
Browser Ext. ──┼─> Collector -> Resolver -> SQLite -> Sync Queue -> HTTPS -> Self-hosted Supabase
Android ───────┘                                               │
                                                               ├─ PostgreSQL
                                                               ├─ Auth
                                                               └─ Realtime (optional)
                                                                        │
                                                                  Next.js Dashboard
```

## Components
- **Collectors:** platform-specific signals.
- **Resolver:** normalizes raw signals into activities.
- **Timeline engine:** opens/closes sessions.
- **Rules engine:** exclusions, normalization, semantic activity detection, classification.
- **SQLite:** device-local source of truth.
- **Sync engine:** reliable batched delivery.
- **Supabase:** central auth/API/database layer.
- **Dashboard:** analytics and configuration.

## Stack
Rust + Tauri; Kotlin + Jetpack Compose; TypeScript WebExtensions; Next.js + TypeScript; SQLite; PostgreSQL; self-hosted Supabase; Docker.

## Portability
Client contracts must not depend on the physical Supabase host so migration from home-lab Supabase to Supabase Cloud is possible without changing collectors.
