# Calendar Scraper

A Rust-based REST API for fetching and filtering calendar events from ICS files. Supports filtering events by the current week, multiple calendar management, and timezone conversion.

## Features

- 📅 Fetch events from ICS calendar URLs
- 🗂️ **Multiple calendar support** - manage multiple calendars from a TOML config file
- 🔐 API token authentication for secure access
- 🗓️ Filter events by current week (Monday 00:00 - Sunday 23:59 UTC)
- 🌍 Timezone support - properly handles timezone-aware ICS events (EST, PST, etc.)
- ⏰ Convert event times to user's timezone via query parameter
- 🚀 Fast and efficient async REST API built with Actix-web
- ✅ Comprehensive test coverage (TDD approach)
- 📝 JSON response format

## API Endpoints

> **Note**: All API endpoints (except `/health`) require authentication using an API token in the `Authorization` header:
> ```
> Authorization: Bearer YOUR_API_TOKEN
> ```

### List Calendars
```
GET /api/v1/calendars
```

Returns a list of all configured calendars.

**Response:**
```json
{
  "calendars": [
    { "name": "work" },
    { "name": "personal" }
  ]
}
```

### Get Weekly Events from Specific Calendar
```
GET /api/v1/calendars/{name}/events/weekly?timezone=<tz>
```

Fetches events from a specific calendar configured in `calendars.toml` and returns only events occurring in the current week.

**Parameters:**
- `name` (path parameter, required): Name of the calendar as configured in calendars.toml
- `timezone` (query parameter, optional): IANA timezone name (e.g., `America/New_York`, `Europe/London`)

### Get Today's Events from Specific Calendar
```
GET /api/v1/calendars/{name}/events/today?timezone=<tz>
```

Fetches events from a specific calendar and returns only events occurring today (00:00:00 to 23:59:59 UTC).

**Parameters:**
- `name` (path parameter, required): Name of the calendar as configured in calendars.toml
- `timezone` (query parameter, optional): IANA timezone name

### Get Today + Tomorrow Events from Specific Calendar
```
GET /api/v1/calendars/{name}/events/today-and-tomorrow?timezone=<tz>
```

Fetches events from a specific calendar and returns events occurring today and tomorrow.

**Parameters:**
- `name` (path parameter, required): Name of the calendar as configured in calendars.toml
- `timezone` (query parameter, optional): IANA timezone name

### Get Weekly Events from All Calendars
```
GET /api/v1/events/weekly/all?timezone=<tz>
```

Fetches weekly events from all configured calendars and returns them grouped by calendar.

**Parameters:**
- `timezone` (optional): IANA timezone name

### Get Today's Events from All Calendars
```
GET /api/v1/events/today/all?timezone=<tz>
```

Fetches today's events from all configured calendars and returns them grouped by calendar.

**Parameters:**
- `timezone` (optional): IANA timezone name

### Get Today + Tomorrow Events from All Calendars
```
GET /api/v1/events/today-and-tomorrow/all?timezone=<tz>
```

Fetches today and tomorrow's events from all configured calendars and returns them grouped by calendar.

**Parameters:**
- `timezone` (optional): IANA timezone name

**Response:**
```json
{
  "week": {
    "start": "2026-02-23T00:00:00Z",
    "end": "2026-03-01T23:59:59Z"
  },
  "calendars": [
    {
      "calendar_name": "work",
      "events": [
        {
          "id": "event-1",
          "title": "Team Meeting",
          "description": null,
          "start": "2026-02-25T14:00:00Z",
          "end": "2026-02-25T15:00:00Z",
          "location": "Office",
          "all_day": false,
          "calendar": "work"
        }
      ]
    },
    {
      "calendar_name": "personal",
      "events": [
        {
          "id": "event-2",
          "title": "Dentist Appointment",
          "description": null,
          "start": "2026-02-26T10:00:00Z",
          "end": "2026-02-26T11:00:00Z",
          "location": null,
          "all_day": false,
          "calendar": "personal"
        }
      ]
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

## Configuration

### 1. Set up API Token

The API requires an authentication token. Set it via environment variable:

```bash
# Copy the example env file
cp .env.example .env

# Edit .env and set your API token
# API_TOKEN=your-secure-api-token-here
```

Or export it directly:
```bash
export API_TOKEN="your-secure-token"
```

**Important**: Generate a secure random token for production use.

### 2. Configure Calendars

Create a `calendars.toml` file to manage your calendars:

```bash
# Copy the example file
cp calendars.toml.example calendars.toml

# Edit calendars.toml with your calendar URLs
```

Example `calendars.toml`:
```toml
[[calendars]]
name = "work"
url = "https://outlook.office365.com/owa/calendar/YOUR_ID/calendar.ics"

