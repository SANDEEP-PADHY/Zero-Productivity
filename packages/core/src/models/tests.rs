use crate::models::{Observation, ObservationKind};
use chrono::{TimeZone, Utc};

#[test]
fn test_observation_creation() {
    let ts = Utc.with_ymd_and_hms(2026, 9, 3, 10, 0, 0).unwrap();
    
    let obs = Observation::new_foreground(
        ts,
        1000,
        Some("Document - Word".to_string()),
        Some("word.exe".to_string()),
        Some("C:\\Program Files\\word.exe".to_string()),
        1234,
        5678,
        "windows".to_string(),
    );

    assert_eq!(obs.kind, ObservationKind::ForegroundChange);
    assert_eq!(obs.monotonic_ms, 1000);
    assert_eq!(obs.window_title.as_deref(), Some("Document - Word"));
    assert_eq!(obs.app_name.as_deref(), Some("word.exe"));
    assert_eq!(obs.process_id, Some(1234));
    assert_eq!(obs.window_handle, Some(5678));
    assert_eq!(obs.platform, "windows");
}

#[test]
fn test_locked_observation() {
    let ts = Utc::now();
    let obs = Observation::new_locked(ts, 2000, "windows".to_string());
    
    assert_eq!(obs.kind, ObservationKind::SessionLocked);
    assert_eq!(obs.monotonic_ms, 2000);
    assert!(obs.app_name.is_none());
    assert!(obs.window_handle.is_none());
}
