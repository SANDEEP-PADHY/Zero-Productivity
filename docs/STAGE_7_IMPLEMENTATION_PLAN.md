# Stage 7 Implementation Plan: Dashboard & Authentication

## Phase 1: Audit Report

### 1. Current Dashboard Implementation Status
The `apps/dashboard` directory is entirely empty except for a `.gitkeep` file. No Next.js application, configuration, or routing exists yet.

### 2. Current Supabase API Contract
The backend runs a local/self-hosted Supabase instance.
Exposed tables: `devices`, `activity_sessions`, `settings`, `tombstones`.
All tables are accessible via standard PostgREST endpoints under `/rest/v1/`.

### 3. Current Authentication Implementation
Authentication on the desktop client exists as a token storage wrapper (`keyring`) awaiting an actual JWT. No frontend OAuth UI is implemented.

### 4. Current Database Schema
- **devices**: `id` (UUID), `user_id` (UUID), `device_name`, `platform`, `os_version`, `app_version`, `first_registered_at`, `last_sync_at`, `last_activity_sync_at`, `settings_mode`, `revoked_at`.
- **activity_sessions**: `id` (UUID), `user_id` (UUID), `device_id` (UUID), `source`, `activity_type`, `application_id`, `application_name`, `browser_name`, `domain`, `url`, `title`, `started_at`, `ended_at`, `duration_ms`, `foreground_ms`, `interaction_ms`, `media_ms`, `idle_ms`, `classification`, `confidence`, `metadata`.
- **settings**: `id` (UUID), `user_id` (UUID), `device_id` (UUID), `version`, `mode`, `payload`.
- **tombstones**: `id` (UUID), `target_record_id` (UUID), `user_id` (UUID), `device_id` (UUID), `deletion_timestamp`, `schema_version`, `sync_state`.

### 5. Current RLS Policies
Every table enforce strict multi-tenant isolation via: `auth.uid() = user_id`.

### 6. Existing Environment Variables
Stored in `.env.example`:
- `SUPABASE_URL`
- `SUPABASE_ANON_KEY`
- `SUPABASE_SERVICE_ROLE_KEY`
- `GOOGLE_OAUTH_CLIENT_ID`
- `GOOGLE_OAUTH_CLIENT_SECRET`
- `DATABASE_URL`

### 7. Existing Next.js Configuration
None.

### 8. Conflicts between Specifications
- **DASHBOARD_SPEC routes vs USER_REQUEST routes**: The prompt specifically requests `/login`, `/dashboard`, `/activity`, `/analytics`, `/devices`, `/settings`. I will merge these seamlessly. `/dashboard` maps to the Overview. `/activity` maps to the Timeline.
- **Tombstone target_table vs initial schema**: In Stage 6 we validated tombstones against a `table_name`, but `table_name` is inexplicably absent from the `tombstones` schema in `20240903000000_initial_schema.sql`. (Wait, let's address this during the phase 4 typed schema generation; it doesn't block the UI).

---

## Phase 2: Architecture & Implementation Plan

### Architecture
- **Framework**: Next.js 14 (App Router) + TypeScript.
- **Styling**: Tailwind CSS (adhering strictly to UI_SPEC.md neo-brutalism).
- **Client Library**: `@supabase/ssr` to securely handle SSR and browser auth cookies.
- **State**: React Server Components for initial fetches + client-side state where interactive filters are required.

### Authentication Flow (Phase 2)
- **Supabase SSR**: The `middleware.ts` will refresh sessions seamlessly.
- **Login**: Unauthenticated users visiting protected routes are redirected to `/login`.
- **OAuth**: Google OAuth triggered via `supabase.auth.signInWithOAuth({ provider: 'google' })`.
- **Tokens**: Tokens remain entirely inside the Supabase cookies. We do not expose or parse them manually.

### Routes & Pages (Phase 3)
1. `/login`: Clean neo-brutalist login surface.
2. `/dashboard`: Overview metrics (today's time, distribution, top apps/sites).
3. `/activity`: Chronological timeline of raw `activity_sessions`. Filters for devices, apps, and dates.
4. `/analytics`: High-level aggregated views (week-over-week, classification grouping).
5. `/devices`: List of registered devices pulling directly from the `devices` table. Actions to rename/revoke.
6. `/settings`: Account-wide rules and configuration. (Settings are JSONB payloads).

### Data Fetching & Authorization (Phase 4)
- **Server Actions / SSR**: All major data fetching uses Supabase's `createServerClient`.
- **Types**: We will generate and export TS interfaces representing the exact schema. No duplicative mapping layer.
- **RLS**: The generated `supabase` client automatically attaches the cookie session, making RLS mathematically foolproof.

### UI & Styling (Phase 5)
- Follow `UI_SPEC.md` colors (`#080808`, `#3D3D3D`, `#FFFFFF`, `#FFECD1`, `#15616D`).
- Clean typography and strong borders.

### Error, Loading & Empty States (Phase 6)
- Wrap all page components in standard Next.js `loading.tsx` and `error.tsx` boundaries.
- Render explicit Empty State cards (e.g., "No devices found", "No activity tracked today").

### Implementation Sequence
1. Initialize Next.js project with Tailwind.
2. Setup Supabase SSR and `middleware.ts` authentication guard.
3. Build `/login` and test OAuth callback.
4. Define TypeScript interfaces for the database schema.
5. Implement Layout and Navigation Shell.
6. Build `/dashboard` (Overview) page.
7. Build `/activity` (Timeline) page.
8. Build `/analytics` page.
9. Build `/devices` page.
10. Build `/settings` page.
11. Security Audit and Testing.
12. Final Report.
