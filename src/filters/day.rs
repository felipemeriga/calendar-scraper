use crate::models::{Event, WeekPeriod};
use chrono::{Datelike, Duration, TimeZone, Utc};

/// Represents a day period
pub type DayPeriod = WeekPeriod;

/// Get the current day period (today 00:00 to 23:59:59 UTC)
pub fn get_current_day() -> DayPeriod {
    let now = Utc::now();
    let start = Utc
        .with_ymd_and_hms(now.year(), now.month(), now.day(), 0, 0, 0)
        .unwrap();
    let end = Utc
        .with_ymd_and_hms(now.year(), now.month(), now.day(), 23, 59, 59)
        .unwrap();

    DayPeriod { start, end }
}

/// Get today and tomorrow period (today 00:00 to tomorrow 23:59:59 UTC)
pub fn get_today_and_tomorrow() -> DayPeriod {
    let now = Utc::now();
    let start = Utc
        .with_ymd_and_hms(now.year(), now.month(), now.day(), 0, 0, 0)
        .unwrap();

    let tomorrow = now + Duration::days(1);
    let end = Utc
        .with_ymd_and_hms(
            tomorrow.year(),
            tomorrow.month(),
            tomorrow.day(),
            23,
            59,
            59,
        )
        .unwrap();

    DayPeriod { start, end }
}

