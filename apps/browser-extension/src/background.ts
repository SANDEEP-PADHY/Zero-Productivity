let nativePort: chrome.runtime.Port | null = null;
let reconnectTimer: number | null = null;

function connectNative() {
    if (nativePort) return;
    
    console.log("Connecting to Native Messaging host...");
    nativePort = chrome.runtime.connectNative('com.zeroproductivity.desktop');
    
    nativePort.onDisconnect.addListener(() => {
        console.error("Disconnected from Native Messaging host:", chrome.runtime.lastError?.message);
        nativePort = null;
        scheduleReconnect();
    });

    nativePort.onMessage.addListener((msg) => {
        console.log("Received from native host:", msg);
    });
    
    // Send immediate state
    reportCurrentState();
}

function scheduleReconnect() {
    if (reconnectTimer) return;
    reconnectTimer = setTimeout(() => {
        reconnectTimer = null;
        connectNative();
    }, 5000) as unknown as number;
}

function getDomain(urlStr: string | undefined): string | null {
    if (!urlStr) return null;
    try {
        const url = new URL(urlStr);
        if (url.protocol === 'http:' || url.protocol === 'https:') {
            return url.hostname;
        }
        return null;
    } catch {
        return null;
    }
}

async function reportCurrentState() {
    if (!nativePort) return;

    try {
        const window = await chrome.windows.getLastFocused();
        if (!window || window.id === undefined) return;
        
        const tabs = await chrome.tabs.query({ active: true, windowId: window.id });
        if (tabs.length === 0) return;
        
        const tab = tabs[0];
        
        const payload = {
            url: tab.url || null,
            domain: getDomain(tab.url),
            title: tab.title || null,
            windowId: window.id,
            tabId: tab.id,
            timestamp: new Date().toISOString(),
            isFocused: window.focused
        };

        nativePort.postMessage(payload);
    } catch (err) {
        console.error("Error reporting state:", err);
    }
}

// Listen to focus changes
chrome.windows.onFocusChanged.addListener(async (windowId) => {
    if (windowId === chrome.windows.WINDOW_ID_NONE) {
        // All Chrome windows lost focus, wait for next focus or ignore.
        // It's still useful to report that Chrome lost focus.
        if (nativePort) {
            nativePort.postMessage({
                url: null,
                domain: null,
                title: null,
                windowId: null,
                tabId: null,
                timestamp: new Date().toISOString(),
                isFocused: false
            });
        }
    } else {
        await reportCurrentState();
    }
});

// Listen to tab activation
chrome.tabs.onActivated.addListener(async (activeInfo) => {
    await reportCurrentState();
});

// Listen to tab updates (URL/title changes)
chrome.tabs.onUpdated.addListener(async (tabId, changeInfo, tab) => {
    if (tab.active) {
        // Only report if it's the active tab that updated
        await reportCurrentState();
    }
});

// Initialize
connectNative();
