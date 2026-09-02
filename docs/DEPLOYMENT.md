# Deployment

Initial backend is self-hosted Supabase on the home lab using Docker.

Put a reverse proxy/secure tunnel in front of required public HTTPS endpoints. Never expose PostgreSQL directly to the internet.

Back up PostgreSQL and test restoration. Maintain development/staging/production as the project matures. Keep deployment portable for later Supabase Cloud migration.
