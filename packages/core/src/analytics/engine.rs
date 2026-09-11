use std::collections::HashMap;
use chrono::{Datelike, TimeZone, NaiveDate, Utc, DateTime};
use chrono_tz::Tz;
use crate::rules::models::{ClassifiedSession, Classification};
use super::models::{DailySummary, ApplicationSummary, DomainSummary, SessionSummary, WeeklySummary};

pub fn clamp_session(session: &ClassifiedSession, range_start: DateTime<Utc>, range_end: DateTime<Utc>) -> Option<ClassifiedSession> {
    let s_start = session.resolved_session.start_utc;
    let s_end = session.resolved_session.end_utc;

    if s_end <= range_start || s_start >= range_end {
        return None;
    }

    let clamped_start = std::cmp::max(s_start, range_start);
    let clamped_end = std::cmp::min(s_end, range_end);

    let original_duration = session.resolved_session.duration_ms;
    if original_duration == 0 {
        return Some(session.clone());
    }

    let clamped_duration = (clamped_end - clamped_start).num_milliseconds() as u64;
    
    if clamped_duration == original_duration {
        return Some(session.clone());
    }

    let ratio = clamped_duration as f64 / original_duration as f64;

    let mut part = session.clone();
    part.resolved_session.start_utc = clamped_start;
    part.resolved_session.end_utc = clamped_end;
    part.resolved_session.duration_ms = clamped_duration;
    part.resolved_session.foreground_ms = (part.resolved_session.foreground_ms as f64 * ratio).round() as u64;
    part.resolved_session.interaction_ms = (part.resolved_session.interaction_ms as f64 * ratio).round() as u64;
    part.resolved_session.media_ms = (part.resolved_session.media_ms as f64 * ratio).round() as u64;
    part.resolved_session.idle_ms = (part.resolved_session.idle_ms as f64 * ratio).round() as u64;

    Some(part)
}

pub fn split_sessions_at_midnights(session: &ClassifiedSession, tz: &Tz) -> Vec<ClassifiedSession> {
    let mut results = Vec::new();
    let start_local = session.resolved_session.start_utc.with_timezone(tz);
    let end_local = session.resolved_session.end_utc.with_timezone(tz);

    if start_local.date_naive() == end_local.date_naive() {
        results.push(session.clone());
        return results;
    }

    let mut current_start = session.resolved_session.start_utc;
    let end_utc = session.resolved_session.end_utc;
    let total_duration = (end_utc - current_start).num_milliseconds() as u64;

    if total_duration == 0 {
        results.push(session.clone());
        return results;
    }

    let mut remaining_duration = session.resolved_session.duration_ms;
    let mut remaining_foreground = session.resolved_session.foreground_ms;
    let mut remaining_interaction = session.resolved_session.interaction_ms;
    let mut remaining_media = session.resolved_session.media_ms;
    let mut remaining_idle = session.resolved_session.idle_ms;

    while current_start.with_timezone(tz).date_naive() < end_local.date_naive() {
        let current_local = current_start.with_timezone(tz);
        let next_date = current_local.date_naive() + chrono::Duration::days(1);
        let next_midnight_local = next_date.and_hms_opt(0, 0, 0).unwrap();
        
        let next_midnight_utc = match tz.from_local_datetime(&next_midnight_local) {
            chrono::LocalResult::Single(t) => t.with_timezone(&Utc),
            chrono::LocalResult::Ambiguous(t1, _) => t1.with_timezone(&Utc),
            chrono::LocalResult::None => {
                let alt = next_date.and_hms_opt(1, 0, 0).unwrap();
                match tz.from_local_datetime(&alt) {
                    chrono::LocalResult::Single(t) => t.with_timezone(&Utc),
                    chrono::LocalResult::Ambiguous(t1, _) => t1.with_timezone(&Utc),
                    chrono::LocalResult::None => panic!("Both 00:00 and 01:00 skipped in tz"),
                }
            }
        };

        let split_end = std::cmp::min(next_midnight_utc, end_utc);
        let split_duration = (split_end - current_start).num_milliseconds() as u64;
        let ratio = split_duration as f64 / total_duration as f64;

        let alloc_foreground = (session.resolved_session.foreground_ms as f64 * ratio).round() as u64;
        let alloc_interaction = (session.resolved_session.interaction_ms as f64 * ratio).round() as u64;
        let alloc_media = (session.resolved_session.media_ms as f64 * ratio).round() as u64;
        let alloc_idle = (session.resolved_session.idle_ms as f64 * ratio).round() as u64;

        let mut part = session.clone();
        part.resolved_session.start_utc = current_start;
        part.resolved_session.end_utc = split_end;
        part.resolved_session.duration_ms = split_duration;
        part.resolved_session.foreground_ms = alloc_foreground;
        part.resolved_session.interaction_ms = alloc_interaction;
        part.resolved_session.media_ms = alloc_media;
        part.resolved_session.idle_ms = alloc_idle;
        
        results.push(part);
        
        remaining_duration = remaining_duration.saturating_sub(split_duration);
        remaining_foreground = remaining_foreground.saturating_sub(alloc_foreground);
        remaining_interaction = remaining_interaction.saturating_sub(alloc_interaction);
        remaining_media = remaining_media.saturating_sub(alloc_media);
        remaining_idle = remaining_idle.saturating_sub(alloc_idle);
        
        current_start = split_end;
    }

    if current_start < end_utc {
        let mut part = session.clone();
        part.resolved_session.start_utc = current_start;
        part.resolved_session.end_utc = end_utc;
        // Use exactly whatever is remaining to guarantee exact sum
        part.resolved_session.duration_ms = remaining_duration;
        part.resolved_session.foreground_ms = remaining_foreground;
        part.resolved_session.interaction_ms = remaining_interaction;
        part.resolved_session.media_ms = remaining_media;
        part.resolved_session.idle_ms = remaining_idle;
        
        results.push(part);
    }

    results
}

