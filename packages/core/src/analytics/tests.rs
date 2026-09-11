use crate::analytics::engine::*;
use chrono::{DateTime, TimeZone, Utc};
use chrono_tz::America::New_York;
use crate::rules::models::{ClassifiedSession, Classification, ActivityType};
use crate::resolver::{ResolvedSession, NormalizedIdentity, NormalizedApplication};
use crate::tracking::session::FinalizationReason;

fn create_session(
    id: &str,
    app_id: &str,
    start: DateTime<Utc>,
    duration_ms: u64,
    foreground_ms: u64,
    classification: Classification,
) -> ClassifiedSession {
    ClassifiedSession {
        resolved_session: ResolvedSession {
            session_id: id.to_string(),
            start_utc: start,
            end_utc: start + chrono::Duration::milliseconds(duration_ms as i64),
            duration_ms,
            foreground_ms,
            interaction_ms: 0,
            media_ms: 0,
            idle_ms: 0,
            identity: NormalizedIdentity::Application(NormalizedApplication {
                app_id: app_id.to_string(),
                normalized_name: app_id.to_string(),
                raw_name: app_id.to_string(),
                path: None,
            }),
            window_title: None,
            finalization_reason: FinalizationReason::Unknown,
            metadata: serde_json::json!({}),
        },
        classification,
        activity_type: ActivityType::Application,
        winning_rule_id: None,
        reason: None,
    }
}

#[test]
fn test_midnight_splitting() {
    let tz = &New_York;
    
    // 23:00 to 01:00 (2 hours total, 1 hour in Day 1, 1 hour in Day 2)
    // 1000ms duration, 800ms foreground. Ratio is 50%.
    let start = tz.with_ymd_and_hms(2026, 9, 3, 23, 0, 0).unwrap().with_timezone(&Utc);
    let session = create_session("1", "appA", start, 7_200_000, 3_600_000, Classification::Productive);
    
    let split = split_sessions_at_midnights(&session, tz);
    assert_eq!(split.len(), 2);
    
    // Part 1: 23:00 to 00:00 (1 hour)
    assert_eq!(split[0].resolved_session.duration_ms, 3_600_000);
    assert_eq!(split[0].resolved_session.foreground_ms, 1_800_000); // Proportional
    
    // Part 2: 00:00 to 01:00 (1 hour)
    assert_eq!(split[1].resolved_session.duration_ms, 3_600_000);
    assert_eq!(split[1].resolved_session.foreground_ms, 1_800_000);
}

#[test]
fn test_fractional_split_exact_sum() {
    let tz = &New_York;
    
    // Total 1001ms. 500ms in Day 1, 501ms in Day 2.
    // foreground = 1001. Ratio for Day 1 = 500/1001.
    // alloc_foreground = (1001 * (500/1001)).round() = 500.
    // Remaining = 1001 - 500 = 501.
    let start = tz.with_ymd_and_hms(2026, 9, 3, 23, 59, 59).unwrap().with_timezone(&Utc) + chrono::Duration::milliseconds(500);
    let session = create_session("frac", "appA", start, 1001, 1001, Classification::Productive);
    
    let split = split_sessions_at_midnights(&session, tz);
    assert_eq!(split.len(), 2);
    
    assert_eq!(split[0].resolved_session.duration_ms, 500);
    assert_eq!(split[0].resolved_session.foreground_ms, 500);
    
    assert_eq!(split[1].resolved_session.duration_ms, 501);
    assert_eq!(split[1].resolved_session.foreground_ms, 501);
    
    assert_eq!(split[0].resolved_session.foreground_ms + split[1].resolved_session.foreground_ms, 1001);
}

#[test]
fn test_dst_spring_forward_boundary() {
    let tz = &New_York;
    // In NY, DST spring forward happens 2nd Sunday in March at 2:00 AM.
    // E.g. March 8, 2026. 01:59:59 -> 03:00:00.
    // If a session starts at 23:00 March 7, and runs for 4 hours (14.4M ms).
    // It should end at 04:00 March 8 local time.
    let start = tz.with_ymd_and_hms(2026, 3, 7, 23, 0, 0).unwrap().with_timezone(&Utc);
    let session = create_session("dst", "appA", start, 14_400_000, 14_400_000, Classification::Productive);
    
    let split = split_sessions_at_midnights(&session, tz);
    assert_eq!(split.len(), 2);
    
    // March 7: 23:00 to 00:00 (1 hour = 3.6M ms)
    assert_eq!(split[0].resolved_session.duration_ms, 3_600_000);
    // March 8: 00:00 to 04:00 (but 02:00 doesn't exist, so 3 hours absolute = 10.8M ms)
    assert_eq!(split[1].resolved_session.duration_ms, 10_800_000);
}

#[test]
fn test_context_switching_counts() {
    let tz = &New_York;
    let start1 = tz.with_ymd_and_hms(2026, 9, 3, 10, 0, 0).unwrap().with_timezone(&Utc);
    let start2 = tz.with_ymd_and_hms(2026, 9, 3, 11, 0, 0).unwrap().with_timezone(&Utc);
    let start3 = tz.with_ymd_and_hms(2026, 9, 3, 12, 0, 0).unwrap().with_timezone(&Utc);
    
    let s1 = create_session("1", "appA", start1, 1000, 1000, Classification::Productive);
    let s2 = create_session("2", "appB", start2, 1000, 1000, Classification::Neutral);
    let s3 = create_session("3", "appA", start3, 1000, 1000, Classification::Productive);
    
    let daily = generate_daily_summaries(&[s1, s2, s3], tz);
    assert_eq!(daily.len(), 1);
    assert_eq!(daily[0].context_switches, 2); // A -> B (1), B -> A (2)
}

#[test]
fn test_weekly_summaries() {
    let tz = &New_York;
    let s_tuesday = create_session("1", "app", tz.with_ymd_and_hms(2026, 9, 1, 10,0,0).unwrap().with_timezone(&Utc), 1000, 1000, Classification::Productive);
    let s_wednesday = create_session("2", "app", tz.with_ymd_and_hms(2026, 9, 2, 10,0,0).unwrap().with_timezone(&Utc), 1000, 1000, Classification::Productive);
    
    let daily = generate_daily_summaries(&[s_tuesday, s_wednesday], tz);
    let weekly = generate_weekly_summaries(&daily, chrono::Weekday::Mon);
    
    assert_eq!(weekly.len(), 1);
    assert_eq!(weekly[0].total_tracked_ms, 2000);
}
