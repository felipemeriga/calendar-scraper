use crate::models::Event;
use chrono::{DateTime, Datelike, Duration, NaiveDate, TimeZone, Utc};
use chrono_tz::America::Sao_Paulo;
use chrono_tz::Tz;
use icalendar::{Calendar, CalendarDateTime, Component, DatePerhapsTime, EventLike};
use regex::Regex;
use reqwest::blocking::Client;
use rrule::RRuleSet;
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
pub fn fetch_ics(url: &str, calendar_name: &str) -> Result<Vec<Event>, IcsError> {
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

    parse_ics_for_test(&body, calendar_name)
}

/// Maps Microsoft timezone names to IANA timezone names
fn map_microsoft_timezone(ms_tz: &str) -> &str {
    match ms_tz {
        "Eastern Standard Time" => "America/New_York",
        "Pacific Standard Time" => "America/Los_Angeles",
        "Central Standard Time" => "America/Chicago",
        "Mountain Standard Time" => "America/Denver",
        "E. South America Standard Time" => "America/Sao_Paulo",
        "GMT Standard Time" => "Europe/London",
        "W. Europe Standard Time" => "Europe/Paris",
        "UTC" => "UTC",
        // Add more mappings as needed
        _ => ms_tz, // Return as-is if no mapping found
    }
}

