# Calendar Scraper - Project Specifications

## Objective
Develop a Rust application that collects calendar events from ICS links and exposes them via REST API in JSON format, initially focusing on current week events.

## Architecture

### Main Modules

1. **ICS Parser** (`src/ics/`)
   - Responsible for downloading and parsing ICS files
   - Extracts event information (title, date/time, description, location)
   - Handles different formats and encodings

2. **Event Filter** (`src/filters/`)
   - Filters events by period (initially current week)
   - Current week: from Monday 00:00 to Sunday 23:59
   - Future support for other periods (month, specific day)

3. **API REST** (`src/api/`)
   - Framework: Actix-web
   - RESTful endpoints for event queries
   - JSON serialization

4. **Models** (`src/models/`)
   - Data structures for events
   - DTOs for requests and responses

## Technical Requirements

### Test-Driven Development (TDD)
- All modules must have unit tests written BEFORE implementation
- Minimum test coverage: 80%
- Integration tests for API endpoints
- Parsing tests for different ICS formats

### Technology Stack
- **Language**: Rust (edition 2021)
- **Web Framework**: Actix-web
- **ICS Parser**: icalendar crate
- **HTTP Client**: reqwest
- **Serialization**: serde + serde_json
- **Date/Time**: chrono
- **Testing**: cargo test + mockito (for HTTP mocks)

### Endpoint Structure

#### v1.0
```
GET /api/v1/events/weekly?ics_url=<url>
```
Returns current week events from the provided ICS calendar.

**Response:**
```json
{
  "week": {
    "start": "2026-02-24T00:00:00Z",
    "end": "2026-03-02T23:59:59Z"
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

### Future Roadmap

#### Phase 2: Google Calendar Integration
- OAuth2 authentication
- Bidirectional synchronization
- Event caching
- Webhooks for real-time updates

#### Phase 3: Improvements
- Support for multiple calendars
- Advanced filters (by category, location, etc)
- Export to different formats
- Web dashboard

## Development Standards

### Language
- **Everything in English**: code, comments, commits, documentation
- All Rust code, inline comments, function names, variables, and tests must be in English
- Commit messages in English using conventional commits format

### Git Workflow
- Work in feature branches (never directly on main)
- Small and atomic commits
- Descriptive commit messages in English
- Format: `<type>: <description>`
  - types: feat, fix, test, refactor, docs, chore
- Author: felipe
- Merge to main via pull requests (for now, direct merge is acceptable)

### Code Style
- Follow Rust style guide
- Use `cargo fmt` before each commit
- Use `cargo clippy` for linting
- Inline documentation for public functions (in English)

### Error Handling
- Use `Result<T, E>` for operations that can fail
- Custom error types with thiserror
- Structured logging with tracing

### Performance
- Async I/O operations
- Optional cache for frequent ICS URLs
- Rate limiting for external requests

## Security
- ICS URL validation
- HTTP request timeouts
- ICS file size limits
- Input data sanitization

## Observability
- Structured logs (info, warn, error)
- Basic metrics (request count, latency)
- Health check endpoint

## Acceptance Criteria
1. Functional ICS parser with tests
2. Correct current week filter
3. REST API responding in JSON
4. Test coverage >= 80%
5. Complete README documentation
6. Zero clippy warnings
7. CI build passing (future)
