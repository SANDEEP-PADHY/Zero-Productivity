pub mod windows_api;
pub mod tracker;
pub mod clock;

use std::sync::{mpsc, atomic::{AtomicU32, Ordering}, Arc, Mutex};
use zero_core::collector::{EventReceiver, EventSender};
use std::thread;

pub struct CollectorManager {
    sender: EventSender,
    receiver: Mutex<Option<EventReceiver>>,
    thread_id: Arc<AtomicU32>,
}

impl Default for CollectorManager {
    fn default() -> Self {
        Self::new()
    }
}

impl CollectorManager {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self { 
            sender, 
            receiver: Mutex::new(Some(receiver)),
            thread_id: Arc::new(AtomicU32::new(0)),
        }
    }

    pub fn start(&self) {
        #[cfg(target_os = "windows")]
        {
            let sender = self.sender.clone();
            let thread_id = self.thread_id.clone();
            thread::spawn(move || {
                tracker::run_foreground_tracker(sender, thread_id);
            });
        }
    }

    pub fn take_receiver(&self) -> Option<EventReceiver> {
        self.receiver.lock().unwrap().take()
    }
}

impl Drop for CollectorManager {
    fn drop(&mut self) {
        #[cfg(target_os = "windows")]
        {
            let tid = self.thread_id.load(Ordering::SeqCst);
            if tid != 0 {
                unsafe {
                    let _ = windows::Win32::UI::WindowsAndMessaging::PostThreadMessageW(
                        tid,
                        windows::Win32::UI::WindowsAndMessaging::WM_QUIT,
                        windows::Win32::Foundation::WPARAM(0),
                        windows::Win32::Foundation::LPARAM(0)
                    );
                }
            }
        }
    }
}
