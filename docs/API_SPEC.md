# API Specification

Logical operations:
```text
POST /activity/sessions/batch
GET  /activity/sessions
GET  /activity/timeline
GET  /analytics/summary
GET  /analytics/weekly
GET  /devices
POST /devices
PATCH /devices/:id
POST /devices/:id/revoke
GET  /settings
PUT  /settings
GET  /rules
POST /rules
PATCH /rules/:id
DELETE /rules/:id
```

Implementation may use Supabase REST or controlled server functions. Use authenticated requests, versioned payloads, cursor pagination, and machine-readable error codes.

Errors include `AUTH_REQUIRED`, `DEVICE_REVOKED`, `SCHEMA_UNSUPPORTED`, `VALIDATION_FAILED`, `RATE_LIMITED`, `TEMPORARY_UNAVAILABLE`, `DUPLICATE`.
