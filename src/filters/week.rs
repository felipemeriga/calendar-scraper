use crate::models::{Event, WeekPeriod};
use chrono::{DateTime, Datelike, Duration, TimeZone, Timelike, Utc, Weekday};

/// Returns the current week period (Monday 00:00 to Sunday 23:59)
pub fn get_current_week() -> WeekPeriod {
    let now = Utc::now();
    get_week_for_date(now)
}

/// Returns the week period for a given date (Monday 00:00 to Sunday 23:59)
pub fn get_week_for_date(date: DateTime<Utc>) -> WeekPeriod {
    // Calculate days since Monday (0 = Monday, 6 = Sunday)
    let days_since_monday = match date.weekday() {
        Weekday::Mon => 0,
        Weekday::Tue => 1,
        Weekday::Wed => 2,
        Weekday::Thu => 3,
        Weekday::Fri => 4,
        Weekday::Sat => 5,
        Weekday::Sun => 6,
    };

    // Get Monday at 00:00:00
    let monday = date - Duration::days(days_since_monday);
    let monday_start = Utc
        .with_ymd_and_hms(monday.year(), monday.month(), monday.day(), 0, 0, 0)
        .unwrap();

    // Get Sunday at 23:59:59
    let sunday = monday + Duration::days(6);
    let sunday_end = Utc
        .with_ymd_and_hms(sunday.year(), sunday.month(), sunday.day(), 23, 59, 59)
        .unwrap();

    WeekPeriod {
        start: monday_start,
        end: sunday_end,
    }
}

/// Filters events that occur within the given week period
pub fn filter_events_by_week(events: Vec<Event>, week: &WeekPeriod) -> Vec<Event> {
    events
        .into_iter()
        .filter(|event| {
            // Event is in the week if:
            // 1. Event starts before week ends AND
            // 2. Event ends after week starts
            event.start <= week.end && event.end >= week.start
        })
        .collect()
}