[[calendars]]
name = "personal"
url = "https://calendar.google.com/calendar/ical/YOUR_EMAIL/public/basic.ics"
```

**Security Note**: The `calendars.toml` file is gitignored by default to prevent accidentally committing sensitive calendar URLs.

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `API_TOKEN` | Authentication token for API access | `dev-token` (dev only) |
| `CALENDARS_CONFIG` | Path to calendars TOML file | `calendars.toml` |
| `HOST` | Server host | `127.0.0.1` |
| `PORT` | Server port | `8080` |

## Usage

### Example with cURL

```bash
# Set your API token
export API_TOKEN="your-api-token"

# Health check (no auth required)
curl http://127.0.0.1:8080/health

# List all configured calendars
curl -H "Authorization: Bearer $API_TOKEN" \
  http://127.0.0.1:8080/api/v1/calendars

# Get weekly events from all calendars (UTC times)
curl -H "Authorization: Bearer $API_TOKEN" \
  http://127.0.0.1:8080/api/v1/events/weekly/all

# Get weekly events from all calendars with timezone conversion
curl -H "Authorization: Bearer $API_TOKEN" \
  "http://127.0.0.1:8080/api/v1/events/weekly/all?timezone=America/New_York"

# Get weekly events from a specific calendar
curl -H "Authorization: Bearer $API_TOKEN" \
  http://127.0.0.1:8080/api/v1/calendars/work/events/weekly

# Get weekly events from a specific calendar in EST
curl -H "Authorization: Bearer $API_TOKEN" \
  "http://127.0.0.1:8080/api/v1/calendars/work/events/weekly?timezone=America/New_York"

# Get today's events from all calendars
curl -H "Authorization: Bearer $API_TOKEN" \
  http://127.0.0.1:8080/api/v1/events/today/all

# Get today's events from specific calendar
curl -H "Authorization: Bearer $API_TOKEN" \
  http://127.0.0.1:8080/api/v1/calendars/work/events/today

# Get today + tomorrow events from all calendars
curl -H "Authorization: Bearer $API_TOKEN" \
  http://127.0.0.1:8080/api/v1/events/today-and-tomorrow/all

# Get today + tomorrow events from specific calendar with timezone
curl -H "Authorization: Bearer $API_TOKEN" \
  "http://127.0.0.1:8080/api/v1/calendars/personal/events/today-and-tomorrow?timezone=America/New_York"
```

### Example with HTTPie

```bash
# Set your API token
export API_TOKEN="your-api-token"

# Health check (no auth required)
http GET http://127.0.0.1:8080/health

# List calendars
http GET http://127.0.0.1:8080/api/v1/calendars \
  Authorization:"Bearer $API_TOKEN"

# Get all calendar events
http GET http://127.0.0.1:8080/api/v1/events/weekly/all \
  Authorization:"Bearer $API_TOKEN"

# Get events from specific calendar with timezone
http GET http://127.0.0.1:8080/api/v1/calendars/work/events/weekly \
  Authorization:"Bearer $API_TOKEN" \
  timezone=="America/New_York"

# Get today's events
http GET http://127.0.0.1:8080/api/v1/events/today/all \
  Authorization:"Bearer $API_TOKEN"

# Get today + tomorrow events with timezone
http GET http://127.0.0.1:8080/api/v1/events/today-and-tomorrow/all \
  Authorization:"Bearer $API_TOKEN" \
  timezone=="America/New_York"
```

## Architecture

The project follows a modular architecture with TDD (Test-Driven Development):

```
src/
├── models/         # Data structures (Event, WeeklyEventsResponse)
├── ics/            # ICS fetching and parsing
├── filters/        # Event filtering logic (current week)
├── config/         # Calendar configuration from TOML
├── api/
│   ├── auth.rs     # API token authentication middleware
│   ├── handlers.rs # Request handlers
│   ├── routes.rs   # Route configuration
│   └── mod.rs
└── main.rs         # Application entry point
```

### Key Components

- **Models**: Event and response DTOs with serde serialization
- **Config**: TOML-based calendar configuration management
- **Authentication**: Token-based API authentication middleware
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
- **Date/Time**: chrono 0.4 + chrono-tz 0.10
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
- [x] Support for multiple calendars
- [x] API token authentication
- [ ] Advanced filters (by category, location, etc)
- [ ] Export to different formats (PDF, Excel, etc)
- [ ] Web dashboard
- [ ] Docker support
- [ ] CI/CD pipeline
- [ ] Rate limiting
- [ ] Calendar event caching

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
