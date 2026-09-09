use chrono::Utc;
use std::cell::{Cell, RefCell};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use windows::core::w;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::RemoteDesktop::{
    WTSRegisterSessionNotification, WTSUnRegisterSessionNotification, NOTIFY_FOR_THIS_SESSION
};
use windows::Win32::System::SystemInformation::GetTickCount64;
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::Accessibility::{SetWinEventHook, UnhookWinEvent, HWINEVENTHOOK};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetForegroundWindow, GetMessageW, 
    RegisterClassW, TranslateMessage, EVENT_SYSTEM_FOREGROUND, MSG, WNDCLASSW, CS_HREDRAW, CS_VREDRAW, 
    WM_WTSSESSION_CHANGE, WM_POWERBROADCAST, SetTimer, KillTimer, WM_TIMER,
};
const PBT_APMSUSPEND: u32 = 0x0004;
const PBT_APMRESUMEAUTOMATIC: u32 = 0x0012;

const WTS_SESSION_LOCK: u32 = 0x7;
const WTS_SESSION_UNLOCK: u32 = 0x8;
use zero_core::collector::EventSender;
use zero_core::models::Observation;
use crate::collector::windows_api;

thread_local! {
    static SENDER: RefCell<Option<EventSender>> = const { RefCell::new(None) };
    static IS_LOCKED: Cell<bool> = const { Cell::new(false) };
}

pub fn run_foreground_tracker(sender: EventSender, thread_id: Arc<AtomicU32>) {
    SENDER.with(|s| {
        *s.borrow_mut() = Some(sender);
    });

    unsafe {
        let tid = GetCurrentThreadId();
        thread_id.store(tid, Ordering::SeqCst);

        let instance = GetModuleHandleW(None).unwrap();

        let window_class = w!("ZeroProductivityCollector");
        let wc = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wndproc),
            hInstance: instance.into(),
            lpszClassName: window_class,
            ..Default::default()
        };
        let _ = RegisterClassW(&wc);

        let hwnd = CreateWindowExW(
            Default::default(),
            window_class,
            w!("Zero Productivity Hidden Window"),
            Default::default(),
            0, 0, 0, 0,
            HWND::default(),
            None,
            instance,
            None,
        ).unwrap();

        let _ = WTSRegisterSessionNotification(hwnd, NOTIFY_FOR_THIS_SESSION);

        let hook = SetWinEventHook(
            EVENT_SYSTEM_FOREGROUND,
            EVENT_SYSTEM_FOREGROUND,
            None,
            Some(win_event_proc),
            0,
            0,
            0,
        );

        // Capture initial foreground state
        let initial_hwnd = GetForegroundWindow();
        if !initial_hwnd.is_invalid() {
            emit_foreground(initial_hwnd);
        }

        let _ = SetTimer(hwnd, 1, 60000, None);

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, HWND::default(), 0, 0).into() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        // Cleanup
        let _ = KillTimer(hwnd, 1);
        if !hook.is_invalid() {
            let _ = UnhookWinEvent(hook);
        }
        let _ = WTSUnRegisterSessionNotification(hwnd);
        let _ = DestroyWindow(hwnd);
    }
}

unsafe extern "system" fn wndproc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if msg == WM_WTSSESSION_CHANGE {
        let event = wparam.0 as u32;
        if event == WTS_SESSION_LOCK {
            emit_locked();
        } else if event == WTS_SESSION_UNLOCK {
            emit_unlocked();
        }
    } else if msg == WM_POWERBROADCAST {
        let event = wparam.0 as u32;
        if event == PBT_APMSUSPEND {
            emit_suspended();
        } else if event == PBT_APMRESUMEAUTOMATIC {
            emit_resumed();
        }
    } else if msg == WM_TIMER {
        let timer_id = wparam.0;
        if timer_id == 1 {
            let locked = IS_LOCKED.with(|l| l.get());
            if !locked {
                let hwnd = windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow();
                if !hwnd.is_invalid() {
                    emit_foreground(hwnd);
                }
            }
        }
    }
    DefWindowProcW(hwnd, msg, wparam, lparam)
}

unsafe extern "system" fn win_event_proc(
    _h_win_event_hook: HWINEVENTHOOK,
    event: u32,
    hwnd: HWND,
    _id_object: i32,
    _id_child: i32,
    _id_event_thread: u32,
    _dwms_event_time: u32,
) {
    if event == EVENT_SYSTEM_FOREGROUND {
        let locked = IS_LOCKED.with(|l| l.get());
        if !locked {
            emit_foreground(hwnd);
        }
    }
}

fn emit_foreground(hwnd: HWND) {
    if hwnd.is_invalid() { return; }

    let title = windows_api::get_window_title(hwnd);
    let pid = windows_api::get_process_id_from_hwnd(hwnd);
    let path = windows_api::get_process_path(pid);
    let name = path.as_ref().and_then(|p| windows_api::get_process_name_from_path(p));

    let monotonic_ms = unsafe { GetTickCount64() };

    let obs = Observation::new_foreground(
        Utc::now(),
        monotonic_ms,
        title,
        name,
        path,
        pid,
        hwnd.0 as u64,
        "windows".to_string(),
    );

    SENDER.with(|s| {
        if let Some(sender) = s.borrow().as_ref() {
            let _ = sender.send(obs);
        }
    });
}

fn emit_locked() {
    IS_LOCKED.with(|l| l.set(true));
    let monotonic_ms = unsafe { GetTickCount64() };
    let obs = Observation::new_locked(Utc::now(), monotonic_ms, "windows".to_string());
    SENDER.with(|s| {
        if let Some(sender) = s.borrow().as_ref() {
            let _ = sender.send(obs);
        }
    });
}

fn emit_unlocked() {
    IS_LOCKED.with(|l| l.set(false));
    let monotonic_ms = unsafe { GetTickCount64() };
    let obs = Observation::new_unlocked(Utc::now(), monotonic_ms, "windows".to_string());
    SENDER.with(|s| {
        if let Some(sender) = s.borrow().as_ref() {
            let _ = sender.send(obs);
        }
    });
    
    unsafe {
        let hwnd = windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow();
        if !hwnd.is_invalid() {
            emit_foreground(hwnd);
        }
    }
}

fn emit_suspended() {
    IS_LOCKED.with(|l| l.set(true));
    let monotonic_ms = unsafe { GetTickCount64() };
    let obs = Observation::new_suspended(Utc::now(), monotonic_ms, "windows".to_string());
    SENDER.with(|s| {
        if let Some(sender) = s.borrow().as_ref() {
            let _ = sender.send(obs);
        }
    });
}

fn emit_resumed() {
    IS_LOCKED.with(|l| l.set(false));
    let monotonic_ms = unsafe { GetTickCount64() };
    let obs = Observation::new_resumed(Utc::now(), monotonic_ms, "windows".to_string());
    SENDER.with(|s| {
        if let Some(sender) = s.borrow().as_ref() {
            let _ = sender.send(obs);
        }
    });
    
    unsafe {
        let hwnd = windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow();
        if !hwnd.is_invalid() {
            emit_foreground(hwnd);
        }
    }
}


