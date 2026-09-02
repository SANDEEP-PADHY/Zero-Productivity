# Privacy

## Data minimization
Collect only data needed to identify and measure activity. Do not collect keystroke content, passwords, clipboard data, screenshots, or document contents by default.

## Local-first
Raw observations primarily remain local. Upload resolved sessions and necessary metadata.

## Local Data Retention
To prevent unbounded local history storage, explicit retention limits are applied: `observations` (default 7 days), `activity_sessions` (default 90 days). Unsynced data is never deleted by retention policies.

## URLs
Full URLs can reveal sensitive information. Support domain-only mode and exclusions. Avoid unnecessary display of full URLs.

## User controls
Pause tracking, exclude apps/sites, configure capture detail, delete activity, revoke devices, configure sync.

## Deletion and Tombstones
Deletion should cover central records and local data where applicable and must not retain deleted content merely for auditability.
A bidirectional Tombstone mechanism ensures that sessions deleted on the server or another device synchronize downwards to all offline devices, securely purging the records instead of inadvertently re-uploading them.
