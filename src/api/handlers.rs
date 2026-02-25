use crate::config::Config;
use crate::filters::{
    filter_events_by_day, filter_events_by_week, get_current_day, get_current_week,
    get_today_and_tomorrow,
};
use crate::ics::{fetch_ics, IcsError};
use crate::models::{
    AllCalendarsDailyEventsResponse, AllCalendarsDailyEventsResponseWithTz,
    AllCalendarsWeeklyEventsResponse, AllCalendarsWeeklyEventsResponseWithTz, CalendarEvents,
    CalendarEventsWithTz, DailyEventsResponse, DailyEventsResponseWithTz, Event,
    WeeklyEventsResponse, WeeklyEventsResponseWithTz,
};
use actix_web::{web, HttpResponse, Responder};
use chrono_tz::Tz;
use serde::Deserialize;

/// Filter out cancelled events (events with "cancelled" in the title)
fn filter_cancelled_events(events: Vec<Event>) -> Vec<Event> {
    events
        .into_iter()
        .filter(|event| !event.title.to_lowercase().contains("cancelled"))
        .collect()
}

/// Handler for GET /health
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "service": "calendar-scraper"
    }))
}

#[derive(Deserialize)]
pub struct TimezoneQuery {
    pub timezone: Option<String>,
}

/// Handler for GET /api/v1/calendars
/// Lists all available calendars from the config
pub async fn list_calendars(config: web::Data<Config>) -> impl Responder {
    let calendars: Vec<_> = config
        .calendars
        .iter()
        .map(|c| {
            serde_json::json!({
                "name": c.name,
            })
        })
        .collect();

    HttpResponse::Ok().json(serde_json::json!({
        "calendars": calendars
    }))
}

/// Handler for GET /api/v1/calendars/:name/events/weekly
/// Get weekly events from a specific calendar
pub async fn get_calendar_weekly_events(
    path: web::Path<String>,
    query: web::Query<TimezoneQuery>,
    config: web::Data<Config>,
) -> impl Responder {
    let calendar_name = path.into_inner();

    // Get calendar from config
    let calendar = match config.get_calendar(&calendar_name) {
        Ok(cal) => cal,
        Err(_) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": format!("Calendar '{}' not found", calendar_name)
            }));
        }
    };

    let url = calendar.url.clone();
    let cal_name = calendar.name.clone();

    // Fetch and parse ICS calendar
    let events = match web::block(move || fetch_ics(&url, &cal_name)).await {
        Ok(result) => match result {
            Ok(events) => events,
            Err(e) => {
                let error_msg = match e {
                    IcsError::InvalidUrl(msg) => format!("Invalid URL: {}", msg),
                    IcsError::FetchError(msg) => format!("Failed to fetch calendar: {}", msg),
                    IcsError::ParseError(msg) => format!("Failed to parse calendar: {}", msg),
                };
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "error": error_msg
                }));
            }
        },
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to process request"
            }));
        }
    };

    // Get current week period
    let week = get_current_week();

    // Filter events for current week
    let filtered_events = filter_events_by_week(events, &week);

    // Filter out cancelled events
    let filtered_events = filter_cancelled_events(filtered_events);

    // Check if timezone conversion is requested
    if let Some(tz_str) = &query.timezone {
        match tz_str.parse::<Tz>() {
            Ok(tz) => {
                let events_in_tz: Vec<_> =
                    filtered_events.iter().map(|e| e.to_timezone(&tz)).collect();

                let response = WeeklyEventsResponseWithTz {
                    week,
                    events: events_in_tz,
                    timezone: tz_str.clone(),
                };

                return HttpResponse::Ok().json(response);
            }
            Err(_) => {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "error": format!("Invalid timezone: {}. Use IANA timezone names like 'America/New_York' or 'Europe/London'", tz_str)
                }));
            }
        }
    }

    // Build response (UTC times)
    let response = WeeklyEventsResponse {
        week,
        events: filtered_events,
    };

    HttpResponse::Ok().json(response)
}

