# Android Specification

Initial scope: application identity only. 

Do not attempt to reuse the desktop Chromium WebExtension architecture on Android. Browser URL/tab tracking on Android is a future feature requiring a separate Android-specific architecture and appropriate permissions (e.g., Accessibility Services).

Stack: Kotlin + Jetpack Compose.

Flow: Android signals -> collector -> resolver -> SQLite -> sync queue -> Supabase.

Respect Android background/usage-access restrictions and explain tracking permissions clearly.
