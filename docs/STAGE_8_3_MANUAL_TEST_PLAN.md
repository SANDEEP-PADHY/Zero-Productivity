# Stage 8.3: Browser Extension Integration Test Plan

## Prerequisites
1. Open Google Chrome.
2. Go to `chrome://extensions/`.
3. Enable "Developer mode" in the top right.
4. Click "Load unpacked" and select `apps/browser-extension`.
5. Note the generated Extension ID (e.g. `abcdefghijklmnopabcdefghijklmnop`).
6. Run the installation script in a PowerShell window:
   ```powershell
   .\install_native_manifest.ps1 -ExtensionId "<YOUR_EXTENSION_ID>"
   ```
7. Build the desktop app:
   ```powershell
   cd apps/desktop/src-tauri
   cargo build
   ```
8. Start the desktop app in a terminal:
   ```powershell
   .\target\debug\desktop.exe
   ```

## Test 1: Native Messaging Connection
1. In `chrome://extensions/`, click on "Service worker" for the Zero Productivity extension to open the DevTools.
2. Verify the console logs "Connecting to Native Messaging host...".
3. There should be NO error like "Specified native messaging host not found."
4. If it connects, the connection is successful.

## Test 2: Active Tab Tracking
1. Keep the desktop daemon running. Ensure the local Supabase Docker stack is running if you want to test sync, but it's not strictly required for local SQLite testing.
2. Focus Google Chrome and open a few tabs (e.g. `https://example.com`, `https://github.com`).
3. Switch between the tabs.
4. Open another application (e.g., VS Code or Notepad) so Chrome loses focus.
5. Wait 10-15 seconds for the desktop engine to persist the sessions.

## Verification
1. Open the SQLite database:
   ```powershell
   sqlite3 $env:LOCALAPPDATA\zero-productivity\data.db
   ```
   *(Adjust the path if your AppData resolves differently)*
2. Query the sessions:
   ```sql
   SELECT app_name, window_title, url, domain, duration_ms 
   FROM activity_sessions 
   ORDER BY end_utc DESC LIMIT 10;
   ```
3. Verify that:
   - There are sessions for `chrome.exe`.
   - The `url` and `domain` fields match the tabs you visited.
   - When Chrome was focused, the duration was tracked correctly per tab.
   - When you switched to Notepad, the Chrome tab session ended gracefully (duration stops increasing), and a new Notepad session began.
   - Switching back to Chrome starts a new session with the currently active tab.
