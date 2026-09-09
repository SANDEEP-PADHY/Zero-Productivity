pub mod models;
pub mod normalize;

use crate::tracking::session::FinalizedSession;
pub use models::{NormalizedApplication, NormalizedBrowser, NormalizedIdentity, ResolvedSession};
use normalize::{generate_app_id, normalize_app_name};

pub fn resolve_session(session: FinalizedSession) -> ResolvedSession {
    let raw_app_name = session.app_name.clone().unwrap_or_else(|| "Unknown".to_string());
    let normalized_name = normalize_app_name(&raw_app_name);

    let is_browser = matches!(
        normalized_name.as_str(),
        "chrome" | "firefox" | "msedge" | "brave" | "safari" | "iexplore" | "opera" | "vivaldi"
    );

    let identity = if is_browser {
        NormalizedIdentity::Browser(NormalizedBrowser {
            browser_name: normalized_name,
            raw_name: raw_app_name,
            domain: None,
            url: None,
            title: session.window_title.clone(),
        })
    } else {
        NormalizedIdentity::Application(NormalizedApplication {
            app_id: generate_app_id(&raw_app_name, session.app_path.as_deref()),
            normalized_name,
            raw_name: raw_app_name,
            path: session.app_path.clone(),
        })
    };

    let metadata = serde_json::json!({
        "process_id": session.process_id,
        "app_path": session.app_path,
        "finalization_reason": session.finalization_reason,
    });

    ResolvedSession {
        session_id: session.session_id,
        start_utc: session.start_utc,
        end_utc: session.end_utc,
        duration_ms: session.duration_ms,
        identity,
        window_title: session.window_title,
        finalization_reason: session.finalization_reason,
        metadata,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use crate::tracking::session::FinalizationReason;
    use crate::tracking::engine::TrackingEngine;
    use crate::tracking::clock::{FakeClock, Clock};
    use crate::models::Observation;

    #[test]
    fn test_resolve_application() {
        let session = FinalizedSession {
            session_id: "test-1".into(),
            start_utc: Utc.with_ymd_and_hms(2026, 9, 3, 10, 0, 0).unwrap(),
            end_utc: Utc.with_ymd_and_hms(2026, 9, 3, 10, 5, 0).unwrap(),
            duration_ms: 300000,
            app_name: Some("Code.exe".into()),
            app_path: Some("C:\\App\\Code.exe".into()),
            window_title: Some("Project - VS Code".into()),
            process_id: Some(1234),
            finalization_reason: FinalizationReason::ApplicationChanged,
        };

        let resolved = resolve_session(session.clone());

        assert_eq!(resolved.session_id, session.session_id);
        assert_eq!(resolved.start_utc, session.start_utc);
        assert_eq!(resolved.end_utc, session.end_utc);
        assert_eq!(resolved.duration_ms, session.duration_ms);
        assert_eq!(resolved.window_title, session.window_title);
        assert_eq!(resolved.finalization_reason, session.finalization_reason);

        match resolved.identity {
            NormalizedIdentity::Application(app) => {
                assert_eq!(app.normalized_name, "code");
                assert_eq!(app.raw_name, "Code.exe");
                assert_eq!(app.app_id, "app:code:c:/app/code.exe");
            }
            _ => panic!("Expected Application identity"),
        }
    }

    #[test]
    fn test_resolve_browser() {
        let session = FinalizedSession {
            session_id: "test-2".into(),
            start_utc: Utc.with_ymd_and_hms(2026, 9, 3, 10, 0, 0).unwrap(),
            end_utc: Utc.with_ymd_and_hms(2026, 9, 3, 10, 5, 0).unwrap(),
            duration_ms: 300000,
            app_name: Some("chrome.exe".into()),
            app_path: None,
            window_title: Some("GitHub - Google Chrome".into()),
            process_id: Some(5678),
            finalization_reason: FinalizationReason::ApplicationChanged,
        };

        let resolved = resolve_session(session);

        match resolved.identity {
            NormalizedIdentity::Browser(browser) => {
                assert_eq!(browser.browser_name, "chrome");
                assert_eq!(browser.raw_name, "chrome.exe");
                assert_eq!(browser.domain, None, "Domain MUST be None until extension provides it");
                assert_eq!(browser.url, None, "URL MUST be None until extension provides it");
                assert_eq!(browser.title, Some("GitHub - Google Chrome".into()));
            }
            _ => panic!("Expected Browser identity"),
        }
    }

    #[test]
    fn test_integration_tracking_engine_to_resolver() {
        let ts = Utc.with_ymd_and_hms(2026, 9, 3, 10, 0, 0).unwrap();
        let clock = FakeClock::new(ts, 1000);
        let mut engine = TrackingEngine::new(clock.clone());

        // Raw observation
        let obs1 = Observation::new_foreground(
            ts, 1000, Some("My Title".into()), Some("MyApp.exe".into()), Some("C:\\MyApp.exe".into()), 12, 100, "win".into()
        );
        engine.handle_observation(obs1);

        clock.advance_ms(5000);

        let obs2 = Observation::new_foreground(
            clock.now_utc(), clock.now_monotonic_ms(), Some("Other Title".into()), Some("OtherApp.exe".into()), None, 13, 101, "win".into()
        );
        let finalized = engine.handle_observation(obs2).unwrap();

        let resolved = resolve_session(finalized.clone());

        // Verify resolver did not change TrackingEngine output timings
        assert_eq!(resolved.start_utc, finalized.start_utc);
        assert_eq!(resolved.end_utc, finalized.end_utc);
        assert_eq!(resolved.duration_ms, finalized.duration_ms);
        assert_eq!(resolved.duration_ms, 5000);

        // Verify Classification is NOT present
        // (Currently our NormalizedIdentity model simply doesn't contain classification fields)
        match resolved.identity {
            NormalizedIdentity::Application(app) => {
                assert_eq!(app.raw_name, "MyApp.exe");
                assert_eq!(app.normalized_name, "myapp");
            }
            _ => panic!("Expected Application"),
        }
    }
}
