# Civic Sentinel API Reference

Complete API documentation for Civic Sentinel backend endpoints.

**Base URL:** `http://localhost:3000` (development) or your deployment URL  
**API Version:** v1  
**Last Updated:** July 3, 2026

---

## Table of Contents

1. [Authentication](#authentication)
2. [Issues](#issues)
3. [Comments](#comments)
4. [Status History](#status-history)
5. [Analytics](#analytics)
6. [Uploads](#uploads)
7. [Health & Info](#health--info)
8. [Error Responses](#error-responses)
9. [Rate Limiting](#rate-limiting)

---

## Authentication

### Register

Create a new user account.

**Endpoint:** `POST /api/v1/auth/register`  
**Authentication:** None

**Request Body:**
```json
{
  "email": "user@example.com",
  "password": "SecurePassword123!",
  "name": "John Doe"
}
```

**Response:** `201 Created`
```json
{
  "success": true,
  "data": {
    "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "user": {
      "id": "123e4567-e89b-12d3-a456-426614174000",
      "email": "user@example.com",
      "name": "John Doe",
      "role": "reporter",
      "phone_verified": false
    }
  }
}
```

**Errors:**
- `400 Bad Request` — Invalid email format or password too weak
- `409 Conflict` — Email already registered

---

### Login

Authenticate and receive JWT tokens.

**Endpoint:** `POST /api/v1/auth/login`  
**Authentication:** None

**Request Body:**
```json
{
  "email": "user@example.com",
  "password": "SecurePassword123!"
}
```

**Response:** `200 OK`
```json
{
  "success": true,
  "data": {
    "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "user": {
      "id": "123e4567-e89b-12d3-a456-426614174000",
      "email": "user@example.com",
      "name": "John Doe",
      "role": "reporter",
      "phone_verified": true
    }
  }
}
```

**Errors:**
- `400 Bad Request` — Missing email or password
- `401 Unauthorized` — Invalid credentials

---

### Refresh Token

Get a new access token using refresh token.

**Endpoint:** `POST /api/v1/auth/refresh`  
**Authentication:** None (refresh token in body)

**Request Body:**
```json
{
  "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

**Response:** `200 OK`
```json
{
  "success": true,
  "data": {
    "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "user": {
      "id": "123e4567-e89b-12d3-a456-426614174000",
      "email": "user@example.com",
      "name": "John Doe",
      "role": "reporter"
    }
  }
}
```

**Errors:**
- `401 Unauthorized` — Invalid or expired refresh token

**Notes:**
- Old refresh token is revoked automatically
- New refresh token must be stored for future refreshes

---

### Logout

Revoke the current refresh token.

**Endpoint:** `POST /api/v1/auth/logout`  
**Authentication:** None (refresh token in body)

**Request Body:**
```json
{
  "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

**Response:** `200 OK`
```json
{
  "success": true,
  "message": "Logged out successfully"
}
```

---

### Request Phone OTP

Request a one-time password for phone verification.

**Endpoint:** `POST /api/v1/auth/otp/request`  
**Authentication:** None

**Request Body:**
```json
{
  "phone": "+66812345678"
}
```

**Response:** `200 OK`
```json
{
  "success": true,
  "data": {
    "challenge_id": "223e4567-e89b-12d3-a456-426614174000",
    "expires_at": "2026-07-03T10:05:00Z",
    "message": "OTP sent to +66812345678"
  }
}
```

**Notes:**
- OTP is sent via SMS (implementation pending — currently returns mock response)
- OTP expires in 5 minutes
- Maximum 5 verification attempts per challenge

---

### Verify Phone OTP

Verify the OTP code and receive a verification token.

**Endpoint:** `POST /api/v1/auth/otp/verify`  
**Authentication:** None

**Request Body:**
```json
{
  "phone": "+66812345678",
  "code": "123456"
}
```

**Response:** `200 OK`
```json
{
  "success": true,
  "data": {
    "verification_token": "abc123def456...",
    "expires_at": "2026-07-10T10:00:00Z"
  }
}
```

**Errors:**
- `400 Bad Request` — Invalid code format
- `401 Unauthorized` — Incorrect code
- `429 Too Many Requests` — Max attempts exceeded

**Notes:**
- Verification token is valid for 7 days
- Use this token in `otp_token` field when creating issues

---

## Issues

### List Issues

Retrieve a paginated list of issues with optional filters.

**Endpoint:** `GET /api/v1/issues`  
**Authentication:** None

**Query Parameters:**
| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `page` | integer | 1 | Page number (min: 1) |
| `limit` | integer | 20 | Items per page (min: 1, max: 100) |
| `status` | string | — | Filter by status: `reported`, `underreview`, `inprogress`, `resolved`, `closed`, `escalated` |
| `category` | string | — | Filter by category: `infrastructure`, `safety`, `environment`, `sanitation`, `transportation`, `publicutility`, `other` |
| `priority` | string | — | Filter by priority: `low`, `medium`, `high`, `critical` |
| `verification_state` | string | — | Filter by state: `trusted`, `needs_review`, `suspicious` |
| `sort` | string | `created_at` | Sort by: `created_at`, `updated_at`, `priority`, `triage` |

**Example Request:**
```
GET /api/v1/issues?page=1&limit=20&status=reported&sort=triage
```

**Response:** `200 OK`
```json
{
  "success": true,
  "data": [
    {
      "id": "123e4567-e89b-12d3-a456-426614174000",
      "title": "Broken streetlight on Main St",
      "description": "The streetlight has been out for 3 days.",
      "category": "infrastructure",
      "priority": "medium",
      "status": "reported",
      "verification_score": 65,
      "verification_state": "trusted",
      "duplicate_of": null,
      "corroboration_count": 2,
      "triage_score": 75,
      "location": {
        "latitude": 13.7563,
        "longitude": 100.5018,
        "address": "123 Main St, Bangkok"
      },
      "reporter_id": "223e4567-e89b-12d3-a456-426614174000",
      "assigned_to": null,
      "media_urls": ["https://example.com/uploads/image1.jpg"],
      "tags": ["streetlight", "urgent"],
      "created_at": "2026-07-01T10:00:00Z",
      "updated_at": "2026-07-03T09:00:00Z",
      "resolved_at": null
    }
  ],
  "meta": {
    "total": 42,
    "page": 1,
    "per_page": 20,
    "total_pages": 3
  }
}
```

---

### Create Issue

Report a new civic issue.

**Endpoint:** `POST /api/v1/issues`  
**Authentication:** Optional (recommended for verified reports)

**Request Body:**
```json
{
  "title": "Pothole on Highway 101",
  "description": "Large pothole causing traffic hazard near exit 23.",
  "category": "infrastructure",
  "location": {
    "latitude": 13.7563,
    "longitude": 100.5018,
    "address": "Highway 101, Exit 23"
  },
  "media_urls": ["https://example.com/uploads/pothole.jpg"],
  "tags": ["pothole", "highway"],
  "otp_phone": "+66812345678",
  "otp_token": "abc123def456..."
}
```

**Fields:**
| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `title` | string | Yes | Issue title (3-200 chars) |
| `description` | string | Yes | Detailed description (10-5000 chars) |
| `category` | string | No | Category (auto-detected if omitted) |
| `location` | object | Yes | Geographic location |
| `location.latitude` | number | Yes | Latitude (-90 to 90) |
| `location.longitude` | number | Yes | Longitude (-180 to 180) |
| `location.address` | string | No | Human-readable address |
| `media_urls` | array | No | URLs from upload endpoint |
| `tags` | array | No | Optional tags |
| `otp_phone` | string | No | Phone number if using OTP |
| `otp_token` | string | No | Verification token from OTP flow |

**Response:** `201 Created`
```json
{
  "success": true,
  "data": {
    "id": "323e4567-e89b-12d3-a456-426614174000",
    "title": "Pothole on Highway 101",
    "description": "Large pothole causing traffic hazard near exit 23.",
    "category": "infrastructure",
    "priority": "high",
    "status": "reported",
    "verification_score": 75,
    "verification_state": "trusted",
    "duplicate_of": null,
    "corroboration_count": 0,
    "triage_score": 60,
    "location": {
      "latitude": 13.7563,
      "longitude": 100.5018,
      "address": "Highway 101, Exit 23"
    },
    "reporter_id": "223e4567-e89b-12d3-a456-426614174000",
    "assigned_to": null,
    "media_urls": ["https://example.com/uploads/pothole.jpg"],
    "tags": ["pothole", "highway"],
    "created_at": "2026-07-03T10:00:00Z",
    "updated_at": "2026-07-03T10:00:00Z",
    "resolved_at": null
  }
}
```

**Notes:**
- If authenticated, `reporter_id` is set to current user
- If `otp_phone` and `otp_token` provided, verification score is boosted
- Category and priority are auto-detected if not provided
- Duplicate detection runs automatically (within 500m radius)

---

### Get Issue

Retrieve details of a single issue.

**Endpoint:** `GET /api/v1/issues/:id`  
**Authentication:** None

**Path Parameters:**
- `id` (UUID) — Issue ID

**Response:** `200 OK`
```json
{
  "success": true,
  "data": {
    "id": "123e4567-e89b-12d3-a456-426614174000",
    "title": "Broken streetlight on Main St",
    "description": "The streetlight has been out for 3 days.",
    "category": "infrastructure",
    "priority": "medium",
    "status": "reported",
    "verification_score": 65,
    "verification_state": "trusted",
    "duplicate_of": null,
    "corroboration_count": 2,
    "triage_score": 75,
    "location": {
      "latitude": 13.7563,
      "longitude": 100.5018,
      "address": "123 Main St, Bangkok"
    },
    "reporter_id": "223e4567-e89b-12d3-a456-426614174000",
    "assigned_to": null,
    "media_urls": ["https://example.com/uploads/image1.jpg"],
    "tags": ["streetlight", "urgent"],
    "created_at": "2026-07-01T10:00:00Z",
    "updated_at": "2026-07-03T09:00:00Z",
    "resolved_at": null
  }
}
```

**Errors:**
- `404 Not Found` — Issue does not exist or was deleted

---

### Update Issue

Update an existing issue (status, priority, assignment).

**Endpoint:** `PATCH /api/v1/issues/:id`  
**Authentication:** Required (JWT)  
**Required Role:** Responder or Admin

**Path Parameters:**
- `id` (UUID) — Issue ID

**Request Body:**
```json
{
  "status": "inprogress",
  "priority": "high",
  "assigned_to": "423e4567-e89b-12d3-a456-426614174000",
  "reason": "Escalating to high priority due to safety concern"
}
```

**Fields (all optional):**
| Field | Type | Description |
|-------|------|-------------|
| `title` | string | Update title |
| `description` | string | Update description |
| `status` | string | New status (see transitions below) |
| `priority` | string | `low`, `medium`, `high`, `critical` |
| `assigned_to` | UUID or null | Assign to user (null to unassign) |
| `reason` | string | Explanation for status change (required for backward transitions) |

**Status Transitions:**
```
reported → underreview → inprogress → resolved → closed
         ↘                ↓            ↓
           escalated ←────┴────────────┘
```

**Response:** `200 OK`
```json
{
  "success": true,
  "data": {
    "id": "123e4567-e89b-12d3-a456-426614174000",
    "status": "inprogress",
    "priority": "high",
    "assigned_to": "423e4567-e89b-12d3-a456-426614174000",
    "updated_at": "2026-07-03T10:30:00Z"
  }
}
```

**Errors:**
- `401 Unauthorized` — Not authenticated
- `403 Forbidden` — Insufficient permissions
- `404 Not Found` — Issue does not exist
- `400 Bad Request` — Invalid status transition or missing reason

---

### Delete Issue

Soft delete an issue (sets `deleted_at` timestamp).

**Endpoint:** `DELETE /api/v1/issues/:id`  
**Authentication:** Required (JWT)  
**Required Role:** Admin only

**Path Parameters:**
- `id` (UUID) — Issue ID

**Response:** `200 OK`
```json
{
  "success": true,
  "message": "Issue deleted successfully"
}
```

**Errors:**
- `401 Unauthorized` — Not authenticated
- `403 Forbidden` — Not an admin
- `404 Not Found` — Issue does not exist

---

## Comments

### Get Comments

Retrieve all comments for an issue.

**Endpoint:** `GET /api/v1/issues/:id/comments`  
**Authentication:** None

**Path Parameters:**
- `id` (UUID) — Issue ID

**Response:** `200 OK`
```json
{
  "success": true,
  "data": [
    {
      "id": "523e4567-e89b-12d3-a456-426614174000",
      "issue_id": "123e4567-e89b-12d3-a456-426614174000",
      "author_id": "223e4567-e89b-12d3-a456-426614174000",
      "content": "I also noticed this issue yesterday.",
      "is_internal": false,
      "created_at": "2026-07-02T14:30:00Z"
    }
  ]
}
```

---

### Create Comment

Add a comment to an issue.

**Endpoint:** `POST /api/v1/issues/:id/comments`  
**Authentication:** Required (JWT)

**Path Parameters:**
- `id` (UUID) — Issue ID

**Request Body:**
```json
{
  "content": "I reported this to the city council.",
  "is_internal": false
}
```

**Fields:**
| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `content` | string | Yes | Comment text (1-5000 chars) |
| `is_internal` | boolean | No | Admin-only note (default: false) |

**Response:** `201 Created`
```json
{
  "success": true,
  "data": {
    "id": "623e4567-e89b-12d3-a456-426614174000",
    "issue_id": "123e4567-e89b-12d3-a456-426614174000",
    "author_id": "223e4567-e89b-12d3-a456-426614174000",
    "content": "I reported this to the city council.",
    "is_internal": false,
    "created_at": "2026-07-03T10:45:00Z"
  }
}
```

---

## Status History

### Get Status History

Retrieve the status change history for an issue.

**Endpoint:** `GET /api/v1/issues/:id/history`  
**Authentication:** None

**Path Parameters:**
- `id` (UUID) — Issue ID

**Response:** `200 OK`
```json
{
  "success": true,
  "data": [
    {
      "id": "723e4567-e89b-12d3-a456-426614174000",
      "issue_id": "123e4567-e89b-12d3-a456-426614174000",
      "old_status": "reported",
      "new_status": "underreview",
      "changed_by": "423e4567-e89b-12d3-a456-426614174000",
      "reason": "Initial review",
      "created_at": "2026-07-02T09:00:00Z"
    },
    {
      "id": "823e4567-e89b-12d3-a456-426614174000",
      "issue_id": "123e4567-e89b-12d3-a456-426614174000",
      "old_status": "underreview",
      "new_status": "inprogress",
      "changed_by": "423e4567-e89b-12d3-a456-426614174000",
      "reason": "Assigned to repair team",
      "created_at": "2026-07-03T10:00:00Z"
    }
  ]
}
```

---

## Analytics

### Get Analytics

Retrieve real-time analytics summary.

**Endpoint:** `GET /api/v1/analytics`  
**Authentication:** None

**Response:** `200 OK`
```json
{
  "success": true,
  "data": {
    "total_issues": 127,
    "open_issues": 89,
    "resolved_issues": 35,
    "avg_resolution_hours": 48.5,
    "issues_by_category": [
      { "category": "infrastructure", "count": 45 },
      { "category": "safety", "count": 28 },
      { "category": "environment", "count": 22 },
      { "category": "sanitation", "count": 18 },
      { "category": "transportation", "count": 10 },
      { "category": "publicutility", "count": 3 },
      { "category": "other", "count": 1 }
    ],
    "issues_by_priority": [
      { "priority": "critical", "count": 5 },
      { "priority": "high", "count": 23 },
      { "priority": "medium", "count": 67 },
      { "priority": "low", "count": 32 }
    ]
  }
}
```

---

## Uploads

### Upload File

Upload an image for issue attachment.

**Endpoint:** `POST /api/v1/uploads`  
**Authentication:** None  
**Content-Type:** `multipart/form-data`

**Request:**
```
POST /api/v1/uploads
Content-Type: multipart/form-data

--boundary
Content-Disposition: form-data; name="file"; filename="photo.jpg"
Content-Type: image/jpeg

[binary data]
--boundary--
```

**Response:** `201 Created`
```json
{
  "success": true,
  "data": {
    "url": "http://localhost:3000/uploads/1688371200_abc123.jpg",
    "filename": "1688371200_abc123.jpg",
    "size": 245678
  }
}
```

**Notes:**
- Currently accepts all file types (validation pending)
- Files stored in `UPLOAD_DIR` (default: `uploads/`)
- No size limit enforced (pending implementation)
- Filename sanitization required (security issue)

---

## Health & Info

### Health Check

Check if API is running.

**Endpoint:** `GET /health`  
**Authentication:** None

**Response:** `200 OK`
```json
{
  "status": "healthy",
  "version": "0.1.0",
  "timestamp": "2026-07-03T10:00:00Z"
}
```

---

### API Info

Get API version and endpoint list.

**Endpoint:** `GET /api/v1/info`  
**Authentication:** None

**Response:** `200 OK`
```json
{
  "name": "Civic Sentinel API",
  "version": "0.1.0",
  "description": "Civic issue reporting and tracking platform",
  "endpoints": [
    "GET /health — Health check",
    "GET /api/v1/info — API information",
    "POST /api/v1/auth/register — Create account",
    "..."
  ]
}
```

---

## Error Responses

All error responses follow this format:

```json
{
  "success": false,
  "error": "Error message",
  "details": "Additional context (optional)"
}
```

### HTTP Status Codes

| Code | Meaning | Usage |
|------|---------|-------|
| 200 | OK | Successful GET/PATCH/DELETE request |
| 201 | Created | Successful POST request |
| 400 | Bad Request | Invalid input (validation failed) |
| 401 | Unauthorized | Missing or invalid JWT token |
| 403 | Forbidden | Authenticated but insufficient permissions |
| 404 | Not Found | Resource does not exist |
| 409 | Conflict | Resource already exists (e.g., duplicate email) |
| 429 | Too Many Requests | Rate limit exceeded |
| 500 | Internal Server Error | Server-side error |

---

## Rate Limiting

**Default Limits:**
- **100 requests per 60 seconds** per IP address
- Applies to all endpoints globally

**Response Headers:**
```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1688371260
```

**Rate Limit Exceeded:**
```
HTTP/1.1 429 Too Many Requests

{
  "success": false,
  "error": "Rate limit exceeded",
  "details": "Try again in 30 seconds"
}
```

**Notes:**
- Rate limits are tracked per client IP (from `X-Forwarded-For` or `X-Real-IP` headers)
- Token bucket algorithm with automatic cleanup every 5 minutes
- Configurable via `RATE_LIMIT_REQUESTS` and `RATE_LIMIT_WINDOW_SECONDS` environment variables

---

## Authentication Headers

**For protected endpoints, include JWT token:**

```
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

**Token expiration:**
- Access token: 24 hours (configurable via `JWT_EXPIRATION_HOURS`)
- Refresh token: 7 days (configurable via `REFRESH_TOKEN_EXPIRATION_DAYS`)

**Refresh flow:**
1. Access token expires → API returns `401 Unauthorized`
2. Frontend automatically calls `/api/v1/auth/refresh` with refresh token
3. Receives new access + refresh tokens
4. Retries original request with new access token

---

## Examples

### cURL

**Register:**
```bash
curl -X POST http://localhost:3000/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "user@example.com",
    "password": "SecurePassword123!",
    "name": "John Doe"
  }'
```

**Create Issue (with auth):**
```bash
curl -X POST http://localhost:3000/api/v1/issues \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_ACCESS_TOKEN" \
  -d '{
    "title": "Broken sidewalk",
    "description": "Sidewalk is cracked and dangerous.",
    "category": "infrastructure",
    "location": {
      "latitude": 13.7563,
      "longitude": 100.5018,
      "address": "123 Main St"
    }
  }'
```

### JavaScript (Axios)

```javascript
import axios from 'axios';

const api = axios.create({
  baseURL: 'http://localhost:3000/api/v1',
  headers: { 'Content-Type': 'application/json' }
});

// Login
const { data } = await api.post('/auth/login', {
  email: 'user@example.com',
  password: 'SecurePassword123!'
});

// Store tokens
localStorage.setItem('access_token', data.data.access_token);
localStorage.setItem('refresh_token', data.data.refresh_token);

// Authenticated request
const issues = await api.get('/issues', {
  headers: {
    Authorization: `Bearer ${localStorage.getItem('access_token')}`
  }
});
```

---

**API Version:** v1  
**Last Updated:** July 3, 2026  
**Maintained by:** [@FirstPrinciples-Sun](https://github.com/FirstPrinciples-Sun)
