# Device Management

Each installation has a unique device ID.

Display: name, platform/OS, app version, first registered, last sync, last activity sync, settings mode.

Actions: rename, revoke, inspect, change settings mode.

Do not claim online status from last sync alone. A future Realtime presence signal may be added if its semantics are reliable.

Revoked devices must stop authenticated synchronization.
