# Security

## Threats
Token theft, compromised device, account takeover, exposed backend, malicious browser content, local DB theft, replayed sync, cross-user access.

## Controls
- Supabase Auth + Google OAuth.
- PostgreSQL RLS on user-owned data.
- HTTPS only.
- No public PostgreSQL.
- Revocable device identity.
- Least-privilege service credentials.
- Secrets excluded from Git.
- Idempotent sync.
- Minimum browser permissions.

Service-role credentials are server-only and must never ship in clients.
