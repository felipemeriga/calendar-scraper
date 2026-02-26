# Development Guidelines for Rust Microservices

This document captures the development standards and best practices used in this project. Use this as a template for new microservices.

## Core Principles

### 1. Language & Communication
- **Everything in English**: code, comments, commits, documentation, variable names
- No exceptions - maintain consistency across all artifacts
- Makes code accessible to international teams

### 2. Test-Driven Development (TDD)
- Write tests BEFORE implementation
- Minimum test coverage: 80%
- Run tests before every commit: `cargo test`
- All tests must pass before merging

### 3. Code Quality Standards
```bash
# Before every commit, run:
cargo fmt              # Format code
cargo clippy -- -D warnings  # Zero warnings allowed
cargo test            # All tests must pass
```

### 4. Git Workflow

#### Branch Strategy
- Work in feature branches (never directly on main)
- Branch naming: `feature/description`, `fix/description`, `refactor/description`
- Small, atomic commits
- Merge to main via pull requests (or direct merge for solo projects)

#### Commit Message Format
Use conventional commits format:
```
<type>: <description>

[optional body]

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `test`: Adding or updating tests
- `refactor`: Code refactoring
- `docs`: Documentation changes
- `chore`: Maintenance tasks
- `perf`: Performance improvements

**Examples:**
```
feat: add OAuth2 authentication for Google Calendar API

- Implement authorization flow
- Add token storage with Redis
- Add token refresh mechanism

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>
```

```
fix: prevent duplicate recurring events from Outlook calendars

Skip RECURRENCE-ID instances to avoid duplicates.
Add deduplication using HashSet.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>
```

## Technology Stack Recommendations

### Core Dependencies
```toml
[dependencies]
# Web framework
actix-web = "4.9"
actix-rt = "2.10"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Date and time
chrono = { version = "0.4", features = ["serde"] }
chrono-tz = "0.10"  # If timezone support needed

# HTTP client
reqwest = { version = "0.12", features = ["blocking"] }

# Error handling
thiserror = "1.0"

# Logging
tracing = "0.1"
tracing-subscriber = "0.3"

# Environment variables
dotenv = "0.15"

# Config file parsing (if needed)
toml = "0.8"

[dev-dependencies]
mockito = "1.5"  # HTTP mocking
tempfile = "3.8"  # Temporary files for tests
```

### Project Structure
```
src/
├── main.rs              # Entry point, server setup
├── api/
│   ├── mod.rs          # API module
│   ├── auth.rs         # Authentication middleware
│   ├── handlers.rs     # Request handlers
│   └── routes.rs       # Route configuration
├── models/
│   ├── mod.rs          # Data models
│   ├── request.rs      # Request DTOs
│   └── response.rs     # Response DTOs
├── services/           # Business logic
│   └── mod.rs
└── config/            # Configuration
    └── mod.rs
```

## Code Style

### Error Handling
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Failed to fetch: {0}")]
    FetchError(String),

    #[error("Parse error: {0}")]
    ParseError(String),
}

// Use Result<T, E> for all fallible operations
pub fn fetch_data(url: &str) -> Result<Data, ApiError> {
    // ...
}
```

### Structured Logging
```rust
use tracing::{info, warn, error, debug};

// Initialize in main.rs
tracing_subscriber::fmt::init();

// Use throughout code
info!("Starting server at {}:{}", host, port);
warn!("Failed to fetch calendar '{}': {:?}", name, err);
error!("Critical error: {}", msg);
debug!("Processing {} events", count);
```

### Environment Variables
```rust
use std::env;

// Required variable
let api_token = env::var("API_TOKEN")
    .expect("API_TOKEN environment variable must be set");

// Optional with default
let host = env::var("HOST").unwrap_or_else(|_| {
    info!("HOST not set, using default: 0.0.0.0");
    "0.0.0.0".to_string()
});

// Parse with validation
let port = env::var("PORT")
    .ok()
    .and_then(|p| p.parse::<u16>().ok())
    .unwrap_or_else(|| {
        info!("PORT not set, using default: 8080");
        8080
    });
```

### Async/Await Patterns
```rust
use actix_web::{web, HttpResponse, Responder};

// Async handlers
pub async fn handler(data: web::Data<State>) -> impl Responder {
    // Use web::block for blocking operations
    let result = web::block(move || {
        expensive_operation()
    }).await;

    match result {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(e) => HttpResponse::InternalServerError().json(
            serde_json::json!({"error": e.to_string()})
        ),
    }
}
```

## Security Best Practices

### 1. API Authentication
```rust
// Use Bearer token authentication
Authorization: Bearer YOUR_TOKEN

// Implement as middleware
pub struct ApiTokenAuth {
    token: String,
}

// Skip auth for health check
if req.path() == "/health" {
    return Ok(req);
}
```

### 2. Input Validation
```rust
// Validate URLs before use
if !url.starts_with("https://") && !url.starts_with("http://") {
    return Err(ApiError::InvalidUrl("URL must start with http:// or https://".to_string()));
}

// Sanitize user input
let title = event.title.trim();
```

### 3. Secrets Management
```bash
# Never commit secrets
# Add to .gitignore:
.env
*.toml  # If contains sensitive data
credentials.json

# Use environment variables
API_TOKEN=xxx
DATABASE_URL=xxx
OAUTH_CLIENT_SECRET=xxx
```

### 4. Rate Limiting
```rust
// Implement for external API calls
use std::time::{Duration, Instant};

let mut last_request = Instant::now();
if last_request.elapsed() < Duration::from_millis(100) {
    tokio::time::sleep(Duration::from_millis(100)).await;
}
```

