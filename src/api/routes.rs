use crate::api::handlers::{get_weekly_events, health_check};
use actix_web::web;

/// Configure API routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/api/v1").route("/events/weekly", web::get().to(get_weekly_events)))
        .route("/health", web::get().to(health_check));
}
