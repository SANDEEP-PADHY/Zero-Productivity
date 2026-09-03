use chrono::{DateTime, Utc};
use windows::Win32::System::SystemInformation::GetTickCount64;
use zero_core::tracking::clock::Clock;

pub struct WindowsClock;

impl Clock for WindowsClock {
    fn now_utc(&self) -> DateTime<Utc> {
        Utc::now()
    }

    fn now_monotonic_ms(&self) -> u64 {
        unsafe { GetTickCount64() }
    }
}
