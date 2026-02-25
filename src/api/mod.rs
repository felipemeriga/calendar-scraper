// API module - REST endpoints

pub mod auth;
mod handlers;
mod routes;

pub use routes::configure;