pub fn generate_daily_summaries(sessions: &[ClassifiedSession], tz: &Tz) -> Vec<DailySummary> {
    let mut split_sessions = Vec::new();
    for session in sessions {
        split_sessions.extend(split_sessions_at_midnights(session, tz));
    }

    split_sessions.sort_by_key(|s| s.resolved_session.start_utc);

    let mut daily_map: HashMap<NaiveDate, DailySummary> = HashMap::new();
    let mut app_map: HashMap<NaiveDate, HashMap<String, (Option<String>, u64)>> = HashMap::new();
    let mut domain_map: HashMap<NaiveDate, HashMap<String, u64>> = HashMap::new();
    let mut session_lists: HashMap<NaiveDate, Vec<SessionSummary>> = HashMap::new();

    let mut prev_identity: Option<String> = None;

    for session in &split_sessions {
        let date = session.resolved_session.start_utc.with_timezone(tz).date_naive();
        
        let (app_id, app_name, domain) = match &session.resolved_session.identity {
            crate::resolver::models::NormalizedIdentity::Application(app) => {
                (app.app_id.clone(), Some(app.raw_name.clone()), None)
            },
            crate::resolver::models::NormalizedIdentity::Browser(browser) => {
                (browser.browser_name.clone(), Some(browser.raw_name.clone()), browser.domain.clone())
            }
        };

        let identity = app_id.clone();
        
        let mut switched = false;
        if let Some(prev) = &prev_identity {
            if prev != &identity {
                switched = true;
            }
        }
        
        let summary = daily_map.entry(date).or_insert_with(|| DailySummary {
            date,
            total_tracked_ms: 0,
            total_foreground_ms: 0,
            total_interaction_ms: 0,
            total_idle_ms: 0,
            total_media_ms: 0,
            total_productive_ms: 0,
            total_neutral_ms: 0,
            total_distracting_ms: 0,
            total_uncategorized_ms: 0,
            context_switches: 0,
            top_applications: Vec::new(),
            top_domains: Vec::new(),
            longest_sessions: Vec::new(),
        });
        
        if switched {
            summary.context_switches += 1;
        }

        summary.total_tracked_ms += session.resolved_session.duration_ms;
        summary.total_foreground_ms += session.resolved_session.foreground_ms;
        summary.total_interaction_ms += session.resolved_session.interaction_ms;
        summary.total_idle_ms += session.resolved_session.idle_ms;
        summary.total_media_ms += session.resolved_session.media_ms;

        match session.classification {
            Classification::Productive => summary.total_productive_ms += session.resolved_session.duration_ms,
            Classification::Neutral => summary.total_neutral_ms += session.resolved_session.duration_ms,
            Classification::Distracting => summary.total_distracting_ms += session.resolved_session.duration_ms,
            Classification::Unknown => summary.total_uncategorized_ms += session.resolved_session.duration_ms,
        }

        let amap = app_map.entry(date).or_default();
        let a_entry = amap.entry(identity.clone()).or_insert_with(|| (app_name.clone(), 0));
        a_entry.1 += session.resolved_session.duration_ms;

        if let Some(dom) = &domain {
            let dmap = domain_map.entry(date).or_default();
            *dmap.entry(dom.clone()).or_insert(0) += session.resolved_session.duration_ms;
        }

        let slist = session_lists.entry(date).or_default();
        slist.push(SessionSummary {
            session_id: session.resolved_session.session_id.clone(),
            application_id: app_id.clone(),
            application_name: app_name.clone(),
            domain: domain.clone(),
            duration_ms: session.resolved_session.duration_ms,
            classification: session.classification,
            activity_type: session.activity_type,
        });

        prev_identity = Some(identity);
    }

    let mut results = Vec::new();
    for (date, mut summary) in daily_map {
        if let Some(amap) = app_map.get(&date) {
            for (app_id, (app_name, duration)) in amap {
                summary.top_applications.push(ApplicationSummary {
                    application_id: app_id.clone(),
                    application_name: app_name.clone(),
                    total_ms: *duration,
                });
            }
            summary.top_applications.sort_by(|a, b| b.total_ms.cmp(&a.total_ms).then_with(|| a.application_id.cmp(&b.application_id)));
        }

        if let Some(dmap) = domain_map.get(&date) {
            for (domain, duration) in dmap {
                summary.top_domains.push(DomainSummary {
                    domain: domain.clone(),
                    total_ms: *duration,
                });
            }
            summary.top_domains.sort_by(|a, b| b.total_ms.cmp(&a.total_ms).then_with(|| a.domain.cmp(&b.domain)));
        }

        if let Some(mut slist) = session_lists.remove(&date) {
            slist.sort_by(|a, b| b.duration_ms.cmp(&a.duration_ms).then_with(|| a.session_id.cmp(&b.session_id)));
            slist.truncate(50);
            summary.longest_sessions = slist;
        }

        results.push(summary);
    }

    results.sort_by_key(|s| s.date);
    results
}

