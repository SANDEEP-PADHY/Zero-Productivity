# Authentication

Use Supabase Auth with Google OAuth.

Flow: `Client -> Supabase Auth -> Google -> authenticated session -> API`.

Requirements: secure token storage, refresh handling, logout, reauthentication, account deletion, revoked-session handling.

Authentication identifies the user; RLS authorizes database access. Never trust a client-provided user ID for authorization.

After login, register/load device identity, fetch effective settings, and begin synchronization.

## Offline Authentication Resilience
If authentication expires while the device is offline, the client must:
- Continue local tracking and writing to SQLite.
- Continue queuing synchronization records.
- Not lose any activity data.
- Not repeatedly fail network requests unnecessarily (pause upward sync attempts).

When authentication becomes available (network restored):
- Reauthenticate or refresh the token.
- Resume synchronization and upload queued records.

Tokens must be stored securely using OS-level secure storage (e.g., Windows Credential Manager) to prevent unauthorized offline extraction.
