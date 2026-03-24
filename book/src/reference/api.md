# REST API Reference

ContentForge exposes a REST API when running in server mode (`contentforge serve`). All endpoints are under the `/api/` prefix.

## Base URL

```
http://localhost:3000/api
```

## Authentication

The API currently does not require authentication (it is designed for local use). When running on `127.0.0.1`, only local processes can access it.

## Content Endpoints

### List Content

```
GET /api/content
```

**Query Parameters:**

| Parameter | Type   | Description                 |
|-----------|--------|-----------------------------|
| `status`  | string | Filter by content status    |
| `type`    | string | Filter by content type      |
| `project` | string | Filter by project           |
| `tag`     | string | Filter by tag               |
| `limit`   | number | Max results (default: 50)   |
| `offset`  | number | Pagination offset           |

**Response:**

```json
{
  "items": [
    {
      "id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
      "title": "Rust Error Handling",
      "body": "# Rust Error Handling\n\n...",
      "content_type": "article",
      "status": "drafting",
      "tags": ["rust", "error-handling"],
      "project": "blog",
      "created_at": "2026-03-19T10:00:00Z",
      "updated_at": "2026-03-19T14:30:00Z"
    }
  ],
  "total": 42
}
```

### Create Content

```
POST /api/content
```

**Request Body:**

```json
{
  "title": "Rust Error Handling",
  "body": "# Rust Error Handling\n\nError handling in Rust...",
  "content_type": "article",
  "tags": ["rust", "error-handling"],
  "project": "blog"
}
```

**Response:** `201 Created`

```json
{
  "id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "title": "Rust Error Handling",
  "status": "idea",
  "created_at": "2026-03-19T10:00:00Z"
}
```

### Get Content

```
GET /api/content/:id
```

**Response:**

```json
{
  "id": "a1b2c3d4-...",
  "title": "Rust Error Handling",
  "body": "...",
  "content_type": "article",
  "status": "drafting",
  "tags": ["rust"],
  "project": "blog",
  "adaptations": [
    {
      "platform": "twitter",
      "body": "Rust's error handling...",
      "thread_parts": ["Tweet 1...", "Tweet 2..."]
    }
  ],
  "media": [],
  "created_at": "2026-03-19T10:00:00Z",
  "updated_at": "2026-03-19T14:30:00Z"
}
```

### Update Content

```
PUT /api/content/:id
```

**Request Body:** (partial updates supported)

```json
{
  "title": "Updated Title",
  "body": "Updated body...",
  "tags": ["rust", "tutorial"],
  "status": "ready"
}
```

**Response:** `200 OK` with the updated content object.

### Delete Content

```
DELETE /api/content/:id
```

**Response:** `204 No Content`

---

## Adaptation Endpoints

### Create Adaptation

```
POST /api/content/:id/adapt
```

**Request Body:**

```json
{
  "platform": "twitter",
  "use_ai": true
}
```

**Response:** `201 Created`

```json
{
  "platform": "twitter",
  "body": "Rust's error handling is one of its killer features...",
  "thread_parts": [
    "1/ Rust's error handling is one of its killer features...",
    "2/ The Result<T, E> type forces you to handle errors..."
  ],
  "canonical_url": null
}
```

### Get Adaptations

```
GET /api/content/:id/adaptations
```

**Response:**

```json
{
  "adaptations": [
    {
      "platform": "twitter",
      "body": "...",
      "thread_parts": ["...", "..."]
    },
    {
      "platform": "devto",
      "body": "...",
      "title": "Rust Error Handling Guide"
    }
  ]
}
```

---

## Publishing Endpoints

### Publish

```
POST /api/content/:id/publish
```

**Request Body:**

```json
{
  "platform": "devto"
}
```

**Response:** `200 OK`

```json
{
  "publication": {
    "id": "pub-uuid-...",
    "platform": "dev_to",
    "url": "https://dev.to/username/rust-error-handling-abc",
    "platform_post_id": "1234567",
    "published_at": "2026-03-19T15:00:00Z"
  }
}
```

### Publish to All

```
POST /api/content/:id/publish-all
```

**Response:**

```json
{
  "results": [
    {
      "platform": "dev_to",
      "status": "success",
      "url": "https://dev.to/..."
    },
    {
      "platform": "twitter",
      "status": "success",
      "url": "https://x.com/i/status/..."
    },
    {
      "platform": "linkedin",
      "status": "error",
      "error": "Authentication failed"
    }
  ]
}
```

### List Publications

```
GET /api/content/:id/publications
```

---

## Schedule Endpoints

### Create Schedule Entry

```
POST /api/schedule
```

**Request Body:**

```json
{
  "content_id": "a1b2c3d4-...",
  "platform": "twitter",
  "scheduled_at": "2026-03-20T09:00:00Z"
}
```

**Response:** `201 Created`

### List Schedule

```
GET /api/schedule
```

**Query Parameters:**

| Parameter | Type   | Description                    |
|-----------|--------|--------------------------------|
| `status`  | string | Filter (pending, published, failed) |
| `platform`| string | Filter by platform             |

### Cancel Schedule Entry

```
DELETE /api/schedule/:id
```

**Response:** `204 No Content`

---

## Platform Endpoints

### List Platforms

```
GET /api/platforms
```

**Response:**

```json
{
  "platforms": [
    {
      "platform": "dev_to",
      "display_name": "username",
      "enabled": true,
      "healthy": true
    },
    {
      "platform": "twitter",
      "display_name": "@handle",
      "enabled": true,
      "healthy": true
    }
  ]
}
```

### Platform Health Check

```
GET /api/platforms/health
```

---

## Analytics Endpoints

### Get Analytics for Content

```
GET /api/content/:id/analytics
```

**Response:**

```json
{
  "analytics": [
    {
      "platform": "dev_to",
      "url": "https://dev.to/...",
      "views": 1250,
      "likes": 45,
      "comments": 12,
      "captured_at": "2026-03-19T18:00:00Z"
    }
  ]
}
```

### Get Analytics Summary

```
GET /api/analytics/summary
```

---

## WebSocket

### Real-time Updates

```
WS /api/ws
```

Connect to receive real-time events:

```json
{"type": "publish_success", "content_id": "...", "platform": "twitter", "url": "..."}
{"type": "publish_failed", "content_id": "...", "platform": "linkedin", "error": "..."}
{"type": "schedule_triggered", "schedule_id": "...", "content_id": "..."}
{"type": "analytics_updated", "content_id": "...", "platform": "dev_to"}
```

---

## Error Responses

All errors follow a consistent format:

```json
{
  "error": {
    "code": "content_not_found",
    "message": "Content not found: a1b2c3d4-..."
  }
}
```

Common error codes:

| Code                    | HTTP Status | Description                   |
|-------------------------|-------------|-------------------------------|
| `content_not_found`     | 404         | Content ID does not exist     |
| `platform_not_configured`| 400        | Platform adapter not set up   |
| `publish_failed`        | 502         | Platform API call failed      |
| `rate_limited`          | 429         | Platform rate limit reached   |
| `auth_failed`           | 401         | Platform credentials invalid  |
| `content_too_long`      | 400         | Exceeds platform char limit   |
| `validation_error`      | 422         | Invalid request body          |
