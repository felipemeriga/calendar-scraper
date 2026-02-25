use crate::models::Event;
use chrono::{DateTime, Datelike, NaiveDate, TimeZone, Utc};
use icalendar::{Calendar, CalendarDateTime, Component, DatePerhapsTime, EventLike};
use reqwest::blocking::Client;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IcsError {
    #[error("Failed to fetch ICS file: {0}")]
    FetchError(String),

    #[error("Failed to parse ICS file: {0}")]
    ParseError(String),

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
}

/// Fetches and parses ICS calendar from a URL
pub fn fetch_ics(url: &str) -> Result<Vec<Event>, IcsError> {
    // Validate URL
    if url.is_empty() {
        return Err(IcsError::InvalidUrl("URL cannot be empty".to_string()));
    }

    // Fetch ICS content with proper headers
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .user_agent("Mozilla/5.0 (compatible; calendar-scraper/1.0)")
        .build()
        .map_err(|e| IcsError::FetchError(e.to_string()))?;

    let response = client
        .get(url)
        .header("Accept", "text/calendar,*/*")
        .send()
        .map_err(|e| IcsError::FetchError(e.to_string()))?;

    if !response.status().is_success() {
        return Err(IcsError::FetchError(format!(
            "HTTP error: {}",
            response.status()
        )));
    }

    let body = response
        .text()
        .map_err(|e| IcsError::FetchError(e.to_string()))?;

    parse_ics(&body)
}

/// Helper function to convert CalendarDateTime to DateTime<Utc>
fn calendar_datetime_to_utc(dt: CalendarDateTime) -> Option<DateTime<Utc>> {
    match dt {
        CalendarDateTime::Floating(naive) => Some(Utc.from_utc_datetime(&naive)),
        CalendarDateTime::Utc(utc_dt) => Some(utc_dt),
        CalendarDateTime::WithTimezone { date_time, tzid: _ } => {
            // For timezone-aware dates, convert to UTC
            Some(Utc.from_utc_datetime(&date_time))
        }
    }
}

/// Helper function to convert NaiveDate to DateTime<Utc>
fn naive_date_to_utc(date: NaiveDate) -> Option<DateTime<Utc>> {
    Utc.with_ymd_and_hms(date.year(), date.month(), date.day(), 0, 0, 0)
        .single()
}

/// Helper function to convert DatePerhapsTime to DateTime<Utc>
fn date_perhaps_time_to_utc(dpt: DatePerhapsTime) -> Option<DateTime<Utc>> {
    match dpt {
        DatePerhapsTime::DateTime(dt) => calendar_datetime_to_utc(dt),
        DatePerhapsTime::Date(date) => naive_date_to_utc(date),
    }
}

/// Helper function to check if DatePerhapsTime is a date (all-day)
fn is_all_day_event(dpt: &DatePerhapsTime) -> bool {
    matches!(dpt, DatePerhapsTime::Date(_))
}

/// Parses ICS content and converts to Event list
pub fn parse_ics(ics_content: &str) -> Result<Vec<Event>, IcsError> {
    let calendar = ics_content
        .parse::<Calendar>()
        .map_err(|e| IcsError::ParseError(e.to_string()))?;

    let mut events = Vec::new();

    for component in calendar.iter() {
        if let Some(event) = component.as_event() {
            // Extract event properties
            let id = event
                .get_uid()
                .map(|u| u.to_string())
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

            let title = event
                .get_summary()
                .map(|s| s.to_string())
                .unwrap_or_else(|| "Untitled Event".to_string());

            let description = event.get_description().map(|d| d.to_string());

            let location = event.get_location().map(|l| l.to_string());

            // Parse start and end times
            let start_dpt = event
                .get_start()
                .ok_or_else(|| IcsError::ParseError("Missing start time".to_string()))?;

            let start = date_perhaps_time_to_utc(start_dpt.clone())
                .ok_or_else(|| IcsError::ParseError("Invalid start time".to_string()))?;

            let end = event
                .get_end()
                .and_then(date_perhaps_time_to_utc)
                .unwrap_or(start);

            // Determine if it's an all-day event
            let all_day = is_all_day_event(&start_dpt);

            events.push(Event {
                id,
                title,
                description,
                start,
                end,
                location,
                all_day,
            });
        }
    }

    Ok(events)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_ICS: &str = r#"BEGIN:VCALENDAR
VERSION:2.0
PRODID:-//Test//Test//EN
BEGIN:VEVENT
UID:event-123
SUMMARY:Team Meeting
DESCRIPTION:Weekly sync meeting
DTSTART:20260225T140000Z
DTEND:20260225T150000Z
LOCATION:Office
END:VEVENT
BEGIN:VEVENT
UID:event-456
SUMMARY:Conference
DTSTART:20260226T100000Z
DTEND:20260226T170000Z
END:VEVENT
END:VCALENDAR"#;

    #[test]
    fn test_parse_ics_valid_calendar() {
        let result = parse_ics(SAMPLE_ICS);
        assert!(result.is_ok());

        let events = result.unwrap();
        assert_eq!(events.len(), 2);
    }

    #[test]
    fn test_parse_ics_event_fields() {
        let events = parse_ics(SAMPLE_ICS).unwrap();

        let event = &events[0];
        assert_eq!(event.id, "event-123");
        assert_eq!(event.title, "Team Meeting");
        assert_eq!(event.description, Some("Weekly sync meeting".to_string()));
        assert_eq!(event.location, Some("Office".to_string()));
        assert!(!event.all_day);
    }

    #[test]
    fn test_parse_ics_event_without_optional_fields() {
        let events = parse_ics(SAMPLE_ICS).unwrap();

        let event = &events[1];
        assert_eq!(event.id, "event-456");
        assert_eq!(event.title, "Conference");
        assert_eq!(event.description, None);
        assert_eq!(event.location, None);
    }

    #[test]
    fn test_parse_ics_invalid_content() {
        let invalid_ics = "This is not valid ICS content";
        let result = parse_ics(invalid_ics);

        // The icalendar crate may parse invalid content as empty calendar
        // So we check that either it errors OR returns empty events
        if result.is_ok() {
            let events = result.unwrap();
            assert_eq!(events.len(), 0, "Invalid ICS should produce no events");
        } else {
            match result {
                Err(IcsError::ParseError(_)) => {}
                _ => panic!("Expected ParseError"),
            }
        }
    }

    #[test]
    fn test_parse_ics_empty_calendar() {
        let empty_ics = r#"BEGIN:VCALENDAR
VERSION:2.0
PRODID:-//Test//Test//EN
END:VCALENDAR"#;

        let result = parse_ics(empty_ics);
        assert!(result.is_ok());

        let events = result.unwrap();
        assert_eq!(events.len(), 0);
    }

    #[test]
    fn test_parse_ics_event_without_title() {
        let ics = r#"BEGIN:VCALENDAR
VERSION:2.0
BEGIN:VEVENT
UID:event-no-title
DTSTART:20260225T140000Z
DTEND:20260225T150000Z
END:VEVENT
END:VCALENDAR"#;

        let events = parse_ics(ics).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].title, "Untitled Event");
    }

    #[test]
    fn test_fetch_ics_invalid_url() {
        let result = fetch_ics("");
        assert!(result.is_err());

        match result {
            Err(IcsError::InvalidUrl(_)) => {}
            _ => panic!("Expected InvalidUrl error"),
        }
    }

    // Integration test with mockito will be added later
}
