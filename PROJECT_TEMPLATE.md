# New Rust Microservice Project Template

Quick start guide for creating a new microservice following the same patterns as calendar-scraper.

## Quick Start

```bash
# 1. Create new Rust project
cargo new --bin your-service-name
cd your-service-name

# 2. Initialize git
git init
git add .
git commit -m "chore: initial commit"

# 3. Create main branch
git branch -M main

# 4. Add remote (optional)
git remote add origin git@github.com:username/your-service-name.git
```

## Project Structure

Create this directory structure:

```
your-service-name/
├── .env.example
├── .gitignore
├── Cargo.toml
├── Cargo.lock
├── Dockerfile
├── .dockerignore
├── docker-compose.yml
├── README.md
├── CLAUDE.md                 # Project specifications
├── DEVELOPMENT_GUIDELINES.md  # Copy from calendar-scraper
├── DOCKER_TEMPLATE.md        # Copy from calendar-scraper
├── src/
│   ├── main.rs
│   ├── api/
│   │   ├── mod.rs
│   │   ├── auth.rs          # Authentication middleware
│   │   ├── handlers.rs      # Request handlers
│   │   └── routes.rs        # Route configuration
│   ├── models/
│   │   ├── mod.rs
│   │   ├── request.rs       # Request DTOs
│   │   └── response.rs      # Response DTOs
│   ├── services/            # Business logic
│   │   └── mod.rs
│   └── config/              # Configuration
│       └── mod.rs
└── tests/
    └── integration_test.rs
```

## Initial Cargo.toml

```toml
[package]
name = "your-service-name"
version = "0.1.0"
edition = "2021"

[dependencies]
# Web framework
actix-web = "4.9"
actix-rt = "2.10"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Date and time
chrono = { version = "0.4", features = ["serde"] }

# HTTP client
reqwest = { version = "0.12", features = ["blocking", "json"] }

# Error handling
thiserror = "1.0"

# Logging
tracing = "0.1"
tracing-subscriber = "0.3"

# Environment variables
dotenv = "0.15"

# UUID generation (if needed)
uuid = { version = "1.0", features = ["v4", "serde"] }

[dev-dependencies]
mockito = "1.5"
tempfile = "3.8"
```

## Initial .gitignore

```gitignore
# Rust
/target
**/*.rs.bk
*.pdb
Cargo.lock

# IDE
.idea/
.vscode/
*.swp
*.swo
*~

# OS
.DS_Store
Thumbs.db

# Environment & Secrets
.env
config.toml
secrets.toml
credentials.json

# Logs
*.log

# Other
tmp/
.cache/
```

## Minimal main.rs Template

```rust
mod api;
mod config;
mod models;
mod services;

use actix_web::{web, App, HttpServer};
use std::env;
use tracing::info;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load .env file if it exists
    dotenv::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Read configuration from environment
    let host = env::var("HOST").unwrap_or_else(|_| {
        info!("HOST environment variable not set, using default: 0.0.0.0");
        "0.0.0.0".to_string()
    });

    let port = env::var("PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or_else(|| {
            info!("PORT environment variable not set, using default: 8080");
            8080
        });

    let api_token = env::var("API_TOKEN").unwrap_or_else(|_| {
        info!("API_TOKEN environment variable not set. Using default token 'dev-token'");
        "dev-token".to_string()
    });

    info!("Starting Your Service at http://{}:{}", host, port);
    info!("Health check: http://{}:{}/health", host, port);
    info!("API authentication enabled - use 'Authorization: Bearer <token>' header");

    HttpServer::new(move || {
        App::new()
            // Add your middleware here
            // .wrap(auth::ApiTokenAuth::new(api_token.clone()))
            .configure(api::configure)
    })
    .bind((host.as_str(), port))?
    .run()
    .await
}
```

## src/api/mod.rs Template

```rust
pub mod handlers;
pub mod routes;
// pub mod auth;  // Uncomment if using authentication

pub use routes::configure;
```

## src/api/routes.rs Template

```rust
use crate::api::handlers;
use actix_web::web;

/// Configure API routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg
        .route("/health", web::get().to(handlers::health_check))
        .service(
            web::scope("/api/v1")
                // Add your routes here
                .route("/example", web::get().to(handlers::example))
        );
}
```

## src/api/handlers.rs Template

```rust
use actix_web::{HttpResponse, Responder};

/// Handler for GET /health
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "service": "your-service-name",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

/// Example handler for GET /api/v1/example
pub async fn example() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "message": "Hello from your-service-name!"
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App};

    #[actix_web::test]
    async fn test_health_check() {
        let app = test::init_service(
            App::new().route("/health", web::get().to(health_check))
        ).await;

        let req = test::TestRequest::get()
            .uri("/health")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["status"], "ok");
    }
}
```

## src/models/mod.rs Template

```rust
pub mod request;
pub mod response;

pub use request::*;
pub use response::*;
```

## src/models/response.rs Template

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub data: T,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiError {
    pub error: String,
}

