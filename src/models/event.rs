use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents a calendar event
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Event {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub location: Option<String>,
    pub all_day: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_event_creation() {
        let start = Utc.with_ymd_and_hms(2026, 2, 25, 14, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2026, 2, 25, 15, 0, 0).unwrap();

        let event = Event {
            id: "event-123".to_string(),
            title: "Team Meeting".to_string(),
            description: Some("Weekly sync".to_string()),
            start,
            end,
            location: Some("Office".to_string()),
            all_day: false,
        };

        assert_eq!(event.id, "event-123");
        assert_eq!(event.title, "Team Meeting");
        assert_eq!(event.description, Some("Weekly sync".to_string()));
        assert_eq!(event.start, start);
        assert_eq!(event.end, end);
        assert_eq!(event.location, Some("Office".to_string()));
        assert!(!event.all_day);
    }

    #[test]
    fn test_event_serialization() {
        let start = Utc.with_ymd_and_hms(2026, 2, 25, 14, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2026, 2, 25, 15, 0, 0).unwrap();

        let event = Event {
            id: "event-123".to_string(),
            title: "Meeting".to_string(),
            description: Some("Description".to_string()),
            start,
            end,
            location: Some("Office".to_string()),
            all_day: false,
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"id\":\"event-123\""));
        assert!(json.contains("\"title\":\"Meeting\""));
        assert!(json.contains("\"description\":\"Description\""));
        assert!(json.contains("\"all_day\":false"));
    }

    #[test]
    fn test_event_deserialization() {
        let json = r#"{
            "id": "event-456",
            "title": "Conference",
            "description": "Annual conference",
            "start": "2026-02-25T14:00:00Z",
            "end": "2026-02-25T15:00:00Z",
            "location": "Convention Center",
            "all_day": false
        }"#;

        let event: Event = serde_json::from_str(json).unwrap();
        assert_eq!(event.id, "event-456");
        assert_eq!(event.title, "Conference");
        assert_eq!(event.description, Some("Annual conference".to_string()));
        assert_eq!(event.location, Some("Convention Center".to_string()));
        assert!(!event.all_day);
    }

    #[test]
    fn test_event_with_optional_fields_none() {
        let start = Utc.with_ymd_and_hms(2026, 2, 26, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2026, 2, 26, 23, 59, 59).unwrap();

        let event = Event {
            id: "event-789".to_string(),
            title: "All Day Event".to_string(),
            description: None,
            start,
            end,
            location: None,
            all_day: true,
        };

        assert_eq!(event.description, None);
        assert_eq!(event.location, None);
        assert!(event.all_day);

        // Test serialization handles None values
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"description\":null"));
        assert!(json.contains("\"location\":null"));
    }

    #[test]
    fn test_event_equality() {
        let start = Utc.with_ymd_and_hms(2026, 2, 25, 14, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2026, 2, 25, 15, 0, 0).unwrap();

        let event1 = Event {
            id: "event-1".to_string(),
            title: "Meeting".to_string(),
            description: None,
            start,
            end,
            location: None,
            all_day: false,
        };

        let event2 = Event {
            id: "event-1".to_string(),
            title: "Meeting".to_string(),
            description: None,
            start,
            end,
            location: None,
            all_day: false,
        };

        assert_eq!(event1, event2);
    }
}