/// Handler for GET /api/v1/events/weekly/all
/// Get weekly events from all calendars
pub async fn get_all_calendars_weekly_events(
    query: web::Query<TimezoneQuery>,
    config: web::Data<Config>,
) -> impl Responder {
    let week = get_current_week();
    let mut calendar_events_list = Vec::new();

    // Fetch events from each calendar
    for calendar in &config.calendars {
        let url = calendar.url.clone();
        let calendar_name = calendar.name.clone();

        // Fetch and parse ICS calendar
        let events = match web::block({
            let cal_name = calendar_name.clone();
            move || fetch_ics(&url, &cal_name)
        })
        .await
        {
            Ok(result) => match result {
                Ok(events) => events,
                Err(e) => {
                    // Log error but continue with other calendars
                    tracing::warn!("Failed to fetch calendar '{}': {:?}", calendar_name, e);
                    continue;
                }
            },
            Err(_) => {
                tracing::warn!("Failed to process request for calendar '{}'", calendar_name);
                continue;
            }
        };

        // Filter events for current week
        let filtered_events = filter_events_by_week(events, &week);

        // Filter out cancelled events
        let filtered_events = filter_cancelled_events(filtered_events);

        calendar_events_list.push((calendar_name, filtered_events));
    }

    // Check if timezone conversion is requested
    if let Some(tz_str) = &query.timezone {
        match tz_str.parse::<Tz>() {
            Ok(tz) => {
                let calendars_with_tz: Vec<CalendarEventsWithTz> = calendar_events_list
                    .into_iter()
                    .map(|(name, events)| {
                        let events_in_tz: Vec<_> =
                            events.iter().map(|e| e.to_timezone(&tz)).collect();
                        CalendarEventsWithTz {
                            calendar_name: name,
                            events: events_in_tz,
                        }
                    })
                    .collect();

                let response = AllCalendarsWeeklyEventsResponseWithTz {
                    week,
                    calendars: calendars_with_tz,
                    timezone: tz_str.clone(),
                };

                return HttpResponse::Ok().json(response);
            }
            Err(_) => {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "error": format!("Invalid timezone: {}. Use IANA timezone names like 'America/New_York' or 'Europe/London'", tz_str)
                }));
            }
        }
    }

    // Build response (UTC times)
    let calendars: Vec<CalendarEvents> = calendar_events_list
        .into_iter()
        .map(|(name, events)| CalendarEvents {
            calendar_name: name,
            events,
        })
        .collect();

    let response = AllCalendarsWeeklyEventsResponse { week, calendars };

    HttpResponse::Ok().json(response)
}

/// Handler for GET /api/v1/calendars/:name/events/today
/// Get today's events from a specific calendar
pub async fn get_calendar_today_events(
    path: web::Path<String>,
    query: web::Query<TimezoneQuery>,
    config: web::Data<Config>,
) -> impl Responder {
    let calendar_name = path.into_inner();

    // Get calendar from config
    let calendar = match config.get_calendar(&calendar_name) {
        Ok(cal) => cal,
        Err(_) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": format!("Calendar '{}' not found", calendar_name)
            }));
        }
    };

    let url = calendar.url.clone();
    let cal_name = calendar.name.clone();

    // Fetch and parse ICS calendar
    let events = match web::block(move || fetch_ics(&url, &cal_name)).await {
        Ok(result) => match result {
            Ok(events) => events,
            Err(e) => {
                let error_msg = match e {
                    IcsError::InvalidUrl(msg) => format!("Invalid URL: {}", msg),
                    IcsError::FetchError(msg) => format!("Failed to fetch calendar: {}", msg),
                    IcsError::ParseError(msg) => format!("Failed to parse calendar: {}", msg),
                };
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "error": error_msg
                }));
            }
        },
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to process request"
            }));
        }
    };

    // Get current day period
    let day = get_current_day();

    // Filter events for current day
    let filtered_events = filter_events_by_day(events, &day);

    // Filter out cancelled events
    let filtered_events = filter_cancelled_events(filtered_events);

    // Check if timezone conversion is requested
    if let Some(tz_str) = &query.timezone {
        match tz_str.parse::<Tz>() {
            Ok(tz) => {
                let events_in_tz: Vec<_> =
                    filtered_events.iter().map(|e| e.to_timezone(&tz)).collect();

                let response = DailyEventsResponseWithTz {
                    day,
                    events: events_in_tz,
                    timezone: tz_str.clone(),
                };

                return HttpResponse::Ok().json(response);
            }
            Err(_) => {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "error": format!("Invalid timezone: {}. Use IANA timezone names like 'America/New_York' or 'Europe/London'", tz_str)
                }));
            }
        }
    }

    // Build response (UTC times)
    let response = DailyEventsResponse {
        day,
        events: filtered_events,
    };

    HttpResponse::Ok().json(response)
}

