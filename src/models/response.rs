use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::Event;

/// Represents a week period
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeekPeriod {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

/// Response for weekly events endpoint
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeeklyEventsResponse {
    pub week: WeekPeriod,
    pub events: Vec<Event>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_week_period_creation() {
        let start = Utc.with_ymd_and_hms(2026, 2, 24, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2026, 3, 2, 23, 59, 59).unwrap();

        let period = WeekPeriod { start, end };

        assert_eq!(period.start, start);
        assert_eq!(period.end, end);
    }

    #[test]
    fn test_weekly_events_response_serialization() {
        let start = Utc.with_ymd_and_hms(2026, 2, 24, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2026, 3, 2, 23, 59, 59).unwrap();
        let event_start = Utc.with_ymd_and_hms(2026, 2, 25, 14, 0, 0).unwrap();
        let event_end = Utc.with_ymd_and_hms(2026, 2, 25, 15, 0, 0).unwrap();

        let event = Event {
            id: "event-1".to_string(),
            title: "Meeting".to_string(),
            description: Some("Team sync".to_string()),
            start: event_start,
            end: event_end,
            location: Some("Office".to_string()),
            all_day: false,
        };

        let response = WeeklyEventsResponse {
            week: WeekPeriod { start, end },
            events: vec![event],
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"week\""));
        assert!(json.contains("\"events\""));
        assert!(json.contains("\"start\""));
        assert!(json.contains("\"end\""));
    }

    #[test]
    fn test_weekly_events_response_with_empty_events() {
        let start = Utc.with_ymd_and_hms(2026, 2, 24, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2026, 3, 2, 23, 59, 59).unwrap();

        let response = WeeklyEventsResponse {
            week: WeekPeriod { start, end },
            events: vec![],
        };

        assert_eq!(response.events.len(), 0);

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"events\":[]"));
    }

    #[test]
    fn test_weekly_events_response_with_multiple_events() {
        let start = Utc.with_ymd_and_hms(2026, 2, 24, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2026, 3, 2, 23, 59, 59).unwrap();

        let event1_start = Utc.with_ymd_and_hms(2026, 2, 25, 14, 0, 0).unwrap();
        let event1_end = Utc.with_ymd_and_hms(2026, 2, 25, 15, 0, 0).unwrap();

        let event2_start = Utc.with_ymd_and_hms(2026, 2, 26, 10, 0, 0).unwrap();
        let event2_end = Utc.with_ymd_and_hms(2026, 2, 26, 11, 30, 0).unwrap();

        let events = vec![
            Event {
                id: "event-1".to_string(),
                title: "Meeting 1".to_string(),
                description: None,
                start: event1_start,
                end: event1_end,
                location: None,
                all_day: false,
            },
            Event {
                id: "event-2".to_string(),
                title: "Meeting 2".to_string(),
                description: None,
                start: event2_start,
                end: event2_end,
                location: None,
                all_day: false,
            },
        ];

        let response = WeeklyEventsResponse {
            week: WeekPeriod { start, end },
            events,
        };

        assert_eq!(response.events.len(), 2);
        assert_eq!(response.events[0].title, "Meeting 1");
        assert_eq!(response.events[1].title, "Meeting 2");
    }

    #[test]
    fn test_weekly_events_response_deserialization() {
        let json = r#"{
            "week": {
                "start": "2026-02-24T00:00:00Z",
                "end": "2026-03-02T23:59:59Z"
            },
            "events": [
                {
                    "id": "event-1",
                    "title": "Meeting",
                    "description": "Description",
                    "start": "2026-02-25T14:00:00Z",
                    "end": "2026-02-25T15:00:00Z",
                    "location": "Office",
                    "all_day": false
                }
            ]
        }"#;

        let response: WeeklyEventsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.events.len(), 1);
        assert_eq!(response.events[0].title, "Meeting");
    }
}