/// Filter events that occur within a specific day period
pub fn filter_events_by_day(events: Vec<Event>, period: &DayPeriod) -> Vec<Event> {
    events
        .into_iter()
        .filter(|event| {
            // Event overlaps with period if:
            // - Event starts before period ends AND
            // - Event ends after period starts
            event.start <= period.end && event.end >= period.start
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Timelike};

    #[test]
    fn test_get_current_day() {
        let day = get_current_day();

        // Should start at 00:00:00
        assert_eq!(day.start.hour(), 0);
        assert_eq!(day.start.minute(), 0);
        assert_eq!(day.start.second(), 0);

        // Should end at 23:59:59
        assert_eq!(day.end.hour(), 23);
        assert_eq!(day.end.minute(), 59);
        assert_eq!(day.end.second(), 59);

        // Start and end should be the same day
        assert_eq!(day.start.day(), day.end.day());
    }

    #[test]
    fn test_get_today_and_tomorrow() {
        let period = get_today_and_tomorrow();

        // Should start at today 00:00:00
        assert_eq!(period.start.hour(), 0);
        assert_eq!(period.start.minute(), 0);
        assert_eq!(period.start.second(), 0);

        // Should end at tomorrow 23:59:59
        assert_eq!(period.end.hour(), 23);
        assert_eq!(period.end.minute(), 59);
        assert_eq!(period.end.second(), 59);

        // End should be 1 day after start
        let days_diff = (period.end - period.start).num_days();
        assert!(days_diff == 1 || days_diff == 2); // Account for the hours within the days
    }

    #[test]
    fn test_filter_events_by_day_events_within_day() {
        let day = DayPeriod {
            start: Utc.with_ymd_and_hms(2026, 2, 25, 0, 0, 0).unwrap(),
            end: Utc.with_ymd_and_hms(2026, 2, 25, 23, 59, 59).unwrap(),
        };

        let events = vec![
            Event {
                id: "1".to_string(),
                title: "Morning Event".to_string(),
                description: None,
                start: Utc.with_ymd_and_hms(2026, 2, 25, 9, 0, 0).unwrap(),
                end: Utc.with_ymd_and_hms(2026, 2, 25, 10, 0, 0).unwrap(),
                location: None,
                all_day: false,
                calendar: "test".to_string(),
            },
            Event {
                id: "2".to_string(),
                title: "Afternoon Event".to_string(),
                description: None,
                start: Utc.with_ymd_and_hms(2026, 2, 25, 14, 0, 0).unwrap(),
                end: Utc.with_ymd_and_hms(2026, 2, 25, 15, 0, 0).unwrap(),
                location: None,
                all_day: false,
                calendar: "test".to_string(),
            },
        ];

        let filtered = filter_events_by_day(events, &day);
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_filter_events_by_day_events_outside_day() {
        let day = DayPeriod {
            start: Utc.with_ymd_and_hms(2026, 2, 25, 0, 0, 0).unwrap(),
            end: Utc.with_ymd_and_hms(2026, 2, 25, 23, 59, 59).unwrap(),
        };

        let events = vec![
            // Event yesterday
            Event {
                id: "1".to_string(),
                title: "Yesterday Event".to_string(),
                description: None,
                start: Utc.with_ymd_and_hms(2026, 2, 24, 10, 0, 0).unwrap(),
                end: Utc.with_ymd_and_hms(2026, 2, 24, 11, 0, 0).unwrap(),
                location: None,
                all_day: false,
                calendar: "test".to_string(),
            },
            // Event tomorrow
            Event {
                id: "2".to_string(),
                title: "Tomorrow Event".to_string(),
                description: None,
                start: Utc.with_ymd_and_hms(2026, 2, 26, 14, 0, 0).unwrap(),
                end: Utc.with_ymd_and_hms(2026, 2, 26, 15, 0, 0).unwrap(),
                location: None,
                all_day: false,
                calendar: "test".to_string(),
            },
        ];

        let filtered = filter_events_by_day(events, &day);
        assert_eq!(filtered.len(), 0);
    }

    #[test]
    fn test_filter_events_by_day_spanning_event() {
        let day = DayPeriod {
            start: Utc.with_ymd_and_hms(2026, 2, 25, 0, 0, 0).unwrap(),
            end: Utc.with_ymd_and_hms(2026, 2, 25, 23, 59, 59).unwrap(),
        };

        let events = vec![
            // Event starting yesterday, ending today
            Event {
                id: "1".to_string(),
                title: "Spanning Start".to_string(),
                description: None,
                start: Utc.with_ymd_and_hms(2026, 2, 24, 23, 0, 0).unwrap(),
                end: Utc.with_ymd_and_hms(2026, 2, 25, 1, 0, 0).unwrap(),
                location: None,
                all_day: false,
                calendar: "test".to_string(),
            },
            // Event starting today, ending tomorrow
            Event {
                id: "2".to_string(),
                title: "Spanning End".to_string(),
                description: None,
                start: Utc.with_ymd_and_hms(2026, 2, 25, 23, 0, 0).unwrap(),
                end: Utc.with_ymd_and_hms(2026, 2, 26, 1, 0, 0).unwrap(),
                location: None,
                all_day: false,
                calendar: "test".to_string(),
            },
        ];

        let filtered = filter_events_by_day(events, &day);
        assert_eq!(filtered.len(), 2); // Both should be included
    }

    #[test]
    fn test_filter_events_by_day_empty_list() {
        let day = DayPeriod {
            start: Utc.with_ymd_and_hms(2026, 2, 25, 0, 0, 0).unwrap(),
            end: Utc.with_ymd_and_hms(2026, 2, 25, 23, 59, 59).unwrap(),
        };

        let events: Vec<Event> = vec![];
        let filtered = filter_events_by_day(events, &day);
        assert_eq!(filtered.len(), 0);
    }

    #[test]
    fn test_filter_events_today_and_tomorrow() {
        let period = DayPeriod {
            start: Utc.with_ymd_and_hms(2026, 2, 25, 0, 0, 0).unwrap(),
            end: Utc.with_ymd_and_hms(2026, 2, 26, 23, 59, 59).unwrap(),
        };

        let events = vec![
            // Event today
            Event {
                id: "1".to_string(),
                title: "Today Event".to_string(),
                description: None,
                start: Utc.with_ymd_and_hms(2026, 2, 25, 10, 0, 0).unwrap(),
                end: Utc.with_ymd_and_hms(2026, 2, 25, 11, 0, 0).unwrap(),
                location: None,
                all_day: false,
                calendar: "test".to_string(),
            },
            // Event tomorrow
            Event {
                id: "2".to_string(),
                title: "Tomorrow Event".to_string(),
                description: None,
                start: Utc.with_ymd_and_hms(2026, 2, 26, 14, 0, 0).unwrap(),
                end: Utc.with_ymd_and_hms(2026, 2, 26, 15, 0, 0).unwrap(),
                location: None,
                all_day: false,
                calendar: "test".to_string(),
            },
            // Event day after tomorrow (should not be included)
            Event {
                id: "3".to_string(),
                title: "Day After Tomorrow".to_string(),
                description: None,
                start: Utc.with_ymd_and_hms(2026, 2, 27, 10, 0, 0).unwrap(),
                end: Utc.with_ymd_and_hms(2026, 2, 27, 11, 0, 0).unwrap(),
                location: None,
                all_day: false,
                calendar: "test".to_string(),
            },
        ];

        let filtered = filter_events_by_day(events, &period);
        assert_eq!(filtered.len(), 2); // Only today and tomorrow
    }
}
