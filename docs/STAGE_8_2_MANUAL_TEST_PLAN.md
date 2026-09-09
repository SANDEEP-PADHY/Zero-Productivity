# Stage 8.2 Manual Test Plan

This document outlines the required manual validation steps for the Windows Tracking Reliability improvements. Because system-level events like sleeping, locking, and crashing cannot be fully reproduced in unit tests, manual verification on a Windows machine is required.

## Test 1: Rapid Window Switching (Deduplication & HWND splitting)
**Scenario**: Verify that rapidly switching between identical applications correctly bounds and deduplicates sessions.
1. Open two separate windows of the same application (e.g., two instances of Notepad).
2. Note their process names but observe that they have different window handles.
3. Bring Window A to the foreground. Wait 10 seconds.
4. Bring Window B to the foreground. Wait 10 seconds.
5. Bring Window A back to the foreground. Wait 10 seconds.
6. Check `activity_sessions` database table.
**Expected Result**: There should be three distinct sessions with boundaries corresponding exactly to the switch times, even though `app_name` and `app_path` remained constant.

## Test 2: System Lock and Unlock
**Scenario**: Verify that locking the workstation correctly closes the active session and unlocking it correctly begins a new session.
1. Ensure the tracker is running and an application is in the foreground.
2. Press `Win + L` to lock the workstation. Wait 10 seconds.
3. Unlock the workstation.
4. Check `activity_sessions`.
**Expected Result**: The session prior to locking must have a `finalization_reason` of `WorkstationLocked`. The duration must strictly represent the time up to the lock. A new session must begin immediately upon unlock.

## Test 3: System Sleep and Resume
**Scenario**: Verify that sending the computer to sleep gracefully finalizes the open session via power broadcast messages.
1. Ensure the tracker is running and an application is in the foreground.
2. Put the computer to Sleep (Start > Power > Sleep).
3. Wait at least 1-2 minutes.
4. Wake the computer and unlock.
5. Check `activity_sessions`.
**Expected Result**: The pre-sleep session must have a `finalization_reason` of `SystemSuspended`. The wall-clock time passed during sleep must NOT be included in any session duration. A new session must begin upon waking.

## Test 4: Heartbeat & Missing Hook Recovery
**Scenario**: Verify the 60-second background polling catches state if a hook is missed or dropped.
1. Temporarily disable the `EVENT_SYSTEM_FOREGROUND` hook in code (for testing only).
2. Start the tracker.
3. Switch foreground applications.
4. Wait for 60-70 seconds.
**Expected Result**: The tracker's background heartbeat timer will poll `GetForegroundWindow()`, observe the change, and emit a foreground observation, closing the old session and starting a new one.

## Test 5: Abrupt Process Termination (Crash bounds)
**Scenario**: Verify that the heartbeat establishes a `last_trustworthy_monotonic_ms` bound for crash recovery.
1. Start the tracker with an active foreground application.
2. Wait 2 minutes (to allow heartbeats to fire).
3. Force kill the tracker process abruptly via Task Manager.
4. Restart the tracker.
**Expected Result**: The unfinalized session from the crash will be recovered by the `sync_worker` on next startup. Its duration will be strictly bounded up to the last heartbeat observation (approx. 60 seconds before the crash), rather than stretching until the restart time.
