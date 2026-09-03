# Stage 7 Manual Test Checklist

Because the local execution environment lacks Docker, the live Supabase stack (including GoTrue/Auth and PostgreSQL) could not be launched. The following manual tests must be performed by a human operator in a fully equipped environment.

- [ ] Login
- [ ] Google OAuth
- [ ] Logout
- [ ] Protected route
- [ ] Dashboard
- [ ] Activity
- [ ] Analytics
- [ ] Devices
- [ ] Settings
- [ ] Empty states
- [ ] Error states
- [ ] Multi-device
- [ ] RLS User A
- [ ] RLS User B
- [ ] Unauthorized API request
- [ ] Session expiry
- [ ] Network failure
- [ ] Mobile/responsive layout
