mod api;
mod filters;
mod ics;
mod models;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("Calendar Scraper API - Starting...");

    // TODO: Initialize API server
    Ok(())
}