/// Handler for GET /api/v1/events/today/all
/// Get today's events from all calendars
pub async fn get_all_calendars_today_events(
    query: web::Query<TimezoneQuery>,
    config: web::Data<Config>,
) -> impl Responder {
    let day = get_current_day();
    let mut calendar_events_list = Vec::new();

    // Fetch events from each calendar
    for calendar in &config.calendars {
        let url = calendar.url.clone();
        let calendar_name = calendar.name.clone();

        // Fetch and parse ICS calendar
        let events = match web::block({
            let cal_name = calendar_name.clone();
            move || fetch_ics(&url, &cal_name)
        })
        .await
        {
            Ok(result) => match result {
                Ok(events) => events,
                Err(e) => {
                    // Log error but continue with other calendars
                    tracing::warn!("Failed to fetch calendar '{}': {:?}", calendar_name, e);
                    continue;
                }
            },
            Err(_) => {
                tracing::warn!("Failed to process request for calendar '{}'", calendar_name);
                continue;
            }
        };

        // Filter events for current day
        let filtered_events = filter_events_by_day(events, &day);

        // Filter out cancelled events
        let filtered_events = filter_cancelled_events(filtered_events);

        calendar_events_list.push((calendar_name, filtered_events));
    }

    // Check if timezone conversion is requested
    if let Some(tz_str) = &query.timezone {
        match tz_str.parse::<Tz>() {
            Ok(tz) => {
                let calendars_with_tz: Vec<CalendarEventsWithTz> = calendar_events_list
                    .into_iter()
                    .map(|(name, events)| {
                        let events_in_tz: Vec<_> =
                            events.iter().map(|e| e.to_timezone(&tz)).collect();
                        CalendarEventsWithTz {
                            calendar_name: name,
                            events: events_in_tz,
                        }
                    })
                    .collect();

                let response = AllCalendarsDailyEventsResponseWithTz {
                    day,
                    calendars: calendars_with_tz,
                    timezone: tz_str.clone(),
                };

                return HttpResponse::Ok().json(response);
            }
            Err(_) => {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "error": format!("Invalid timezone: {}. Use IANA timezone names like 'America/New_York' or 'Europe/London'", tz_str)
                }));
            }
        }
    }

    // Build response (UTC times)
    let calendars: Vec<CalendarEvents> = calendar_events_list
        .into_iter()
        .map(|(name, events)| CalendarEvents {
            calendar_name: name,
            events,
        })
        .collect();

    let response = AllCalendarsDailyEventsResponse { day, calendars };

    HttpResponse::Ok().json(response)
}

/// Handler for GET /api/v1/calendars/:name/events/today-and-tomorrow
/// Get today and tomorrow's events from a specific calendar
pub async fn get_calendar_today_and_tomorrow_events(
    path: web::Path<String>,
    query: web::Query<TimezoneQuery>,
    config: web::Data<Config>,
) -> impl Responder {
    let calendar_name = path.into_inner();

    // Get calendar from config
    let calendar = match config.get_calendar(&calendar_name) {
        Ok(cal) => cal,
        Err(_) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": format!("Calendar '{}' not found", calendar_name)
            }));
        }
    };

    let url = calendar.url.clone();
    let cal_name = calendar.name.clone();

    // Fetch and parse ICS calendar
    let events = match web::block(move || fetch_ics(&url, &cal_name)).await {
        Ok(result) => match result {
            Ok(events) => events,
            Err(e) => {
                let error_msg = match e {
                    IcsError::InvalidUrl(msg) => format!("Invalid URL: {}", msg),
                    IcsError::FetchError(msg) => format!("Failed to fetch calendar: {}", msg),
                    IcsError::ParseError(msg) => format!("Failed to parse calendar: {}", msg),
                };
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "error": error_msg
                }));
            }
        },
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to process request"
            }));
        }
    };

    // Get today and tomorrow period
    let day = get_today_and_tomorrow();

    // Filter events for today and tomorrow
    let filtered_events = filter_events_by_day(events, &day);

    // Filter out cancelled events
    let filtered_events = filter_cancelled_events(filtered_events);

    // Check if timezone conversion is requested
    if let Some(tz_str) = &query.timezone {
        match tz_str.parse::<Tz>() {
            Ok(tz) => {
                let events_in_tz: Vec<_> =
                    filtered_events.iter().map(|e| e.to_timezone(&tz)).collect();

                let response = DailyEventsResponseWithTz {
                    day,
                    events: events_in_tz,
                    timezone: tz_str.clone(),
                };

                return HttpResponse::Ok().json(response);
            }
            Err(_) => {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "error": format!("Invalid timezone: {}. Use IANA timezone names like 'America/New_York' or 'Europe/London'", tz_str)
                }));
            }
        }
    }

    // Build response (UTC times)
    let response = DailyEventsResponse {
        day,
        events: filtered_events,
    };

    HttpResponse::Ok().json(response)
}

