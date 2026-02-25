use chrono::{DateTime, Utc};
use chrono_tz::Tz;
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
    pub calendar: String,
}

impl Event {
    /// Converts event times to a specific timezone
    pub fn to_timezone(&self, tz: &Tz) -> EventInTimezone {
        EventInTimezone {
            id: self.id.clone(),
            title: self.title.clone(),
            description: self.description.clone(),
            start: self.start.with_timezone(tz).to_rfc3339(),
            end: self.end.with_timezone(tz).to_rfc3339(),
            location: self.location.clone(),
            all_day: self.all_day,
            calendar: self.calendar.clone(),
        }
    }
}

/// Event with times in a specific timezone (as RFC3339 strings)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventInTimezone {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub start: String,
    pub end: String,
    pub location: Option<String>,
    pub all_day: bool,
    pub calendar: String,
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
            calendar: "test".to_string(),
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
            calendar: "test".to_string(),
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
            "all_day": false,
            "calendar": "test"
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
            calendar: "test".to_string(),
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
            calendar: "test".to_string(),
        };

        let event2 = Event {
            id: "event-1".to_string(),
            title: "Meeting".to_string(),
            description: None,
            start,
            end,
            location: None,
            all_day: false,
            calendar: "test".to_string(),
        };

        assert_eq!(event1, event2);
    }

    #[test]
    fn test_event_to_timezone_est() {
        use chrono_tz::America::New_York;

        // Event at 14:00 UTC (9:00 AM EST)
        let start = Utc.with_ymd_and_hms(2026, 2, 25, 14, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2026, 2, 25, 15, 0, 0).unwrap();

        let event = Event {
            id: "event-1".to_string(),
            title: "Meeting".to_string(),
            description: None,
            start,
            end,
            location: None,
            all_day: false,
            calendar: "test".to_string(),
        };

        let event_in_tz = event.to_timezone(&New_York);

        assert_eq!(event_in_tz.id, "event-1");
        assert_eq!(event_in_tz.title, "Meeting");
        // 14:00 UTC = 09:00 EST (UTC-5)
        assert!(event_in_tz.start.contains("09:00:00"));
        assert!(event_in_tz.end.contains("10:00:00"));
    }

    #[test]
    fn test_event_to_timezone_london() {
        use chrono_tz::Europe::London;

        // Event at 14:00 UTC (14:00 GMT in winter, 15:00 BST in summer)
        let start = Utc.with_ymd_and_hms(2026, 2, 25, 14, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2026, 2, 25, 15, 0, 0).unwrap();

        let event = Event {
            id: "event-1".to_string(),
            title: "Meeting".to_string(),
            description: None,
            start,
            end,
            location: None,
            all_day: false,
            calendar: "test".to_string(),
        };

        let event_in_tz = event.to_timezone(&London);

        assert_eq!(event_in_tz.id, "event-1");
        // February is winter, so GMT (UTC+0)
        assert!(event_in_tz.start.contains("14:00:00"));
        assert!(event_in_tz.end.contains("15:00:00"));
    }

    #[test]
    fn test_event_in_timezone_serialization() {
        use chrono_tz::America::New_York;

        let start = Utc.with_ymd_and_hms(2026, 2, 25, 19, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2026, 2, 25, 20, 0, 0).unwrap();

        let event = Event {
            id: "event-1".to_string(),
            title: "Meeting".to_string(),
            description: Some("Team sync".to_string()),
            start,
            end,
            location: Some("Office".to_string()),
            all_day: false,
            calendar: "test".to_string(),
        };

        let event_in_tz = event.to_timezone(&New_York);
        let json = serde_json::to_string(&event_in_tz).unwrap();

        assert!(json.contains("\"id\":\"event-1\""));
        assert!(json.contains("\"title\":\"Meeting\""));
        // 19:00 UTC = 14:00 EST
        assert!(json.contains("14:00:00"));
    }
}
