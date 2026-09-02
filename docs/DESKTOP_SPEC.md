# Desktop Specification

## Platforms
Windows first; Linux second. For the first Linux release, officially support X11. Do not attempt to solve all Wayland compositor differences in the first implementation; document Wayland as a future compatibility project so it does not block the MVP.

## Stack
Rust + Tauri.

## Responsibilities
Foreground window, input/idle, media state where available, lock/sleep/resume, SQLite, rules, timeline, sync, settings, device registration.

## UI
Lightweight companion UI: current activity, tracking state, pause/resume, sync status, privacy, settings. Analytics live primarily in the web dashboard.
