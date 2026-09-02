# Android Specification

Initial scope: application identity only. Browser URL/tab tracking is out of scope for v1.

Stack: Kotlin + Jetpack Compose.

Flow: Android signals -> collector -> resolver -> SQLite -> sync queue -> Supabase.

Respect Android background/usage-access restrictions and explain tracking permissions clearly.
