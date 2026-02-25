mod api;
mod config;
mod filters;
mod ics;
mod models;

use actix_web::{web, App, HttpServer};
use api::auth::ApiTokenAuth;
use config::Config;
use std::env;
use tracing::{error, info};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load .env file if it exists
    dotenv::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load API token from environment variable
    let api_token = env::var("API_TOKEN").unwrap_or_else(|_| {
        error!("API_TOKEN environment variable not set. Using default token 'dev-token'");
        "dev-token".to_string()
    });

    // Load calendars configuration
    let config_path = env::var("CALENDARS_CONFIG").unwrap_or_else(|_| {
        info!("CALENDARS_CONFIG not set, using default: calendars.toml");
        "calendars.toml".to_string()
    });

    let config = match Config::from_file(&config_path) {
        Ok(config) => {
            if config.calendars.is_empty() {
                error!("No calendars configured in {}", config_path);
                error!("Please add at least one calendar to the configuration file");
                std::process::exit(1);
            }
            info!(
                "Loaded {} calendars from {}",
                config.calendars.len(),
                config_path
            );
            config
        }
        Err(e) => {
            error!("Failed to load config from {}: {}", config_path, e);
            error!("Please ensure {} exists and is valid TOML", config_path);
            std::process::exit(1);
        }
    };

    // Read host and port from environment variables with defaults
    // Default to 0.0.0.0 for Docker compatibility, but can be overridden
    let host = env::var("HOST").unwrap_or_else(|_| {
        info!("HOST environment variable not set, using default: 0.0.0.0");
        "0.0.0.0".to_string()
    });
    let port = env::var("PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or_else(|| {
            info!("PORT environment variable not set or invalid, using default: 8080");
            8080
        });

    info!("Starting Calendar Scraper API at http://{}:{}", host, port);
    info!("Health check: http://{}:{}/health", host, port);
    info!("API authentication enabled - use 'Authorization: Bearer <token>' header");
    info!("Available endpoints:");
    info!("  GET /api/v1/calendars - List all calendars");
    info!("  Weekly events:");
    info!("    GET /api/v1/calendars/{{name}}/events/weekly - From specific calendar");
    info!("    GET /api/v1/events/weekly/all - From all calendars");
    info!("  Daily events:");
    info!("    GET /api/v1/calendars/{{name}}/events/today - From specific calendar");
    info!("    GET /api/v1/events/today/all - From all calendars");
    info!("  Today + Tomorrow events:");
    info!("    GET /api/v1/calendars/{{name}}/events/today-and-tomorrow - From specific calendar");
    info!("    GET /api/v1/events/today-and-tomorrow/all - From all calendars");

    let config_data = web::Data::new(config);

    HttpServer::new(move || {
        App::new()
            .wrap(ApiTokenAuth::new(api_token.clone()))
            .app_data(config_data.clone())
            .configure(api::configure)
    })
    .bind((host.as_str(), port))?
    .run()
    .await
}
