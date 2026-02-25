mod api;
mod filters;
mod ics;
mod models;

use actix_web::{App, HttpServer};
use tracing::info;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let host = "127.0.0.1";
    let port = 8080;

    info!("Starting Calendar Scraper API at http://{}:{}", host, port);
    info!("Health check: http://{}:{}/health", host, port);
    info!(
        "Weekly events: http://{}:{}/api/v1/events/weekly?ics_url=<url>",
        host, port
    );

    HttpServer::new(|| App::new().configure(api::configure))
        .bind((host, port))?
        .run()
        .await
}
