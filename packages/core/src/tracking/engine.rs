use crate::models::{Observation, ObservationKind};
use crate::tracking::clock::Clock;
use crate::tracking::session::{FinalizedSession, FinalizationReason};
use uuid::Uuid;

pub struct ActiveSession {
    pub session_id: String,
    pub start_utc: chrono::DateTime<chrono::Utc>,
    pub start_monotonic_ms: u64,
    pub last_trustworthy_utc: chrono::DateTime<chrono::Utc>,
    pub last_trustworthy_monotonic_ms: u64,
    pub app_name: Option<String>,
    pub app_path: Option<String>,
    pub window_title: Option<String>,
    pub process_id: Option<u32>,
}

pub enum SessionState {
    Stopped,
    Locked,
    Active(ActiveSession),
}

pub struct TrackingEngine<C: Clock> {
    pub state: SessionState,
    pub clock: C,
}

impl<C: Clock> TrackingEngine<C> {
    pub fn new(clock: C) -> Self {
        Self {
            state: SessionState::Stopped,
            clock,
        }
    }

    pub fn handle_observation(&mut self, obs: Observation) -> Option<FinalizedSession> {
        match obs.kind {
            ObservationKind::ForegroundChange => {
                match &mut self.state {
                    SessionState::Active(active) => {
                        // Check if process identity is the same
                        if active.app_path == obs.app_path && active.app_name == obs.app_name && active.process_id == obs.process_id {
                            // Update last trustworthy bounds and optionally window title metadata
                            active.last_trustworthy_utc = obs.timestamp;
                            active.last_trustworthy_monotonic_ms = obs.monotonic_ms;
                            active.window_title = obs.window_title;
                            return None;
                        }

                        // Different application: finalize old and start new
                        let finalized = self.finalize_active(FinalizationReason::ApplicationChanged, obs.timestamp, obs.monotonic_ms);
                        self.start_new_session(obs);
                        Some(finalized)
                    }
                    SessionState::Stopped | SessionState::Locked => {
                        self.start_new_session(obs);
                        None
                    }
                }
            }
            ObservationKind::SessionLocked => {
                if let SessionState::Active(_) = self.state {
                    let finalized = self.finalize_active(FinalizationReason::WorkstationLocked, obs.timestamp, obs.monotonic_ms);
                    self.state = SessionState::Locked;
                    Some(finalized)
                } else {
                    self.state = SessionState::Locked;
                    None
                }
            }
            ObservationKind::SessionUnlocked => {
                self.state = SessionState::Stopped;
                None
            }
            ObservationKind::Heartbeat => {
                if let SessionState::Active(active) = &mut self.state {
                    active.last_trustworthy_utc = obs.timestamp;
                    active.last_trustworthy_monotonic_ms = obs.monotonic_ms;
                }
                None
            }
        }
    }

    pub fn shutdown(&mut self) -> Option<FinalizedSession> {
        if let SessionState::Active(_) = self.state {
            // Graceful shutdown implies the boundary is exactly now
            let end_utc = self.clock.now_utc();
            let end_monotonic = self.clock.now_monotonic_ms();
            let finalized = self.finalize_active(FinalizationReason::Shutdown, end_utc, end_monotonic);
            self.state = SessionState::Stopped;
            Some(finalized)
        } else {
            None
        }
    }

    fn start_new_session(&mut self, obs: Observation) {
        let active = ActiveSession {
            session_id: Uuid::now_v7().to_string(),
            start_utc: obs.timestamp,
            start_monotonic_ms: obs.monotonic_ms,
            last_trustworthy_utc: obs.timestamp,
            last_trustworthy_monotonic_ms: obs.monotonic_ms,
            app_name: obs.app_name,
            app_path: obs.app_path,
            window_title: obs.window_title,
            process_id: obs.process_id,
        };
        self.state = SessionState::Active(active);
    }

