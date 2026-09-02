# Browser Extension Specification

Chromium first; Firefox later.

Track active tab, foreground browser window, navigation, URL/domain, title, supported media state.

Architecture: `Extension <-> Native Messaging <-> Desktop Agent <-> SQLite`.

Only the active tab in the foreground browser counts. Request minimum permissions.
