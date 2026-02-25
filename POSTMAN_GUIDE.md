# Postman Collection Guide

## Importing the Collection

1. Open Postman
2. Click **Import** button (top left)
3. Select **File** tab
4. Choose `postman_collection.json` from this directory
5. Click **Import**

## Collection Variables

The collection includes the following variables that you can customize:

| Variable | Default Value | Description |
|----------|---------------|-------------|
| `base_url` | `http://127.0.0.1:8080` | Base URL of your API server |
| `api_token` | `testtoken` | Your API authentication token |
| `calendar_name` | `cosm` | Default calendar name for testing specific calendar endpoints |
| `timezone` | `America/New_York` | Default timezone for timezone conversion tests |

### How to Edit Variables

1. Click on the collection name "Calendar Scraper API"
2. Go to the **Variables** tab
3. Update the **Current Value** for any variable
4. Click **Save**

## Authentication

The collection is pre-configured with Bearer Token authentication at the collection level. All requests (except Health Check) will automatically include:

```
Authorization: Bearer {{api_token}}
```

Make sure to set the `api_token` variable to match your `.env` file's `API_TOKEN` value.

## Available Endpoints

### 1. Health Check (No Auth)
- **GET** `/health`
- Tests if the server is running
- No authentication required

### 2. List Calendars
- **GET** `/api/v1/calendars`
- Returns all configured calendars from `calendars.toml`

### 3. Weekly Events
- **All Calendars:**
  - Without timezone: `GET /api/v1/events/weekly/all`
  - With timezone: `GET /api/v1/events/weekly/all?timezone={{timezone}}`

- **Specific Calendar:**
  - Without timezone: `GET /api/v1/calendars/{{calendar_name}}/events/weekly`
  - With timezone: `GET /api/v1/calendars/{{calendar_name}}/events/weekly?timezone={{timezone}}`

### 4. Today's Events
- **All Calendars:**
  - Without timezone: `GET /api/v1/events/today/all`
  - With timezone: `GET /api/v1/events/today/all?timezone={{timezone}}`

- **Specific Calendar:**
  - Without timezone: `GET /api/v1/calendars/{{calendar_name}}/events/today`
  - With timezone: `GET /api/v1/calendars/{{calendar_name}}/events/today?timezone={{timezone}}`

### 5. Today + Tomorrow Events
- **All Calendars:**
  - Without timezone: `GET /api/v1/events/today-and-tomorrow/all`
  - With timezone: `GET /api/v1/events/today-and-tomorrow/all?timezone={{timezone}}`

- **Specific Calendar:**
  - Without timezone: `GET /api/v1/calendars/{{calendar_name}}/events/today-and-tomorrow`
  - With timezone: `GET /api/v1/calendars/{{calendar_name}}/events/today-and-tomorrow?timezone={{timezone}}`

## Testing Workflow

### Quick Start
1. Start your server: `cargo run`
2. Test health endpoint first (no auth required)
3. List calendars to see what's available
4. Test specific calendar endpoints using calendar names from step 3

### Testing Different Timezones
1. Open any endpoint with timezone support
2. Click on the **Params** tab
3. Modify the `timezone` query parameter value
4. Common timezones to test:
   - `America/New_York` (EST/EDT)
   - `America/Los_Angeles` (PST/PDT)
   - `Europe/London` (GMT/BST)
   - `Asia/Tokyo` (JST)
   - `UTC`

### Testing Different Calendars
1. Use "List Calendars" endpoint to get all available calendar names
2. Update the `calendar_name` collection variable
3. Test any "Specific Calendar" endpoint

## Common IANA Timezone Names

- **Americas:**
  - `America/New_York` - Eastern Time
  - `America/Chicago` - Central Time
  - `America/Denver` - Mountain Time
  - `America/Los_Angeles` - Pacific Time
  - `America/Sao_Paulo` - Brazil

- **Europe:**
  - `Europe/London` - UK
  - `Europe/Paris` - Central European Time
  - `Europe/Berlin` - Germany
  - `Europe/Madrid` - Spain

- **Asia:**
  - `Asia/Tokyo` - Japan
  - `Asia/Shanghai` - China
  - `Asia/Dubai` - UAE
  - `Asia/Kolkata` - India

- **Other:**
  - `UTC` - Coordinated Universal Time
  - `Australia/Sydney` - Australia

## Response Examples

### Health Check Response
```json
{
  "status": "ok",
  "service": "calendar-scraper"
}
```

### List Calendars Response
```json
{
  "calendars": [
    { "name": "cosm" },
    { "name": "google" }
  ]
}
```

### Weekly Events Response (UTC)
```json
{
  "week": {
    "start": "2026-02-23T00:00:00Z",
    "end": "2026-03-01T23:59:59Z"
  },
  "calendars": [
    {
      "calendar_name": "cosm",
      "events": [
        {
          "id": "event-id",
          "title": "Team Meeting",
          "description": null,
          "start": "2026-02-25T14:00:00Z",
          "end": "2026-02-25T15:00:00Z",
          "location": "Office",
          "all_day": false,
          "calendar": "cosm"
        }
      ]
    }
  ]
}
```

### Events with Timezone Response
```json
{
  "week": {
    "start": "2026-02-23T00:00:00Z",
    "end": "2026-03-01T23:59:59Z"
  },
  "calendars": [
    {
      "calendar_name": "cosm",
      "events": [
        {
          "id": "event-id",
          "title": "Team Meeting",
          "description": null,
          "start": "2026-02-25T09:00:00-05:00",
          "end": "2026-02-25T10:00:00-05:00",
          "location": "Office",
          "all_day": false,
          "calendar": "cosm"
        }
      ]
    }
  ],
  "timezone": "America/New_York"
}
```

## Troubleshooting

### 401 Unauthorized
- Check that `api_token` variable matches your `API_TOKEN` in `.env`
- Verify the Authorization header is included (except for /health)

### 404 Calendar Not Found
- Run "List Calendars" endpoint to see available calendars
- Update `calendar_name` variable with a valid calendar name
- Check `calendars.toml` configuration

### 400 Invalid Timezone
- Ensure timezone parameter uses IANA timezone names
- Check for typos in timezone name
- Refer to the "Common IANA Timezone Names" section above

### Server Not Running
- Start the server: `cargo run`
- Check that `base_url` variable points to correct host/port
- Verify no other process is using port 8080

## Tips

1. **Use Environments**: Create different Postman environments for development, staging, and production
2. **Save Responses**: Use Postman's "Save Response" feature to compare responses over time
3. **Collection Runner**: Use Collection Runner to test all endpoints in sequence
4. **Tests**: Add test scripts to validate response structure and status codes
5. **Pre-request Scripts**: Use pre-request scripts to dynamically set variables

## Further Reading

- [Postman Documentation](https://learning.postman.com/docs/getting-started/introduction/)
- [IANA Time Zone Database](https://www.iana.org/time-zones)
- [Calendar Scraper README](./README.md)
