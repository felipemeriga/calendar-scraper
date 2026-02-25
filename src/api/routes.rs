use crate::api::handlers::{
    get_all_calendars_today_and_tomorrow_events, get_all_calendars_today_events,
    get_all_calendars_weekly_events, get_calendar_today_and_tomorrow_events,
    get_calendar_today_events, get_calendar_weekly_events, health_check, list_calendars,
};
use actix_web::web;

/// Configure API routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            // Get events from all calendars
            .route(
                "/events/weekly/all",
                web::get().to(get_all_calendars_weekly_events),
            )
            .route(
                "/events/today/all",
                web::get().to(get_all_calendars_today_events),
            )
            .route(
                "/events/today-and-tomorrow/all",
                web::get().to(get_all_calendars_today_and_tomorrow_events),
            )
            // List available calendars
            .route("/calendars", web::get().to(list_calendars))
            // Get events from a specific calendar by name
            .route(
                "/calendars/{name}/events/weekly",
                web::get().to(get_calendar_weekly_events),
            )
            .route(
                "/calendars/{name}/events/today",
                web::get().to(get_calendar_today_events),
            )
            .route(
                "/calendars/{name}/events/today-and-tomorrow",
                web::get().to(get_calendar_today_and_tomorrow_events),
            ),
    )
    .route("/health", web::get().to(health_check));
}
