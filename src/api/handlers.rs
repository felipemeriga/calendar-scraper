use crate::filters::{filter_events_by_week, get_current_week};
use crate::ics::{fetch_ics, IcsError};
use crate::models::WeeklyEventsResponse;
use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct WeeklyEventsQuery {
    pub ics_url: String,
}

/// Handler for GET /api/v1/events/weekly?ics_url=<url>
pub async fn get_weekly_events(query: web::Query<WeeklyEventsQuery>) -> impl Responder {
    // Validate URL is not empty
    if query.ics_url.trim().is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "ics_url parameter is required"
        }));
    }

    let url = query.ics_url.clone();

    // Fetch and parse ICS calendar (blocking operation)
    let events = match web::block(move || fetch_ics(&url)).await {
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

    // Build response
    let response = WeeklyEventsResponse {
        week,
        events: filtered_events,
    };

    HttpResponse::Ok().json(response)
}

/// Handler for GET /health
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "service": "calendar-scraper"
    }))
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

    #[actix_web::test]
    async fn test_get_weekly_events_missing_url() {
        let app = test::init_service(
            App::new().route("/api/v1/events/weekly", web::get().to(get_weekly_events)),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/api/v1/events/weekly?ics_url=")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 400);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert!(body["error"].as_str().unwrap().contains("required"));
    }

    #[actix_web::test]
    async fn test_get_weekly_events_invalid_url() {
        let app = test::init_service(
            App::new().route("/api/v1/events/weekly", web::get().to(get_weekly_events)),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/api/v1/events/weekly?ics_url=not-a-valid-url")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 400);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert!(body.get("error").is_some());
    }

    // Note: Testing with real ICS URL would require network access
    // For comprehensive testing, we'd use mockito to mock HTTP responses
}