/// Filters events that occur in the current week
pub fn filter_current_week_events(events: Vec<Event>) -> Vec<Event> {
    let week = get_current_week();
    filter_events_by_week(events, &week)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_get_week_for_date_monday() {
        // Monday, Feb 23, 2026
        let monday = Utc.with_ymd_and_hms(2026, 2, 23, 15, 30, 0).unwrap();
        let week = get_week_for_date(monday);

        assert_eq!(week.start.year(), 2026);
        assert_eq!(week.start.month(), 2);
        assert_eq!(week.start.day(), 23); // Monday
        assert_eq!(week.start.hour(), 0);
        assert_eq!(week.start.minute(), 0);
        assert_eq!(week.start.second(), 0);

        assert_eq!(week.end.year(), 2026);
        assert_eq!(week.end.month(), 3);
        assert_eq!(week.end.day(), 1); // Sunday
        assert_eq!(week.end.hour(), 23);
        assert_eq!(week.end.minute(), 59);
        assert_eq!(week.end.second(), 59);
    }

    #[test]
    fn test_get_week_for_date_wednesday() {
        // Wednesday, Feb 25, 2026
        let wednesday = Utc.with_ymd_and_hms(2026, 2, 25, 12, 0, 0).unwrap();
        let week = get_week_for_date(wednesday);

        // Should still get the same week (Feb 23 - Mar 1)
        assert_eq!(week.start.day(), 23); // Monday
        assert_eq!(week.end.day(), 1); // Sunday
    }

    #[test]
    fn test_get_week_for_date_sunday() {
        // Sunday, Mar 1, 2026
        let sunday = Utc.with_ymd_and_hms(2026, 3, 1, 20, 0, 0).unwrap();
        let week = get_week_for_date(sunday);

        // Should get the week Feb 23 - Mar 1
        assert_eq!(week.start.day(), 23); // Monday
        assert_eq!(week.end.day(), 1); // Sunday
    }

    #[test]
    fn test_filter_events_by_week_events_within_week() {
        let week = WeekPeriod {
            start: Utc.with_ymd_and_hms(2026, 2, 23, 0, 0, 0).unwrap(),
            end: Utc.with_ymd_and_hms(2026, 3, 1, 23, 59, 59).unwrap(),
        };

        let events = vec![
            Event {
                id: "1".to_string(),
                title: "Event 1".to_string(),
                description: None,
                start: Utc.with_ymd_and_hms(2026, 2, 25, 10, 0, 0).unwrap(),
                end: Utc.with_ymd_and_hms(2026, 2, 25, 11, 0, 0).unwrap(),
                location: None,
                all_day: false,
            },
            Event {
                id: "2".to_string(),
                title: "Event 2".to_string(),
                description: None,
                start: Utc.with_ymd_and_hms(2026, 2, 27, 14, 0, 0).unwrap(),
                end: Utc.with_ymd_and_hms(2026, 2, 27, 15, 0, 0).unwrap(),
                location: None,
                all_day: false,
            },
        ];

        let filtered = filter_events_by_week(events, &week);
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_filter_events_by_week_events_outside_week() {
        let week = WeekPeriod {
            start: Utc.with_ymd_and_hms(2026, 2, 23, 0, 0, 0).unwrap(),
            end: Utc.with_ymd_and_hms(2026, 3, 1, 23, 59, 59).unwrap(),
        };

        let events = vec![
            // Event before the week
            Event {
                id: "1".to_string(),
                title: "Past Event".to_string(),
                description: None,
                start: Utc.with_ymd_and_hms(2026, 2, 20, 10, 0, 0).unwrap(),
                end: Utc.with_ymd_and_hms(2026, 2, 20, 11, 0, 0).unwrap(),
                location: None,
                all_day: false,
            },
            // Event after the week
            Event {
                id: "2".to_string(),
                title: "Future Event".to_string(),
                description: None,
                start: Utc.with_ymd_and_hms(2026, 3, 10, 14, 0, 0).unwrap(),
                end: Utc.with_ymd_and_hms(2026, 3, 10, 15, 0, 0).unwrap(),
                location: None,
                all_day: false,
            },
        ];

        let filtered = filter_events_by_week(events, &week);
        assert_eq!(filtered.len(), 0);
    }

    #[test]
    fn test_filter_events_by_week_event_spanning_week_boundary() {
        let week = WeekPeriod {
            start: Utc.with_ymd_and_hms(2026, 2, 23, 0, 0, 0).unwrap(),
            end: Utc.with_ymd_and_hms(2026, 3, 1, 23, 59, 59).unwrap(),
        };

        let events = vec![
            // Event starting before week but ending during week
            Event {
                id: "1".to_string(),
                title: "Spanning Start".to_string(),
                description: None,
                start: Utc.with_ymd_and_hms(2026, 2, 22, 10, 0, 0).unwrap(),
                end: Utc.with_ymd_and_hms(2026, 2, 24, 11, 0, 0).unwrap(),
                location: None,
                all_day: false,
            },
            // Event starting during week but ending after week
            Event {
                id: "2".to_string(),
                title: "Spanning End".to_string(),
                description: None,
                start: Utc.with_ymd_and_hms(2026, 3, 1, 14, 0, 0).unwrap(),
                end: Utc.with_ymd_and_hms(2026, 3, 5, 15, 0, 0).unwrap(),
                location: None,
                all_day: false,
            },
        ];

        let filtered = filter_events_by_week(events, &week);
        assert_eq!(filtered.len(), 2); // Both should be included
    }

    #[test]
    fn test_filter_events_by_week_empty_list() {
        let week = WeekPeriod {
            start: Utc.with_ymd_and_hms(2026, 2, 23, 0, 0, 0).unwrap(),
            end: Utc.with_ymd_and_hms(2026, 3, 1, 23, 59, 59).unwrap(),
        };

        let events: Vec<Event> = vec![];
        let filtered = filter_events_by_week(events, &week);
        assert_eq!(filtered.len(), 0);
    }

    #[test]
    fn test_filter_events_on_week_boundaries() {
        let week = WeekPeriod {
            start: Utc.with_ymd_and_hms(2026, 2, 23, 0, 0, 0).unwrap(),
            end: Utc.with_ymd_and_hms(2026, 3, 1, 23, 59, 59).unwrap(),
        };

        let events = vec![
            // Event exactly at Monday start
            Event {
                id: "1".to_string(),
                title: "Monday Start".to_string(),
                description: None,
                start: Utc.with_ymd_and_hms(2026, 2, 23, 0, 0, 0).unwrap(),
                end: Utc.with_ymd_and_hms(2026, 2, 23, 1, 0, 0).unwrap(),
                location: None,
                all_day: false,
            },
            // Event exactly at Sunday end
            Event {
                id: "2".to_string(),
                title: "Sunday End".to_string(),
                description: None,
                start: Utc.with_ymd_and_hms(2026, 3, 1, 23, 0, 0).unwrap(),
                end: Utc.with_ymd_and_hms(2026, 3, 1, 23, 59, 59).unwrap(),
                location: None,
                all_day: false,
            },
        ];

        let filtered = filter_events_by_week(events, &week);
        assert_eq!(filtered.len(), 2); // Both should be included
    }
}
