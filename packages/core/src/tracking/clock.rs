use chrono::{DateTime, Utc};
use std::sync::{Arc, Mutex};
/// Clock trait for fetching current time and monotonic duration bounds.
pub trait Clock: Send + Sync {
    /// Returns the current UTC wall clock time.
    fn now_utc(&self) -> DateTime<Utc>;
    
    /// Returns a monotonic millisecond counter since an arbitrary epoch.
    fn now_monotonic_ms(&self) -> u64;
}



/// A fake clock for deterministic unit testing.
#[derive(Clone)]
pub struct FakeClock {
    state: Arc<Mutex<FakeClockState>>,
}

struct FakeClockState {
    utc: DateTime<Utc>,
    monotonic_ms: u64,
}

impl FakeClock {
    pub fn new(initial_utc: DateTime<Utc>, initial_monotonic_ms: u64) -> Self {
        Self {
            state: Arc::new(Mutex::new(FakeClockState {
                utc: initial_utc,
                monotonic_ms: initial_monotonic_ms,
            })),
        }
    }

    pub fn advance_ms(&self, ms: u64) {
        let mut state = self.state.lock().unwrap();
        state.utc += chrono::Duration::try_milliseconds(ms as i64).unwrap();
        state.monotonic_ms += ms;
    }
    
    pub fn advance_wall_clock(&self, offset_ms: i64) {
        let mut state = self.state.lock().unwrap();
        state.utc += chrono::Duration::try_milliseconds(offset_ms).unwrap();
        // Monotonic clock is strictly unaffected by wall clock changes!
    }
}

impl Clock for FakeClock {
    fn now_utc(&self) -> DateTime<Utc> {
        self.state.lock().unwrap().utc
    }

    fn now_monotonic_ms(&self) -> u64 {
        self.state.lock().unwrap().monotonic_ms
    }
}