/// Handler for GET /api/v1/events/today-and-tomorrow/all
/// Get today and tomorrow's events from all calendars
pub async fn get_all_calendars_today_and_tomorrow_events(
    query: web::Query<TimezoneQuery>,
    config: web::Data<Config>,
) -> impl Responder {
    let day = get_today_and_tomorrow();
    let mut calendar_events_list = Vec::new();

    // Fetch events from each calendar
    for calendar in &config.calendars {
        let url = calendar.url.clone();
        let calendar_name = calendar.name.clone();

        // Fetch and parse ICS calendar
        let events = match web::block({
            let cal_name = calendar_name.clone();
            move || fetch_ics(&url, &cal_name)
        })
        .await
        {
            Ok(result) => match result {
                Ok(events) => events,
                Err(e) => {
                    // Log error but continue with other calendars
                    tracing::warn!("Failed to fetch calendar '{}': {:?}", calendar_name, e);
                    continue;
                }
            },
            Err(_) => {
                tracing::warn!("Failed to process request for calendar '{}'", calendar_name);
                continue;
            }
        };

        // Filter events for today and tomorrow
        let filtered_events = filter_events_by_day(events, &day);

        // Filter out cancelled events
        let filtered_events = filter_cancelled_events(filtered_events);

        calendar_events_list.push((calendar_name, filtered_events));
    }

    // Check if timezone conversion is requested
    if let Some(tz_str) = &query.timezone {
        match tz_str.parse::<Tz>() {
            Ok(tz) => {
                let calendars_with_tz: Vec<CalendarEventsWithTz> = calendar_events_list
                    .into_iter()
                    .map(|(name, events)| {
                        let events_in_tz: Vec<_> =
                            events.iter().map(|e| e.to_timezone(&tz)).collect();
                        CalendarEventsWithTz {
                            calendar_name: name,
                            events: events_in_tz,
                        }
                    })
                    .collect();

                let response = AllCalendarsDailyEventsResponseWithTz {
                    day,
                    calendars: calendars_with_tz,
                    timezone: tz_str.clone(),
                };

                return HttpResponse::Ok().json(response);
            }
            Err(_) => {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "error": format!("Invalid timezone: {}. Use IANA timezone names like 'America/New_York' or 'Europe/London'", tz_str)
                }));
            }
        }
    }

    // Build response (UTC times)
    let calendars: Vec<CalendarEvents> = calendar_events_list
        .into_iter()
        .map(|(name, events)| CalendarEvents {
            calendar_name: name,
            events,
        })
        .collect();

    let response = AllCalendarsDailyEventsResponse { day, calendars };

    HttpResponse::Ok().json(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App};

    #[actix_web::test]
    async fn test_health_check() {
        let app =
            test::init_service(App::new().route("/health", web::get().to(health_check))).await;

        let req = test::TestRequest::get().uri("/health").to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["status"], "ok");
        assert_eq!(body["service"], "calendar-scraper");
    }

    // Note: Testing with real ICS URL would require network access
    // For comprehensive testing, we'd use mockito to mock HTTP responses
}
