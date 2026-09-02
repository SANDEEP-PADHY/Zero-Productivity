# Tracking Engine

## State model
```text
UNKNOWN -> ACTIVE <-> IDLE
ACTIVE -> ACTIVE_MEDIA (media is orthogonal, not a replacement state)
ACTIVE/IDLE -> STOPPED on lock/sleep/logout
ANY -> RECOVERY after crash/restart
```

## Foreground
Only the active foreground window counts. Background windows do not accumulate foreground time.

## Browser
When a browser owns foreground, only its active tab counts. Tab activation/navigation creates a new resolved activity boundary.

Example: ChatGPT active while YouTube is a background tab => ChatGPT counts.

## Media
Media time is measured independently. A foreground movie playing for 120 minutes may produce 120 minutes foreground/media time but only 8 minutes interaction.

## Idle
Default threshold: 2 minutes; configurable. Idle remains part of the timeline.

## Lock/sleep
Close/stop current foreground session. After wake/unlock start a new session.

## Collection
Prefer OS/browser events. Use low-frequency reconciliation only for recovery/safety.

## Crash recovery
Never blindly extend an open session. Close at the last trustworthy observation/system boundary and start a fresh session after recovery.

## Confidence
Maintain an internal confidence score based on direct events, inferred reconciliation, and uncertain recovery boundaries.