## Docker Best Practices

### Multi-Stage Dockerfile
```dockerfile
# Build stage
FROM rust:slim as builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy manifests and build dependencies first (caching layer)
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy source and build
COPY src ./src
RUN touch src/main.rs && \
    cargo build --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    tzdata \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/YOUR_APP /app/YOUR_APP

# Environment variables
ENV HOST=0.0.0.0
ENV PORT=8080

EXPOSE 8080

CMD ["/app/YOUR_APP"]
```

### .dockerignore
```
# Git
.git
.gitignore

# Rust build artifacts
target/
**/*.rs.bk
*.pdb

# IDE
.idea/
.vscode/

# OS
.DS_Store

# Environment and secrets
.env
*.toml  # If contains secrets

# Documentation
*.md
!README.md

# CI/CD
.github/
```

### docker-compose.yml
```yaml
version: '3.8'

services:
  app:
    image: your-app:latest
    container_name: your-app
    restart: unless-stopped
    ports:
      - "8080:8080"
    environment:
      - API_TOKEN=${API_TOKEN}
      - HOST=0.0.0.0
      - PORT=8080
      - RUST_LOG=${RUST_LOG:-info}
    volumes:
      - ./config.toml:/app/config/config.toml:ro
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s
```

## API Design Patterns

### Health Check Endpoint
```rust
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "service": "your-service-name"
    }))
}
```

### Versioned API Routes
```rust
// Configure versioned routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .route("/resource", web::get().to(handler))
    );
}
```

### Standard Response Format
```rust
// Success response
{
  "data": { /* payload */ }
}

// Error response
{
  "error": "Human readable error message",
  "code": "ERROR_CODE"  // Optional
}
```

### Pagination (if needed)
```rust
#[derive(Deserialize)]
pub struct PaginationQuery {
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

// Response
{
  "data": [...],
  "pagination": {
    "page": 1,
    "limit": 20,
    "total": 100
  }
}
```

## Testing Strategy

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_name() {
        // Arrange
        let input = "test";

        // Act
        let result = function_to_test(input);

        // Assert
        assert_eq!(result, expected);
    }
}
```

### Integration Tests
```rust
#[actix_web::test]
async fn test_endpoint() {
    let app = test::init_service(
        App::new().route("/api/v1/resource", web::get().to(handler))
    ).await;

    let req = test::TestRequest::get()
        .uri("/api/v1/resource")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}
```

### Mock External Services
```rust
use mockito::{mock, server_url};

#[tokio::test]
async fn test_external_api() {
    let _m = mock("GET", "/api/data")
        .with_status(200)
        .with_body(r#"{"status":"ok"}"#)
        .create();

    let result = fetch_data(&server_url()).await;
    assert!(result.is_ok());
}
```

## Documentation Standards

### README.md Structure
```markdown
# Project Name

Brief description

## Features
- Feature 1
- Feature 2

## Installation
## Configuration
## Usage
## API Endpoints
## Docker Deployment
## Development
## License
```

### Inline Documentation
```rust
/// Brief description of function
///
/// # Arguments
/// * `param` - Description of parameter
///
/// # Returns
/// Description of return value
///
/// # Errors
/// Description of when errors occur
///
/// # Examples
/// ```
/// let result = function(param);
/// ```
pub fn function(param: Type) -> Result<Return, Error> {
    // Implementation
}
```

## Performance Considerations

### 1. Async I/O
- Use async/await for all I/O operations
- Use `web::block` for CPU-intensive tasks
- Don't block the runtime

### 2. Connection Pooling
```rust
// For databases
use sqlx::PgPool;

let pool = PgPool::connect(&database_url).await?;
let pool_data = web::Data::new(pool);
```

### 3. Caching
```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

type Cache = Arc<RwLock<HashMap<String, CachedData>>>;
```

## Observability

### Structured Logs
```rust
use tracing::{info, instrument};

#[instrument]
async fn process_request(id: &str) -> Result<Response> {
    info!("Processing request");
    // Automatic span tracking
}
```

### Metrics (Optional)
```rust
use prometheus::{Counter, Histogram};

lazy_static! {
    static ref REQUEST_COUNTER: Counter =
        Counter::new("requests_total", "Total requests").unwrap();
}
```

## Deployment Checklist

- [ ] All tests passing (`cargo test`)
- [ ] Zero clippy warnings (`cargo clippy -- -D warnings`)
- [ ] Code formatted (`cargo fmt`)
- [ ] Environment variables documented
- [ ] Docker image builds successfully
- [ ] Health check endpoint works
- [ ] README updated
- [ ] Secrets not committed
- [ ] CHANGELOG updated (if applicable)

## Useful Commands

```bash
# Development
cargo run                          # Run locally
cargo watch -x run                # Auto-reload on changes

# Testing
cargo test                        # Run all tests
cargo test --test integration     # Run specific test
cargo test -- --nocapture         # Show println! output

# Code Quality
cargo fmt                         # Format code
cargo clippy                      # Lint
cargo clippy -- -D warnings       # Fail on warnings

# Build
cargo build                       # Debug build
cargo build --release            # Optimized build

# Docker
docker build -t app:latest .
docker run -p 8080:8080 app:latest
docker-compose up -d
docker-compose logs -f
```

## Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Actix-web Documentation](https://actix.rs/)
- [Cargo Book](https://doc.rust-lang.org/cargo/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)

---

**Remember**: Consistency is key. Follow these guidelines across all microservices for maintainability.
