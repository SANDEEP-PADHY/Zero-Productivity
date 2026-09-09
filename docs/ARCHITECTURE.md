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
- **Collectors:** platform-specific signals (raw executable, path, titles).
- **Tracking Engine:** converts raw events into exactly-bounded temporal sessions.
- **Resolver:** maps sessions to normalized deterministic identities.
  - **Identity vs Classification:** The Resolver answers "What is this?" (App identity, Domain). It strictly does NOT answer "How is this categorized?" (Productivity scoring).
  - **Application Identity:** Stable, deterministic UUIDs derived from stripped/lowercased executable names. Window titles are metadata, not identity.
  - **Website Identity:** Primary identity is the normalized domain. URL and page titles remain optional metadata.
- **Rules engine:** semantic activity detection, classification.
- **SQLite:** device-local source of truth.
- **Sync engine:** reliable batched delivery.
- **Supabase:** central auth/API/database layer.
- **Dashboard:** analytics and configuration.

## Stack
Rust + Tauri; Kotlin + Jetpack Compose; TypeScript WebExtensions; Next.js + TypeScript; SQLite; PostgreSQL; self-hosted Supabase; Docker.

## Portability
Client contracts must not depend on the physical Supabase host so migration from home-lab Supabase to Supabase Cloud is possible without changing collectors.
