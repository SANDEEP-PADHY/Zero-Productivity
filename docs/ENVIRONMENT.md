# Environment

Never commit real secrets. Use `.env`/`.env.local`; provide `.env.example`.

Typical server variables:
```text
SUPABASE_URL=
SUPABASE_ANON_KEY=
SUPABASE_SERVICE_ROLE_KEY=
GOOGLE_OAUTH_CLIENT_ID=
GOOGLE_OAUTH_CLIENT_SECRET=
DATABASE_URL=
```

Service-role and OAuth secrets are server-only.
