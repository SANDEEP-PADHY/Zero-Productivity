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

## Session Continuity (Midnight)
A session represents continuous activity and is never forced to split at midnight or day boundaries. For example, continuous activity from 23:30 to 01:30 remains a single session record. Analytics layers split durations at query time.

## Lock/sleep
Close/stop current foreground session. After wake/unlock start a new session.

## Collection
Architecture must remain event-driven. Do NOT replace event-driven tracking with one-second polling.

## Crash recovery
Never blindly extend an open session. Close at the last trustworthy observation/system boundary and start a fresh session after recovery.

To bound uncertainty when a process crashes while an activity remains unchanged, a low-frequency recovery heartbeat (approximately 60 seconds) is used.
- **Purpose**: Strictly for recovery bounds, not for active polling.
- **Records**: A heartbeat event in the `observations` table verifying the current state is unchanged.
- **Recovery**: If a crash occurs, the session is safely bounded up to the last heartbeat. Missing heartbeats signify the process stopped tracking and the session ended.

## Confidence
Maintain an internal confidence score based on direct events, inferred reconciliation, and uncertain recovery boundaries.
