# Calendar Scraper

A Rust-based REST API for fetching and filtering calendar events from ICS files. Currently supports filtering events by the current week, with plans to integrate with Google Calendar.

## Features

- 📅 Fetch events from ICS calendar URLs
- 🗓️ Filter events by current week (Monday 00:00 - Sunday 23:59 UTC)
- 🚀 Fast and efficient async REST API built with Actix-web
- ✅ Comprehensive test coverage (TDD approach)
- 📝 JSON response format

## API Endpoints

### Get Weekly Events
```
GET /api/v1/events/weekly?ics_url=<url>
```

Fetches events from an ICS calendar URL and returns only events occurring in the current week.

**Parameters:**
- `ics_url` (required): URL to the ICS calendar file

**Response:**
```json
{
  "week": {
    "start": "2026-02-23T00:00:00Z",
    "end": "2026-03-01T23:59:59Z"
  },
  "events": [
    {
      "id": "unique-event-id",
      "title": "Meeting Title",
      "description": "Event description",
      "start": "2026-02-25T14:00:00Z",
      "end": "2026-02-25T15:00:00Z",
      "location": "Office",
      "all_day": false
    }
  ]
}
```

### Health Check
```
GET /health
```

Returns the service status.

**Response:**
```json
{
  "status": "ok",
  "service": "calendar-scraper"
}
```

## Installation

### Prerequisites
- Rust 1.70+ (edition 2021)
- Cargo

### Build from Source

```bash
# Clone the repository
git clone <repository-url>
cd calendar-scraper

# Build the project
cargo build --release

# Run tests
cargo test

# Run the server
cargo run --release
```

The server will start on `http://127.0.0.1:8080`

## Usage

### Example with cURL

```bash
# Health check
curl http://127.0.0.1:8080/health

# Get weekly events from an ICS calendar
curl "http://127.0.0.1:8080/api/v1/events/weekly?ics_url=https://example.com/calendar.ics"
```

### Example with HTTPie

```bash
# Health check
http GET http://127.0.0.1:8080/health

# Get weekly events
http GET http://127.0.0.1:8080/api/v1/events/weekly ics_url=="https://example.com/calendar.ics"
```

## Architecture

The project follows a modular architecture with TDD (Test-Driven Development):

```
src/
├── models/         # Data structures (Event, WeeklyEventsResponse)
├── ics/            # ICS fetching and parsing
├── filters/        # Event filtering logic (current week)
├── api/            # REST API handlers and routes
└── main.rs         # Application entry point
```

### Key Components

- **Models**: Event and response DTOs with serde serialization
- **ICS Parser**: Fetches ICS files via HTTP and parses them using the icalendar crate
- **Week Filter**: Calculates current week boundaries and filters events
- **API**: Actix-web handlers with async/blocking operation support

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test module
cargo test models
cargo test ics
cargo test filters
cargo test api

# Run with output
cargo test -- --nocapture
```

### Code Quality

```bash
# Format code
cargo fmt

# Run linter
cargo clippy

# Check without building
cargo check
```

## Technology Stack

- **Language**: Rust (edition 2021)
- **Web Framework**: Actix-web 4.9
- **ICS Parsing**: icalendar 0.16
- **HTTP Client**: reqwest 0.12
- **Serialization**: serde + serde_json
- **Date/Time**: chrono 0.4
- **Error Handling**: thiserror
- **Logging**: tracing + tracing-subscriber
- **Testing**: cargo test + mockito

## Roadmap

### Phase 2: Google Calendar Integration
- [ ] OAuth2 authentication
- [ ] Bidirectional synchronization
- [ ] Event caching
- [ ] Webhooks for real-time updates

### Phase 3: Enhancements
- [ ] Support for multiple calendars
- [ ] Advanced filters (by category, location, etc)
- [ ] Export to different formats
- [ ] Web dashboard
- [ ] Docker support
- [ ] CI/CD pipeline

## Contributing

This project follows TDD principles. When contributing:

1. Write tests first
2. Implement functionality to pass tests
3. Ensure all existing tests pass
4. Run `cargo fmt` and `cargo clippy`
5. Maintain test coverage >= 80%

## License

[Add your license here]

## Author

Felipe Meriga