impl ApiError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            error: message.into(),
        }
    }
}
```

## src/services/mod.rs Template

```rust
// Add your business logic modules here
// pub mod example_service;
```

## src/config/mod.rs Template

```rust
use serde::Deserialize;
use std::fs;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    ReadError(String),

    #[error("Failed to parse config: {0}")]
    ParseError(String),
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    // Add your configuration fields here
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path)
            .map_err(|e| ConfigError::ReadError(e.to_string()))?;

        toml::from_str(&content)
            .map_err(|e| ConfigError::ParseError(e.to_string()))
    }
}
```

## README.md Template

```markdown
# Your Service Name

Brief description of what this service does.

## Features

- Feature 1
- Feature 2
- Feature 3

## Installation

### Prerequisites
- Rust 1.70+ (edition 2021)
- Cargo

### Build from Source

\`\`\`bash
# Clone the repository
git clone <repository-url>
cd your-service-name

# Build the project
cargo build --release

# Run tests
cargo test

# Run the server
cargo run --release
\`\`\`

The server will start on `http://127.0.0.1:8080`

## Configuration

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `API_TOKEN` | Authentication token | `dev-token` |
| `HOST` | Server bind address | `0.0.0.0` |
| `PORT` | Server port | `8080` |

Create a `.env` file:

\`\`\`bash
cp .env.example .env
# Edit .env with your values
\`\`\`

## Docker Deployment

### Quick Start with Docker

\`\`\`bash
# Build the image
docker build -t your-service:latest .

# Run
docker run -d \
  --name your-service \
  -p 8080:8080 \
  -e API_TOKEN=your-token \
  your-service:latest
\`\`\`

### Using Docker Compose

\`\`\`bash
docker-compose up -d
\`\`\`

## API Endpoints

### Health Check
\`\`\`
GET /health
\`\`\`

Returns service status.

**Response:**
\`\`\`json
{
  "status": "ok",
  "service": "your-service-name",
  "version": "0.1.0"
}
\`\`\`

### Example Endpoint
\`\`\`
GET /api/v1/example
\`\`\`

Authentication required: `Authorization: Bearer YOUR_TOKEN`

**Response:**
\`\`\`json
{
  "message": "Hello from your-service-name!"
}
\`\`\`

## Development

### Run Locally

\`\`\`bash
cargo run
\`\`\`

### Run Tests

\`\`\`bash
cargo test
\`\`\`

### Code Quality

\`\`\`bash
# Format
cargo fmt

# Lint
cargo clippy -- -D warnings
\`\`\`

## License

[Your License]

## Author

[Your Name]
\`\`\`

## CLAUDE.md Template

Create a `CLAUDE.md` file with your project specifications:

\`\`\`markdown
# Your Service Name - Project Specifications

## Objective
Describe what this service does and why it exists.

## Architecture

### Main Modules

1. **Module 1** (`src/module1/`)
   - Responsibility
   - Key functions

2. **Module 2** (`src/module2/`)
   - Responsibility
   - Key functions

## Technical Requirements

### Test-Driven Development (TDD)
- All modules must have unit tests written BEFORE implementation
- Minimum test coverage: 80%
- Integration tests for API endpoints

### Technology Stack
- **Language**: Rust (edition 2021)
- **Web Framework**: Actix-web
- **Database**: [PostgreSQL/Redis/None]
- **External APIs**: [List any external services]

### API Endpoints

#### v1.0
\`\`\`
GET /api/v1/resource
POST /api/v1/resource
PUT /api/v1/resource/:id
DELETE /api/v1/resource/:id
\`\`\`

## Development Standards

### Language
- Everything in English: code, comments, commits, documentation

### Git Workflow
- Work in feature branches
- Small and atomic commits
- Conventional commits format
- Author: [your name]

### Code Style
- Follow Rust style guide
- Use `cargo fmt` before each commit
- Use `cargo clippy` for linting
- Zero warnings policy

### Error Handling
- Use `Result<T, E>` for operations that can fail
- Custom error types with thiserror
- Structured logging with tracing

## Security
- API token authentication
- Input validation
- Rate limiting
- No secrets in code

## Observability
- Structured logs (info, warn, error)
- Health check endpoint
\`\`\`

## Next Steps

1. **Copy template files**:
   - Copy `DEVELOPMENT_GUIDELINES.md` from calendar-scraper
   - Copy `DOCKER_TEMPLATE.md` from calendar-scraper

2. **Customize Dockerfile**:
   - Replace `YOUR_APP_NAME` with your binary name
   - Adjust ports if needed

3. **Update docker-compose.yml**:
   - Replace `your-service` with your service name
   - Add any databases or dependencies

4. **Create .env.example**:
   - List all environment variables
   - Provide safe defaults

5. **Write tests first** (TDD):
   \`\`\`bash
   # Create test
   # Write test that fails
   cargo test  # Should fail

   # Implement feature
   # Write code
   cargo test  # Should pass
   \`\`\`

6. **Build and run**:
   \`\`\`bash
   cargo build
   cargo run
   curl http://localhost:8080/health
   \`\`\`

7. **Dockerize**:
   \`\`\`bash
   docker build -t your-service:latest .
   docker run -p 8080:8080 your-service:latest
   \`\`\`

8. **Push to GitHub**:
   \`\`\`bash
   git remote add origin git@github.com:username/your-service.git
   git push -u origin main
   \`\`\`

## Useful Resources

- Calendar-scraper repository (reference implementation)
- DEVELOPMENT_GUIDELINES.md (best practices)
- DOCKER_TEMPLATE.md (Docker setup)