/// Helper function to convert CalendarDateTime to DateTime<Utc>
fn calendar_datetime_to_utc(dt: CalendarDateTime) -> Option<DateTime<Utc>> {
    match dt {
        CalendarDateTime::Floating(naive) => {
            // Floating time - treat as UTC
            Some(Utc.from_utc_datetime(&naive))
        }
        CalendarDateTime::Utc(utc_dt) => Some(utc_dt),
        CalendarDateTime::WithTimezone { date_time, tzid } => {
            // Map Microsoft timezone names to IANA names
            let iana_tz = map_microsoft_timezone(&tzid);

            // Parse timezone and convert to UTC
            if let Ok(tz) = iana_tz.parse::<Tz>() {
                // Create timezone-aware datetime
                if let Some(tz_dt) = tz.from_local_datetime(&date_time).single() {
                    // Convert to UTC
                    return Some(tz_dt.with_timezone(&Utc));
                }
            }
            // Fallback: treat as UTC if timezone parsing fails
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

/// Extracts only meeting links from description text
fn extract_meeting_links(description: Option<String>) -> Option<String> {
    description.and_then(|desc| {
        // Match URLs for common meeting platforms, including those in angle brackets
        let link_regex = Regex::new(r"(?:https?://[^\s<>]*(?:teams\.microsoft\.com|zoom\.us|meet\.google\.com|webex\.com)[^\s<>]*)").unwrap();

        // Also match URLs inside angle brackets <...>
        let bracket_regex = Regex::new(r"<(https?://[^>]+(?:teams\.microsoft\.com|zoom\.us|meet\.google\.com|webex\.com)[^>]*)>").unwrap();

        let mut links: Vec<String> = Vec::new();

        // Extract from angle brackets first
        for cap in bracket_regex.captures_iter(&desc) {
            if let Some(url) = cap.get(1) {
                links.push(url.as_str().to_string());
            }
        }

        // Then extract direct links
        for m in link_regex.find_iter(&desc) {
            let url = m.as_str();
            // Skip if already added (was in brackets)
            if !links.iter().any(|l| url.contains(l) || l.contains(url)) {
                links.push(url.to_string());
            }
        }

        if links.is_empty() {
            None
        } else {
            Some(links.join("\n"))
        }
    })
}

/// Adds hardcoded CXVH Standup events (Monday-Thursday, 13:30-14:00 BRT)
/// Only adds events for "cosm" calendar
fn add_hardcoded_events(
    events: &mut Vec<Event>,
    calendar_name: &str,
    expansion_start: DateTime<Utc>,
    expansion_end: DateTime<Utc>,
) {
    // Only add hardcoded events for cosm calendar
    if calendar_name != "cosm" {
        return;
    }

    // CXVH Standup: Monday-Thursday at 13:30-14:00 Brazil time
    let meeting_link = "https://teams.microsoft.com/l/meetup-join/19%3ameeting_M2YxMWY0NDEtYjcwMS00MjgwLTk2YWItOTNiMmE2ZmE0ODli%40thread.v2/0?context=%7b%22Tid%22%3a%22fa018d31-a501-4849-b705-4ead72d30235%22%2c%22Oid%22%3a%22f96aec7b-b2e8-4da1-af20-e34f785ef164%22%7d";

    let mut current_date = expansion_start.with_timezone(&Sao_Paulo).date_naive();
    let end_date = expansion_end.with_timezone(&Sao_Paulo).date_naive();

    let mut occurrence_index = 0;
    while current_date <= end_date {
        // Only add for Monday-Thursday
        let weekday = current_date.weekday();
        let is_weekday = matches!(
            weekday,
            chrono::Weekday::Mon
                | chrono::Weekday::Tue
                | chrono::Weekday::Wed
                | chrono::Weekday::Thu
        );

        if is_weekday {
            // Create event at 13:30 BRT
            if let Some(start_brt) = Sao_Paulo
                .with_ymd_and_hms(
                    current_date.year(),
                    current_date.month(),
                    current_date.day(),
                    13,
                    30,
                    0,
                )
                .single()
            {
                let start_utc = start_brt.with_timezone(&Utc);
                let end_utc = start_utc + Duration::minutes(30);

                events.push(Event {
                    id: format!("hardcoded-cxvh-standup-{}", occurrence_index),
                    title: "CXVH Standup".to_string(),
                    description: Some(meeting_link.to_string()),
                    start: start_utc,
                    end: end_utc,
                    location: Some("Microsoft Teams Meeting".to_string()),
                    all_day: false,
                    calendar: calendar_name.to_string(),
                });

                occurrence_index += 1;
            }
        }

        current_date += Duration::days(1);
    }
}

/// Parses ICS content and converts to Event list
/// Expands recurring events for the next year
/// If `include_hardcoded` is true, adds hardcoded events (like CXVH Standup)
fn parse_ics_internal(
    ics_content: &str,
    calendar_name: &str,
    include_hardcoded: bool,
) -> Result<Vec<Event>, IcsError> {
    let calendar = ics_content
        .parse::<Calendar>()
        .map_err(|e| IcsError::ParseError(e.to_string()))?;

    let mut events = Vec::new();
    let mut seen_event_ids = std::collections::HashSet::new();

    // Define expansion window: 1 year from now
    let now = Utc::now();
    // Start from 30 days ago to include recent past events (for weekly queries)
    let expansion_start = now - Duration::days(30);
    let expansion_end = now + Duration::days(365);

    for component in calendar.iter() {
        if let Some(event) = component.as_event() {
            // Skip modified instances (RECURRENCE-ID) to avoid duplicates
            // Modified instances are already handled by the icalendar library's expansion
            if event.property_value("RECURRENCE-ID").is_some() {
                continue;
            }

            // Extract base event properties
            let base_id = event
                .get_uid()
                .map(|u| u.to_string())
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

            let title = event
                .get_summary()
                .map(|s| s.to_string())
                .unwrap_or_else(|| "Untitled Event".to_string());

            // Extract only meeting links from description
            let description = extract_meeting_links(event.get_description().map(|d| d.to_string()));
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

            let all_day = is_all_day_event(&start_dpt);
            let duration = end - start;

            // Check for RRULE (recurring events)
            let rrule_str = event.property_value("RRULE");

            if let Some(rrule) = rrule_str {
                // Parse and expand recurring event
                if let Ok(occurrences) = expand_recurring_event(rrule, &start, expansion_end) {
                    for (index, occurrence_start) in occurrences.iter().enumerate() {
                        let occurrence_end = *occurrence_start + duration;
                        let event_id = format!("{}-occurrence-{}", base_id, index);

                        // Skip if we've already seen this event ID (deduplication)
                        if seen_event_ids.contains(&event_id) {
                            continue;
                        }
                        seen_event_ids.insert(event_id.clone());

                        events.push(Event {
                            id: event_id,
                            title: title.clone(),
                            description: description.clone(),
                            start: *occurrence_start,
                            end: occurrence_end,
                            location: location.clone(),
                            all_day,
                            calendar: calendar_name.to_string(),
                        });
                    }
                } else {
                    // If RRULE parsing fails, just add the base event
                    if !seen_event_ids.contains(&base_id) {
                        seen_event_ids.insert(base_id.clone());
                        events.push(Event {
                            id: base_id,
                            title,
                            description,
                            start,
                            end,
                            location,
                            all_day,
                            calendar: calendar_name.to_string(),
                        });
                    }
                }
            } else {
                // Non-recurring event
                if !seen_event_ids.contains(&base_id) {
                    seen_event_ids.insert(base_id.clone());
                    events.push(Event {
                        id: base_id,
                        title,
                        description,
                        start,
                        end,
                        location,
                        all_day,
                        calendar: calendar_name.to_string(),
                    });
                }
            }
        }
    }

    // Add hardcoded events (only if requested and for cosm calendar)
    if include_hardcoded {
        add_hardcoded_events(&mut events, calendar_name, expansion_start, expansion_end);
    }

    Ok(events)
}

/// Public wrapper that includes hardcoded events
pub fn parse_ics_for_test(ics_content: &str, calendar_name: &str) -> Result<Vec<Event>, IcsError> {
    parse_ics_internal(ics_content, calendar_name, true)
}

/// Expands a recurring event based on RRULE
fn expand_recurring_event(
    rrule_str: &str,
    start: &DateTime<Utc>,
    until: DateTime<Utc>,
) -> Result<Vec<DateTime<Utc>>, IcsError> {
    // Build DTSTART line in the format expected by rrule crate
    let dtstart = format!("DTSTART:{}\n", start.format("%Y%m%dT%H%M%SZ"));

    // Build the full rrule string
    let rrule_full = format!("{}RRULE:{}", dtstart, rrule_str);

    // Parse the rrule
    let rrule_set: RRuleSet = rrule_full
        .parse()
        .map_err(|e| IcsError::ParseError(format!("Failed to parse RRULE: {}", e)))?;

    // Generate occurrences up to the expansion date
    let occurrences: Vec<DateTime<Utc>> = rrule_set
        .into_iter()
        .take_while(|dt| dt <= &until)
        .map(|dt| dt.with_timezone(&Utc))
        .collect();

    Ok(occurrences)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Timelike;

    // Test helper that excludes hardcoded events
    fn parse_ics_for_test(ics_content: &str) -> Result<Vec<Event>, IcsError> {
        parse_ics_internal(ics_content, "test", false)
    }

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
        let result = parse_ics_for_test(SAMPLE_ICS);
        assert!(result.is_ok());

        let events = result.unwrap();
        assert_eq!(events.len(), 2);
    }

    #[test]
    fn test_parse_ics_event_fields() {
        let events = parse_ics_for_test(SAMPLE_ICS).unwrap();

        let event = &events[0];
        assert_eq!(event.id, "event-123");
        assert_eq!(event.title, "Team Meeting");
        // Description now only contains meeting links, this event has none
        assert_eq!(event.description, None);
        assert_eq!(event.location, Some("Office".to_string()));
        assert!(!event.all_day);
    }

    #[test]
    fn test_parse_ics_event_without_optional_fields() {
        let events = parse_ics_for_test(SAMPLE_ICS).unwrap();

        let event = &events[1];
        assert_eq!(event.id, "event-456");
        assert_eq!(event.title, "Conference");
        assert_eq!(event.description, None);
        assert_eq!(event.location, None);
    }

    #[test]
    fn test_parse_ics_invalid_content() {
        let invalid_ics = "This is not valid ICS content";
        let result = parse_ics_for_test(invalid_ics);

        // The icalendar crate may parse invalid content as empty calendar
        // So we check that either it errors OR returns empty events
        if let Ok(events) = result {
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

        let result = parse_ics_for_test(empty_ics);
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

        let events = parse_ics_for_test(ics).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].title, "Untitled Event");
    }

    #[test]
    fn test_fetch_ics_invalid_url() {
        let result = fetch_ics("", "test");
        assert!(result.is_err());

        match result {
            Err(IcsError::InvalidUrl(_)) => {}
            _ => panic!("Expected InvalidUrl error"),
        }
    }

    #[test]
    fn test_parse_ics_with_timezone_aware_event() {
        // Event in EST timezone: 2026-02-27 14:00:00 EST = 2026-02-27 19:00:00 UTC
        let ics_with_tz = r#"BEGIN:VCALENDAR
VERSION:2.0
PRODID:-//Test//Test//EN
BEGIN:VEVENT
UID:event-tz-123
SUMMARY:EST Meeting
DTSTART;TZID=America/New_York:20260227T140000
DTEND;TZID=America/New_York:20260227T150000
END:VEVENT
END:VCALENDAR"#;

        let result = parse_ics_for_test(ics_with_tz);
        assert!(result.is_ok());

        let events = result.unwrap();
        assert_eq!(events.len(), 1);

        let event = &events[0];
        assert_eq!(event.title, "EST Meeting");

        // Verify the time is correctly converted to UTC
        // 14:00 EST (UTC-5) = 19:00 UTC
        assert_eq!(event.start.hour(), 19);
        assert_eq!(event.end.hour(), 20);
    }

    #[test]
    fn test_parse_ics_with_utc_event() {
        let ics_utc = r#"BEGIN:VCALENDAR
VERSION:2.0
PRODID:-//Test//Test//EN
BEGIN:VEVENT
UID:event-utc-456
SUMMARY:UTC Meeting
DTSTART:20260227T140000Z
DTEND:20260227T150000Z
END:VEVENT
END:VCALENDAR"#;

        let result = parse_ics_for_test(ics_utc);
        assert!(result.is_ok());

        let events = result.unwrap();
        assert_eq!(events.len(), 1);

        let event = &events[0];
        assert_eq!(event.title, "UTC Meeting");

        // Verify UTC time is preserved
        assert_eq!(event.start.hour(), 14);
        assert_eq!(event.end.hour(), 15);
    }

    #[test]
    fn test_parse_ics_all_day_event() {
        let ics_all_day = r#"BEGIN:VCALENDAR
VERSION:2.0
PRODID:-//Test//Test//EN
BEGIN:VEVENT
UID:event-all-day-789
SUMMARY:All Day Event
DTSTART;VALUE=DATE:20260227
DTEND;VALUE=DATE:20260228
END:VEVENT
END:VCALENDAR"#;

        let result = parse_ics_for_test(ics_all_day);
        assert!(result.is_ok());

        let events = result.unwrap();
        assert_eq!(events.len(), 1);

        let event = &events[0];
        assert_eq!(event.title, "All Day Event");
        assert!(event.all_day);

        // All-day events should start at 00:00:00
        assert_eq!(event.start.hour(), 0);
        assert_eq!(event.start.minute(), 0);
    }

    #[test]
    fn test_parse_ics_with_rrule() {
        // Weekly recurring event
        let ics_recurring = r#"BEGIN:VCALENDAR
VERSION:2.0
PRODID:-//Test//Test//EN
BEGIN:VEVENT
UID:recurring-event-123
SUMMARY:Weekly Meeting
DTSTART:20260223T140000Z
DTEND:20260223T150000Z
RRULE:FREQ=WEEKLY;COUNT=3;BYDAY=MO
END:VEVENT
END:VCALENDAR"#;

        let result = parse_ics_for_test(ics_recurring);
        assert!(result.is_ok());

        let events = result.unwrap();
        // Should expand to 3 occurrences
        assert_eq!(events.len(), 3);

        // All should have the same title
        assert!(events.iter().all(|e| e.title == "Weekly Meeting"));

        // Each should have a unique ID with occurrence index
        assert!(events[0].id.contains("-occurrence-0"));
        assert!(events[1].id.contains("-occurrence-1"));
        assert!(events[2].id.contains("-occurrence-2"));
    }

    #[test]
    fn test_parse_ics_with_microsoft_timezone() {
        // Test Microsoft timezone name conversion
        let ics_ms_tz = r#"BEGIN:VCALENDAR
VERSION:2.0
PRODID:-//Microsoft Exchange//
BEGIN:VTIMEZONE
TZID:Eastern Standard Time
BEGIN:STANDARD
DTSTART:16010101T020000
TZOFFSETFROM:-0400
TZOFFSETTO:-0500
END:STANDARD
END:VTIMEZONE
BEGIN:VEVENT
UID:ms-event-456
SUMMARY:Microsoft Event
DTSTART;TZID=Eastern Standard Time:20260225T140000
DTEND;TZID=Eastern Standard Time:20260225T150000
END:VEVENT
END:VCALENDAR"#;

        let result = parse_ics_for_test(ics_ms_tz);
        assert!(result.is_ok());

        let events = result.unwrap();
        assert_eq!(events.len(), 1);

        let event = &events[0];
        assert_eq!(event.title, "Microsoft Event");

        // 14:00 EST (UTC-5) = 19:00 UTC
        assert_eq!(event.start.hour(), 19);
    }

    // Integration test with mockito will be added later
}
