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
    pub window_handle: Option<u64>,
    pub url: Option<String>,
    pub domain: Option<String>,
}

pub enum SessionState {
    Stopped,
    Locked,
    Active(Box<ActiveSession>),
}

pub struct TrackingEngine<C: Clock> {
    pub state: SessionState,
    pub clock: C,
    latest_browser_state: Option<crate::models::BrowserStatePayload>,
}

impl<C: Clock> TrackingEngine<C> {
    pub fn new(clock: C) -> Self {
        Self {
            state: SessionState::Stopped,
            clock,
            latest_browser_state: None,
        }
    }

    pub fn handle_observation(&mut self, obs: Observation) -> Option<FinalizedSession> {
        match obs.kind {
            ObservationKind::ForegroundChange => {
                match &mut self.state {
                    SessionState::Active(active) => {
                        // Check if window identity is identical
                        if active.window_handle == obs.window_handle && active.process_id == obs.process_id {
                            // The OS window is the same. Did the title change?
                            let title_changed = active.window_title != obs.window_title;
                            
                            let is_browser = active.app_name.as_ref().is_some_and(|n| {
                                let lower = n.to_lowercase();
                                lower.contains("chrome") || lower.contains("msedge") || lower.contains("firefox") || lower.contains("brave") || lower.contains("vivaldi") || lower.contains("opera")
                            });

                            if is_browser && title_changed {
                                // For browsers, a title change (almost always) means a tab change or navigation.
                                // We must split the session here so the TrackingEngine retains temporal authority
                                // and the new session can be enriched with the new URL.
                                let finalized = self.finalize_active(FinalizationReason::TabChanged, obs.timestamp, obs.monotonic_ms);
                                self.start_new_session(obs);
                                return Some(finalized);
                            } else {
                                // For non-browsers, or if title didn't change, just update metadata
                                active.last_trustworthy_utc = obs.timestamp;
                                active.last_trustworthy_monotonic_ms = obs.monotonic_ms;
                                active.window_title = obs.window_title;
                                return None;
                            }
                        }

                        // Different window/application: finalize old and start new
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
            ObservationKind::SystemSuspended => {
                if let SessionState::Active(_) = self.state {
                    let finalized = self.finalize_active(FinalizationReason::SystemSuspended, obs.timestamp, obs.monotonic_ms);
                    self.state = SessionState::Stopped;
                    Some(finalized)
                } else {
                    self.state = SessionState::Stopped;
                    None
                }
            }
            ObservationKind::SessionUnlocked | ObservationKind::SystemResumed => {
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
            ObservationKind::BrowserState(payload) => {
                // Update the shared blackboard of latest browser state.
                self.latest_browser_state = Some(payload);

                if let SessionState::Active(active) = &mut self.state {
                    let is_browser = active.app_name.as_ref().is_some_and(|n| {
                        let lower = n.to_lowercase();
                        lower.contains("chrome") || lower.contains("msedge") || lower.contains("firefox") || lower.contains("brave") || lower.contains("vivaldi") || lower.contains("opera")
                    });

                    if is_browser {
                        // LIMITATION: Stage 8.3
                        // Currently, there is no reliable way to map an extension's windowId to a Windows HWND.
                        // Two separate Chrome windows can legitimately have the exact same window_title (e.g. "YouTube - Google Chrome").
                        // Therefore, title equality alone MUST NOT be treated as authoritative identity.
                        // We must NOT guess. If we cannot cryptographically prove the BrowserState belongs to the Active HWND,
                        // we must fall back to the conservative behavior: retain browser application activity, but DO NOT attach URL/domain.
                        // 
                        // FUTURE MECHANISM required for stronger correlation:
                        // Either a native UI Automation hook to walk the accessibility tree and read the Chrome internal windowId,
                        // or injecting a temporary UUID into the Chrome window title via the extension which the OS collector can definitively read.
                        
                        // We intentionally DO NOT enrich active.url or active.domain here until a reliable mapping is established.
                        active.last_trustworthy_utc = obs.timestamp;
                        active.last_trustworthy_monotonic_ms = obs.monotonic_ms;
                    }
                }
                None
            }
        }
    }

    pub fn shutdown(&mut self) -> Option<FinalizedSession> {
        if let SessionState::Active(_) = self.state {
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
            window_handle: obs.window_handle,
            url: None, // No URL enrichment due to lacking authoritative correlation mapping
            domain: None,
        };

        self.state = SessionState::Active(Box::new(active));
    }

    fn finalize_active(&mut self, reason: FinalizationReason, end_utc: chrono::DateTime<chrono::Utc>, end_monotonic_ms: u64) -> FinalizedSession {
        if let SessionState::Active(active) = std::mem::replace(&mut self.state, SessionState::Stopped) {
            let duration_ms = end_monotonic_ms.saturating_sub(active.start_monotonic_ms);
            
            FinalizedSession {
                session_id: active.session_id,
                start_utc: active.start_utc,
                end_utc,
                duration_ms,
                foreground_ms: duration_ms,
                interaction_ms: 0,
                media_ms: 0,
                idle_ms: 0,
                app_name: active.app_name,
                app_path: active.app_path,
                window_title: active.window_title,
                process_id: active.process_id,
                window_handle: active.window_handle,
                url: active.url,
                domain: active.domain,
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
    fn test_same_application_different_window_creates_new_session() {
        let (mut engine, _clock, ts) = setup();
        let obs1 = Observation::new_foreground(ts, 1000, Some("Window 1".into()), Some("app.exe".into()), None, 1, 10, "win".into());
        let obs2 = Observation::new_foreground(ts, 2000, Some("Window 2".into()), Some("app.exe".into()), None, 1, 20, "win".into());
        
        engine.handle_observation(obs1);
        let finalized = engine.handle_observation(obs2).unwrap();
        
        assert_eq!(finalized.duration_ms, 1000);
        assert_eq!(finalized.finalization_reason, FinalizationReason::ApplicationChanged);

        match &engine.state {
            SessionState::Active(a) => {
                assert_eq!(a.window_handle, Some(20));
                assert_eq!(a.start_monotonic_ms, 2000);
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

    #[test]
    fn test_system_suspend_resume() {
        let (mut engine, clock, ts) = setup();
        engine.handle_observation(Observation::new_foreground(ts, 1000, None, Some("app.exe".into()), None, 1, 10, "win".into()));

        clock.advance_ms(5000);
        let obs_suspend = Observation::new_suspended(clock.now_utc(), clock.now_monotonic_ms(), "win".into());
        let finalized = engine.handle_observation(obs_suspend).unwrap();

        assert_eq!(finalized.duration_ms, 5000);
        assert_eq!(finalized.finalization_reason, FinalizationReason::SystemSuspended);
        assert!(matches!(engine.state, SessionState::Stopped));

        clock.advance_ms(10000);
        let obs_resume = Observation::new_resumed(clock.now_utc(), clock.now_monotonic_ms(), "win".into());
        engine.handle_observation(obs_resume);
        assert!(matches!(engine.state, SessionState::Stopped));
    }

    #[test]
    fn test_heartbeat() {
        let (mut engine, clock, ts) = setup();
        engine.handle_observation(Observation::new_foreground(ts, 1000, None, Some("app.exe".into()), None, 1, 10, "win".into()));

        clock.advance_ms(5000);
        let obs_hb = Observation::new_heartbeat(clock.now_utc(), clock.now_monotonic_ms(), "win".into());
        let finalized = engine.handle_observation(obs_hb);
        assert!(finalized.is_none());

        match &engine.state {
            SessionState::Active(a) => {
                assert_eq!(a.last_trustworthy_monotonic_ms, 6000);
            },
            _ => panic!("Expected active state"),
        }
    }

    #[test]
    fn test_browser_correlation_same_title_fails_safe() {
        let (mut engine, clock, ts) = setup();
        
        // 1. Initial OS foreground observation for Chrome HWND A
        let obs_os = Observation::new_foreground(ts, 1000, Some("YouTube - Google Chrome".into()), Some("chrome.exe".into()), None, 1, 10, "win".into());
        engine.handle_observation(obs_os);

        clock.advance_ms(1000);
        
        // 2. Extension reports Chrome Window B (which also has title YouTube)
        let payload1 = crate::models::BrowserStatePayload {
            url: Some("https://youtube.com/watch?v=123".into()),
            domain: Some("youtube.com".into()),
            title: Some("YouTube".into()),
            window_id: Some(2),
            tab_id: Some(2),
            is_focused: true,
        };
        let obs_ext1 = Observation::new_browser_state(clock.now_utc(), clock.now_monotonic_ms(), payload1, "win".into());
        let finalized1 = engine.handle_observation(obs_ext1);
        
        // Does not finalize.
        assert!(finalized1.is_none());

        // Because we cannot definitively map window_id=2 to window_handle=10, we MUST fail safe and NOT enrich!
        match &engine.state {
            SessionState::Active(a) => {
                assert_eq!(a.domain, None);
                assert_eq!(a.url, None);
            },
            _ => panic!("Expected active state"),
        }
    }

    #[test]
    fn test_browser_multiple_windows_fail_safe() {
        let (mut engine, clock, ts) = setup();
        
        // Window A
        let obs_os_a = Observation::new_foreground(ts, 1000, Some("YouTube - Google Chrome".into()), Some("chrome.exe".into()), None, 1, 10, "win".into());
        engine.handle_observation(obs_os_a);

        // Extension A
        let payload_a = crate::models::BrowserStatePayload {
            url: Some("https://youtube.com".into()),
            domain: Some("youtube.com".into()),
            title: Some("YouTube".into()),
            window_id: Some(1),
            tab_id: Some(1),
            is_focused: true,
        };
        engine.handle_observation(Observation::new_browser_state(clock.now_utc(), clock.now_monotonic_ms(), payload_a, "win".into()));
        
        // Window B
        clock.advance_ms(1000);
        let obs_os_b = Observation::new_foreground(clock.now_utc(), clock.now_monotonic_ms(), Some("Gmail - Google Chrome".into()), Some("chrome.exe".into()), None, 1, 11, "win".into());
        let finalized_a = engine.handle_observation(obs_os_b).unwrap();
        assert_eq!(finalized_a.domain, None); // Failed safe!
        
        // Extension B
        let payload_b = crate::models::BrowserStatePayload {
            url: Some("https://gmail.com".into()),
            domain: Some("gmail.com".into()),
            title: Some("Gmail".into()),
            window_id: Some(2),
            tab_id: Some(2),
            is_focused: true,
        };
        engine.handle_observation(Observation::new_browser_state(clock.now_utc(), clock.now_monotonic_ms(), payload_b, "win".into()));

        // Switch back to Window A
        clock.advance_ms(1000);
        let obs_os_a2 = Observation::new_foreground(clock.now_utc(), clock.now_monotonic_ms(), Some("YouTube - Google Chrome".into()), Some("chrome.exe".into()), None, 1, 10, "win".into());
        let finalized_b = engine.handle_observation(obs_os_a2).unwrap();
        assert_eq!(finalized_b.domain, None); // Failed safe!

        match &engine.state {
            SessionState::Active(a) => {
                assert_eq!(a.domain, None); // Still safe
            },
            _ => panic!("Expected active state"),
        }
    }
}