    fn finalize_active(&mut self, reason: FinalizationReason, end_utc: chrono::DateTime<chrono::Utc>, end_monotonic_ms: u64) -> FinalizedSession {
        if let SessionState::Active(active) = std::mem::replace(&mut self.state, SessionState::Stopped) {
            let duration_ms = end_monotonic_ms.saturating_sub(active.start_monotonic_ms);
            
            FinalizedSession {
                session_id: active.session_id,
                start_utc: active.start_utc,
                end_utc,
                duration_ms,
                app_name: active.app_name,
                app_path: active.app_path,
                window_title: active.window_title,
                process_id: active.process_id,
                finalization_reason: reason,
            }
        } else {
            unreachable!("finalize_active called when not active");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tracking::clock::FakeClock;
    use chrono::TimeZone;

    fn setup() -> (TrackingEngine<FakeClock>, FakeClock, chrono::DateTime<chrono::Utc>) {
        let ts = chrono::Utc.with_ymd_and_hms(2026, 9, 3, 10, 0, 0).unwrap();
        let clock = FakeClock::new(ts, 1000);
        let engine = TrackingEngine::new(clock.clone());
        (engine, clock, ts)
    }

    #[test]
    fn test_first_foreground_creates_session() {
        let (mut engine, _clock, ts) = setup();
        let obs = Observation::new_foreground(
            ts, 1000, Some("A".to_string()), Some("app_a.exe".to_string()), None, 1, 10, "win".into()
        );
        let finalized = engine.handle_observation(obs);
        assert!(finalized.is_none());
        
        match &engine.state {
            SessionState::Active(a) => {
                assert_eq!(a.app_name.as_deref(), Some("app_a.exe"));
                assert_eq!(a.start_monotonic_ms, 1000);
            },
            _ => panic!("Expected active state"),
        }
    }

    #[test]
    fn test_same_application_does_not_duplicate() {
        let (mut engine, _clock, ts) = setup();
        let obs1 = Observation::new_foreground(ts, 1000, Some("Title 1".into()), Some("app.exe".into()), None, 1, 10, "win".into());
        let obs2 = Observation::new_foreground(ts, 2000, Some("Title 2".into()), Some("app.exe".into()), None, 1, 10, "win".into());
        
        engine.handle_observation(obs1);
        let finalized = engine.handle_observation(obs2);
        assert!(finalized.is_none()); // Deduplicated

        match &engine.state {
            SessionState::Active(a) => {
                assert_eq!(a.window_title.as_deref(), Some("Title 2")); // Metadata updated
                assert_eq!(a.last_trustworthy_monotonic_ms, 2000);
            },
            _ => panic!("Expected active state"),
        }
    }

    #[test]
    fn test_application_switch() {
        let (mut engine, _clock, ts) = setup();
        let obs1 = Observation::new_foreground(ts, 1000, None, Some("app_a.exe".into()), None, 1, 10, "win".into());
        let obs2 = Observation::new_foreground(ts, 3000, None, Some("app_b.exe".into()), None, 2, 20, "win".into());
        
        engine.handle_observation(obs1);
        let finalized = engine.handle_observation(obs2).unwrap();
        
        assert_eq!(finalized.app_name.as_deref(), Some("app_a.exe"));
        assert_eq!(finalized.duration_ms, 2000);
        assert_eq!(finalized.finalization_reason, FinalizationReason::ApplicationChanged);

        match &engine.state {
            SessionState::Active(a) => {
                assert_eq!(a.app_name.as_deref(), Some("app_b.exe"));
                assert_eq!(a.start_monotonic_ms, 3000);
            },
            _ => panic!("Expected active state"),
        }
    }

    #[test]
    fn test_lock_and_unlock() {
        let (mut engine, clock, ts) = setup();
        let obs_active = Observation::new_foreground(ts, 1000, None, Some("app.exe".into()), None, 1, 10, "win".into());
        engine.handle_observation(obs_active);

        clock.advance_ms(5000);
        let obs_lock = Observation::new_locked(clock.now_utc(), clock.now_monotonic_ms(), "win".into());
        let finalized = engine.handle_observation(obs_lock).unwrap();

        assert_eq!(finalized.duration_ms, 5000);
        assert_eq!(finalized.finalization_reason, FinalizationReason::WorkstationLocked);
        assert!(matches!(engine.state, SessionState::Locked));

        clock.advance_ms(10000);
        let obs_unlock = Observation::new_unlocked(clock.now_utc(), clock.now_monotonic_ms(), "win".into());
        engine.handle_observation(obs_unlock);
        assert!(matches!(engine.state, SessionState::Stopped));

        let obs_new = Observation::new_foreground(clock.now_utc(), clock.now_monotonic_ms(), None, Some("app2.exe".into()), None, 2, 20, "win".into());
        engine.handle_observation(obs_new);
        match &engine.state {
            SessionState::Active(a) => {
                assert_eq!(a.app_name.as_deref(), Some("app2.exe"));
                assert_eq!(a.start_monotonic_ms, 16000);
            },
            _ => panic!("Expected active state"),
        }
    }

    #[test]
    fn test_shutdown() {
        let (mut engine, clock, ts) = setup();
        engine.handle_observation(Observation::new_foreground(ts, 1000, None, Some("app.exe".into()), None, 1, 10, "win".into()));
        clock.advance_ms(3000);
        
        let finalized = engine.shutdown().unwrap();
        assert_eq!(finalized.duration_ms, 3000);
        assert_eq!(finalized.finalization_reason, FinalizationReason::Shutdown);
        assert!(matches!(engine.state, SessionState::Stopped));
    }

    #[test]
    fn test_wall_clock_adjustment() {
        let (mut engine, clock, ts) = setup();
        engine.handle_observation(Observation::new_foreground(ts, 1000, None, Some("app.exe".into()), None, 1, 10, "win".into()));
        
        // 5 seconds pass monotonically, but wall clock jumps backward by 1 hour (e.g. DST)
        clock.advance_ms(5000);
        clock.advance_wall_clock(-3600000); 

        let obs2 = Observation::new_foreground(clock.now_utc(), clock.now_monotonic_ms(), None, Some("other.exe".into()), None, 2, 20, "win".into());
        let finalized = engine.handle_observation(obs2).unwrap();

        // Duration must strictly be 5000 despite the -1hr wall clock change
        assert_eq!(finalized.duration_ms, 5000);
    }
}
