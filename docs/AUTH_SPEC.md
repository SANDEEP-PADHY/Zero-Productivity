# Authentication

Use Supabase Auth with Google OAuth.

Flow: `Client -> Supabase Auth -> Google -> authenticated session -> API`.

Requirements: secure token storage, refresh handling, logout, reauthentication, account deletion, revoked-session handling.

Authentication identifies the user; RLS authorizes database access. Never trust a client-provided user ID for authorization.

After login, register/load device identity, fetch effective settings, and begin synchronization.
