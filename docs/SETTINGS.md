# Settings

## Precedence
`Device override -> Account default -> Built-in default`

Account settings are defaults for all devices. A device can enable **Use device-specific settings** and override them.

## Tracking
Desktop apps, browser tabs, websites/domains, full URL/path, window titles, idle detection/threshold, foreground-only, foreground idle counting, background media, media playback, excluded apps/sites.

## Sync
Immediate, 5m, 15m, 30m, 1h, manual; sync-on-close/shutdown/sleep/network restoration.

## Privacy
URL granularity, site/app exclusions, media metadata, background media.

## UI
Every setting should clearly show `Inherited from account` or `Device-specific`.

## Versioning & Conflict Resolution
All settings are versioned so session interpretation remains deterministic.

**Conflict Strategy (Simultaneous Offline Edits):**
Use a deterministic Last-Write-Wins (LWW) strategy based on server-authoritative `updated_at` timestamps or version numbers.
When offline edits synchronize:
- The server timestamp determines the winner for account settings.
- Device-specific overrides continue to apply to their respective devices, regardless of account setting updates.