pub fn generate_weekly_summaries(daily_summaries: &[DailySummary], week_start: chrono::Weekday) -> Vec<WeeklySummary> {
    let mut week_map: HashMap<NaiveDate, Vec<DailySummary>> = HashMap::new();

    for daily in daily_summaries {
        let mut start_date = daily.date;
        while start_date.weekday() != week_start {
            start_date -= chrono::Duration::days(1);
        }
        week_map.entry(start_date).or_default().push(daily.clone());
    }

    let mut results = Vec::new();

    for (start_date, days) in week_map {
        let end_date = start_date + chrono::Duration::days(6);
        let mut summary = WeeklySummary {
            start_date,
            end_date,
            total_tracked_ms: 0,
            total_foreground_ms: 0,
            total_interaction_ms: 0,
            total_idle_ms: 0,
            total_media_ms: 0,
            total_productive_ms: 0,
            total_neutral_ms: 0,
            total_distracting_ms: 0,
            total_uncategorized_ms: 0,
            context_switches: 0,
            top_applications: Vec::new(),
            top_domains: Vec::new(),
            longest_sessions: Vec::new(),
            daily_summaries: days.clone(),
        };

        let mut app_totals: HashMap<String, (Option<String>, u64)> = HashMap::new();
        let mut domain_totals: HashMap<String, u64> = HashMap::new();
        let mut longest_sessions = Vec::new();

        for day in &days {
            summary.total_tracked_ms += day.total_tracked_ms;
            summary.total_foreground_ms += day.total_foreground_ms;
            summary.total_interaction_ms += day.total_interaction_ms;
            summary.total_idle_ms += day.total_idle_ms;
            summary.total_media_ms += day.total_media_ms;
            summary.total_productive_ms += day.total_productive_ms;
            summary.total_neutral_ms += day.total_neutral_ms;
            summary.total_distracting_ms += day.total_distracting_ms;
            summary.total_uncategorized_ms += day.total_uncategorized_ms;
            summary.context_switches += day.context_switches;

            for app in &day.top_applications {
                let entry = app_totals.entry(app.application_id.clone()).or_insert_with(|| (app.application_name.clone(), 0));
                entry.1 += app.total_ms;
            }

            for dom in &day.top_domains {
                *domain_totals.entry(dom.domain.clone()).or_insert(0) += dom.total_ms;
            }

            longest_sessions.extend(day.longest_sessions.clone());
        }

        for (app_id, (app_name, total_ms)) in app_totals {
            summary.top_applications.push(ApplicationSummary { application_id: app_id, application_name: app_name, total_ms });
        }
        summary.top_applications.sort_by(|a, b| b.total_ms.cmp(&a.total_ms).then_with(|| a.application_id.cmp(&b.application_id)));

        for (domain, total_ms) in domain_totals {
            summary.top_domains.push(DomainSummary { domain, total_ms });
        }
        summary.top_domains.sort_by(|a, b| b.total_ms.cmp(&a.total_ms).then_with(|| a.domain.cmp(&b.domain)));

        longest_sessions.sort_by(|a, b| b.duration_ms.cmp(&a.duration_ms).then_with(|| a.session_id.cmp(&b.session_id)));
        // Deduplicate longest sessions (since daily aggregation could theoretically have overlapping sessions if they were split, but we keep them split)
        // Wait, longest sessions in a week shouldn't be pieced back together since Analytics can't merge them.
        longest_sessions.truncate(50);
        summary.longest_sessions = longest_sessions;

        results.push(summary);
    }

    results.sort_by_key(|w| w.start_date);
    results
}

