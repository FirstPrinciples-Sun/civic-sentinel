# Civic Sentinel Architecture

This document describes the technical architecture, design decisions, and system components of Civic Sentinel.

**Last Updated:** July 3, 2026

---

## Table of Contents

1. [System Overview](#system-overview)
2. [Technology Stack](#technology-stack)
3. [Backend Architecture](#backend-architecture)
4. [Frontend Architecture](#frontend-architecture)
5. [Database Design](#database-design)
6. [Security Architecture](#security-architecture)
7. [API Design](#api-design)
8. [Deployment Architecture](#deployment-architecture)
9. [Design Decisions](#design-decisions)
10. [Future Enhancements](#future-enhancements)

---

## System Overview

Civic Sentinel is a full-stack web application that enables communities to report, track, and resolve civic issues such as infrastructure problems, safety concerns, and utility outages.

### Core Features

- **Issue Reporting:** Citizens report problems with photos, location, and description
- **Issue Tracking:** Status workflow from Reported → Under Review → In Progress → Resolved
- **Analytics Dashboard:** Real-time statistics and trends
- **Authentication:** JWT-based auth with phone OTP verification
- **Triage System:** Automatic priority scoring based on verification, corroboration, and age
- **Duplicate Detection:** Prevents redundant reports using geographic proximity

### User Roles

1. **Reporter** (default) — Can create issues and comments
2. **Responder** — Can update issue status and assign
3. **Admin** — Full access including user management
4. **Viewer** — Read-only access

---

## Technology Stack

### Backend
| Component | Technology | Version | Purpose |
|-----------|-----------|---------|---------|
| Language | Rust | 1.75+ | Type safety, performance, memory safety |
| Web Framework | Axum | 0.7 | Async HTTP server with minimal overhead |
| Database | libSQL (SQLite) | 0.3 | Embedded database, Turso cloud-ready |
| Authentication | JWT + Argon2 | jsonwebtoken 9.2, argon2 0.5 | Secure token-based auth |
| Validation | validator | 0.16 | Request payload validation |
| Serialization | serde | 1.0 | JSON serialization/deserialization |

### Frontend
| Component | Technology | Version | Purpose |
|-----------|-----------|---------|---------|
| Language | TypeScript | 5.2+ | Type safety for JavaScript |
| UI Framework | React | 18.2 | Component-based UI |
| Build Tool | Vite | 5.0 | Fast dev server and bundling |
| Styling | Tailwind CSS | 3.4 | Utility-first CSS |
| State Management | Zustand | 4.4 | Lightweight global state |
| Forms | React Hook Form | 7.49 | Form validation and handling |
| HTTP Client | Axios | 1.6 | API requests with interceptors |
| Maps | Leaflet + React-Leaflet | 1.9 / 4.2 | Interactive maps |
| Charts | Recharts | 2.10 | Analytics visualizations |

### Infrastructure
| Component | Technology | Purpose |
|-----------|-----------|---------|
| Container | Docker + Docker Compose | Single-command deployment |
| CI/CD | GitHub Actions | Automated testing and builds |
| Database (Cloud) | Turso | Edge-replicated libSQL |

---

## Backend Architecture

### Project Structure

```
apps/api/
├── src/
│   ├── main.rs              # Application entry point
│   ├── lib.rs               # Public library interface
│   ├── config.rs            # Configuration management
│   ├── error.rs             # Error types and handling
│   ├── db/
│   │   └── mod.rs           # Database abstraction layer
│   ├── middleware/
│   │   ├── auth.rs          # JWT authentication middleware
│   │   ├── rate_limit.rs    # Rate limiting middleware
│   │   └── security_headers.rs # Security headers middleware
│   ├── models/
│   │   ├── mod.rs           # Domain models (Issue, User, etc.)
│   │   └── user.rs          # User model and auth types
│   ├── routes/
│   │   ├── auth.rs          # Authentication endpoints
│   │   ├── issues.rs        # Issue CRUD endpoints
│   │   ├── comments.rs      # Comment endpoints
│   │   ├── status_history.rs # Status history endpoints
│   │   ├── analytics.rs     # Analytics endpoints
│   │   └── uploads.rs       # File upload endpoints
│   └── services/
│       ├── notification.rs  # Notification service (planned)
│       ├── priority_engine.rs # Priority calculation logic
│       └── rule_based_analyzer.rs # Text analysis rules
└── Cargo.toml
```

### Layered Architecture

```
┌─────────────────────────────────────────┐
│         HTTP Layer (Axum)               │
│  - Routing                              │
│  - Request parsing                      │
│  - Response serialization               │
└────────────────┬────────────────────────┘
                 │
┌────────────────▼────────────────────────┐
│       Middleware Layer                  │
│  - Security headers                     │
│  - Rate limiting (shared instance)      │
│  - CORS                                 │
│  - Authentication (JWT validation)      │
└────────────────┬────────────────────────┘
                 │
┌────────────────▼────────────────────────┐
│        Route Handlers                   │
│  - Request validation                   │
│  - Business logic coordination          │
│  - Authorization checks                 │
│  - Error handling                       │
└────────────────┬────────────────────────┘
                 │
┌────────────────▼────────────────────────┐
│       Service Layer                     │
│  - Priority calculation                 │
│  - Duplicate detection                  │
│  - Notification dispatch (planned)      │
│  - Text analysis                        │
└────────────────┬────────────────────────┘
                 │
┌────────────────▼────────────────────────┐
│      Database Layer (db::Database)      │
│  - Query execution                      │
│  - Row mapping                          │
│  - Transaction management               │
└────────────────┬────────────────────────┘
                 │
┌────────────────▼────────────────────────┐
│      libSQL (SQLite/Turso)              │
│  - Data persistence                     │
│  - Full-text search (FTS5)              │
│  - Migrations                           │
└─────────────────────────────────────────┘
```

### Request Flow Example (Create Issue)

1. **HTTP Request:** `POST /api/v1/issues`
2. **Middleware Chain:**
   - Security headers added
   - Rate limit checked (from shared AppState)
   - CORS validated
3. **Route Handler:** `routes::issues::create_issue`
   - Parse and validate `CreateIssueRequest`
   - Optional: Verify OTP token if provided
4. **Service Layer:**
   - Analyze text for category/priority (`rule_based_analyzer`)
   - Check for duplicates within 500m radius (`list_duplicate_candidates`)
   - Calculate verification score based on reporter trust signals
5. **Database Layer:**
   - Insert issue into `issues` table
   - If duplicate found, update `duplicate_of` and increment corroboration count
6. **Response:** `201 Created` with issue JSON

---

## Frontend Architecture

### Project Structure

```
apps/web/
├── src/
│   ├── main.tsx             # App entry point
│   ├── App.tsx              # Root component with router
│   ├── index.css            # Global styles (Tailwind)
│   ├── components/
│   │   ├── Layout.tsx       # App shell (nav, footer)
│   │   └── AppErrorBoundary.tsx # Error boundary
│   ├── context/
│   │   ├── AuthContext.tsx  # Authentication state
│   │   └── LanguageContext.tsx # i18n state
│   ├── hooks/
│   │   ├── useAnalytics.ts  # Analytics data fetching
│   │   └── useIssues.ts     # Issues data fetching
│   ├── pages/
│   │   ├── HomePage.tsx     # Landing page
│   │   ├── ReportPage.tsx   # Issue creation form
│   │   ├── MapPage.tsx      # Interactive map view
│   │   ├── DashboardPage.tsx # Analytics dashboard
│   │   ├── IssueDetailPage.tsx # Single issue view
│   │   ├── AdminPage.tsx    # Admin panel
│   │   ├── LoginPage.tsx    # Login form
│   │   └── RegisterPage.tsx # Registration form
│   ├── services/
│   │   └── api.ts           # Axios instance + interceptors
│   └── i18n/
│       └── translations.ts  # Translation strings
└── package.json
```

### Component Hierarchy

```
<App>
├── <AuthContext.Provider>
│   ├── <LanguageContext.Provider>
│   │   ├── <AppErrorBoundary>
│   │   │   ├── <Layout>
│   │   │   │   ├── <Router>
│   │   │   │   │   ├── <HomePage />
│   │   │   │   │   ├── <ReportPage />
│   │   │   │   │   ├── <MapPage />
│   │   │   │   │   ├── <DashboardPage />
│   │   │   │   │   ├── <IssueDetailPage />
│   │   │   │   │   ├── <AdminPage />        # Protected
│   │   │   │   │   ├── <LoginPage />
│   │   │   │   │   └── <RegisterPage />
```

### State Management Strategy

**Global State (AuthContext):**
- User authentication status
- JWT tokens (access + refresh)
- Current user profile
- Auth actions (login, logout, refresh)

**Server State (React Query - planned):**
- Currently using custom hooks with useEffect
- Future: Migrate to @tanstack/react-query for caching

**Local State (useState):**
- Form inputs
- UI toggles (modals, dropdowns)
- Transient data

**Form State (React Hook Form):**
- Form validation
- Submission handling
- Error display

### API Integration

**Axios Instance** (`services/api.ts`):
```typescript
// Request Interceptor
- Attaches JWT token from localStorage to Authorization header

// Response Interceptor
- On 401: Attempt token refresh using refresh_token
- On success: Retry original request with new token
- On failure: Redirect to /login and clear tokens
```

---

## Database Design

### Entity-Relationship Diagram

```
┌──────────────┐
│    users     │
├──────────────┤
│ id (PK)      │───┐
│ email        │   │
│ password_hash│   │
│ name         │   │
│ phone        │   │
│ role         │   │
│ phone_verified│  │
│ created_at   │   │
└──────────────┘   │
                   │
         ┌─────────┼────────────────────────┐
         │         │                        │
         │         │                        │
┌────────▼─────────▼─┐       ┌──────────────▼────────┐
│      issues        │       │  refresh_tokens       │
├────────────────────┤       ├───────────────────────┤
│ id (PK)            │───┐   │ id (PK)               │
│ title              │   │   │ user_id (FK)          │
│ description        │   │   │ token_hash            │
│ category           │   │   │ expires_at            │
│ priority           │   │   │ revoked_at            │
│ status             │   │   └───────────────────────┘
│ latitude           │   │
│ longitude          │   │
│ address            │   │
│ reporter_id (FK)   │   │   ┌───────────────────────┐
│ assigned_to (FK)   │   ├──▶│  issue_comments       │
│ media_urls         │   │   ├───────────────────────┤
│ verification_score │   │   │ id (PK)               │
│ verification_state │   │   │ issue_id (FK)         │
│ duplicate_of (FK)  │───┘   │ author_id (FK)        │
│ corroboration_count│       │ content               │
│ triage_score       │       │ is_internal           │
│ created_at         │       │ created_at            │
│ resolved_at        │       └───────────────────────┘
│ deleted_at         │
└────────────────────┘       ┌───────────────────────┐
         │                   │ issue_status_history  │
         └──────────────────▶├───────────────────────┤
                             │ id (PK)               │
                             │ issue_id (FK)         │
                             │ old_status            │
                             │ new_status            │
                             │ changed_by (FK)       │
                             │ reason                │
                             │ created_at            │
                             └───────────────────────┘
```

### Key Tables

**users**
- Primary key: UUID
- Email unique index
- Phone verification fields
- Role-based access control

**issues**
- Primary key: UUID
- Soft delete with `deleted_at`
- Geographic data (latitude, longitude)
- Trust scoring fields (verification_score, verification_state)
- Duplicate tracking (duplicate_of, corroboration_count)
- Triage scoring for prioritization

**issue_comments**
- One-to-many with issues
- Internal flag for admin notes

**issue_status_history**
- Audit trail for status changes
- Captures who changed and why

**refresh_tokens**
- JWT refresh token management
- Hash stored (SHA-256)
- Expiration and revocation tracking

**phone_otp_challenges**
- OTP verification flow
- 6-digit code hashed (SHA-256)
- Attempt tracking (max 5)
- 5-minute expiration

**phone_verification_tokens**
- Long-lived token after OTP success
- Links phone to user account

### Indexes

**Performance Indexes:**
```sql
CREATE INDEX idx_issues_status ON issues(status);
CREATE INDEX idx_issues_category ON issues(category);
CREATE INDEX idx_issues_location ON issues(latitude, longitude);
CREATE INDEX idx_issues_created ON issues(created_at);
CREATE INDEX idx_issues_verification_state ON issues(verification_state);
CREATE INDEX idx_issues_duplicate_of ON issues(duplicate_of);
CREATE INDEX idx_issues_triage ON issues(triage_score);
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_phone_otp_phone ON phone_otp_challenges(phone);
```

**Full-Text Search:**
```sql
CREATE VIRTUAL TABLE issues_fts USING fts5(
    title, description,
    content='issues',
    content_rowid='rowid'
);
```

---

## Security Architecture

### Defense in Depth

```
┌─────────────────────────────────────────────────────┐
│  Layer 1: Network (HTTPS, Firewall, CDN)           │
└──────────────────┬──────────────────────────────────┘
                   │
┌──────────────────▼──────────────────────────────────┐
│  Layer 2: Application (CORS, Security Headers)     │
└──────────────────┬──────────────────────────────────┘
                   │
┌──────────────────▼──────────────────────────────────┐
│  Layer 3: Rate Limiting (Token Bucket, Per-IP)     │
└──────────────────┬──────────────────────────────────┘
                   │
┌──────────────────▼──────────────────────────────────┐
│  Layer 4: Authentication (JWT Validation)          │
└──────────────────┬──────────────────────────────────┘
                   │
┌──────────────────▼──────────────────────────────────┐
│  Layer 5: Authorization (Role Checks)              │
└──────────────────┬──────────────────────────────────┘
                   │
┌──────────────────▼──────────────────────────────────┐
│  Layer 6: Input Validation (Type, Range, Format)   │
└──────────────────┬──────────────────────────────────┘
                   │
┌──────────────────▼──────────────────────────────────┐
│  Layer 7: Data Protection (Encryption, Hashing)    │
└─────────────────────────────────────────────────────┘
```

### Authentication Flow

**Registration:**
```
User → POST /api/v1/auth/register
    ↓
Validate email/password
    ↓
Hash password (Argon2, 64MB memory, 3 iterations)
    ↓
Create user record
    ↓
Generate JWT (access + refresh tokens)
    ↓
Return tokens + user profile
```

**Login:**
```
User → POST /api/v1/auth/login
    ↓
Find user by email
    ↓
Verify password (Argon2)
    ↓
Generate JWT pair
    ↓
Store refresh token hash in database
    ↓
Return tokens + user profile
```

**Token Refresh:**
```
User → POST /api/v1/auth/refresh (with refresh_token)
    ↓
Validate refresh token (not expired, not revoked)
    ↓
Generate NEW JWT pair
    ↓
Revoke old refresh token
    ↓
Store new refresh token hash
    ↓
Return new tokens
```

**Protected Request:**
```
User → GET /api/v1/issues/:id (with Authorization: Bearer <token>)
    ↓
Auth middleware extracts token
    ↓
Verify JWT signature + expiration
    ↓
Extract user ID from claims
    ↓
Set user_id in request extensions
    ↓
Route handler accesses user_id
    ↓
Authorization check (if role required)
    ↓
Execute business logic
```

### Phone OTP Flow

```
1. Request OTP:
   POST /api/v1/auth/otp/request { phone: "+66812345678" }
   ↓
   Generate 6-digit code
   ↓
   Hash code (SHA-256)
   ↓
   Store in phone_otp_challenges (expires in 5 min)
   ↓
   Send SMS (planned: Twilio/AWS SNS)
   ↓
   Return challenge_id

2. Verify OTP:
   POST /api/v1/auth/otp/verify { phone, code }
   ↓
   Retrieve latest active challenge for phone
   ↓
   Check expiration + attempt count
   ↓
   Verify hashed code
   ↓
   Mark challenge consumed
   ↓
   Generate verification token (7-day expiry)
   ↓
   Return verification_token

3. Use Token (during issue creation):
   POST /api/v1/issues { ..., otp_phone, otp_token }
   ↓
   Validate token not expired/consumed
   ↓
   Mark user's phone as verified
   ↓
   Boost verification_score
```

---

## API Design

### REST Principles

- **Resource-based URLs:** `/api/v1/issues`, `/api/v1/issues/:id`
- **HTTP methods:** GET (read), POST (create), PATCH (update), DELETE (soft delete)
- **Status codes:** 200 OK, 201 Created, 400 Bad Request, 401 Unauthorized, 404 Not Found, 500 Internal Server Error
- **JSON payloads:** All requests/responses use JSON

### Endpoint Catalog

| Method | Endpoint | Auth | Description |
|--------|----------|------|-------------|
| GET | `/health` | No | Health check |
| GET | `/api/v1/info` | No | API version info |
| POST | `/api/v1/auth/register` | No | Create account |
| POST | `/api/v1/auth/login` | No | Login |
| POST | `/api/v1/auth/refresh` | No | Refresh JWT |
| POST | `/api/v1/auth/logout` | No | Revoke refresh token |
| POST | `/api/v1/auth/otp/request` | No | Request phone OTP |
| POST | `/api/v1/auth/otp/verify` | No | Verify OTP code |
| POST | `/api/v1/uploads` | No | Upload image |
| GET | `/api/v1/issues` | No | List issues (paginated) |
| POST | `/api/v1/issues` | No | Create issue |
| GET | `/api/v1/issues/:id` | No | Get issue details |
| PATCH | `/api/v1/issues/:id` | Yes | Update issue |
| DELETE | `/api/v1/issues/:id` | Yes (Admin) | Soft delete issue |
| GET | `/api/v1/issues/:id/comments` | No | List comments |
| POST | `/api/v1/issues/:id/comments` | Yes | Add comment |
| GET | `/api/v1/issues/:id/history` | No | Status history |
| GET | `/api/v1/analytics` | No | Analytics summary |
| GET | `/uploads/:filename` | No | Serve uploaded file |

### Pagination Response Format

```json
{
  "success": true,
  "data": [...],
  "meta": {
    "total": 42,
    "page": 1,
    "per_page": 20,
    "total_pages": 3
  }
}
```

### Error Response Format

```json
{
  "success": false,
  "error": "Invalid JWT token",
  "details": "Token has expired"
}
```

---

## Deployment Architecture

### Development (Local)

```
┌────────────────────────────────────────────┐
│  Developer Machine                         │
│  ┌──────────────┐      ┌──────────────┐   │
│  │   Backend    │      │   Frontend   │   │
│  │ cargo run    │      │  npm run dev │   │
│  │ :3000        │◀─────│  :5173       │   │
│  └──────┬───────┘      └──────────────┘   │
│         │                                  │
│  ┌──────▼───────┐                          │
│  │  SQLite DB   │                          │
│  │  data/       │                          │
│  └──────────────┘                          │
└────────────────────────────────────────────┘
```

### Production (Docker Compose)

```
┌──────────────────────────────────────────────┐
│  VPS / Cloud VM                              │
│  ┌────────────────────────────────────────┐ │
│  │  Docker Compose                        │ │
│  │  ┌──────────────┐  ┌──────────────┐   │ │
│  │  │   nginx      │  │   Backend    │   │ │
│  │  │   :80/:443   │─▶│   (Rust)     │   │ │
│  │  │              │  │   :3000      │   │ │
│  │  └──────────────┘  └──────┬───────┘   │ │
│  │                            │           │ │
│  │                     ┌──────▼───────┐   │ │
│  │                     │  libSQL DB   │   │ │
│  │                     │  (volume)    │   │ │
│  │                     └──────────────┘   │ │
│  └────────────────────────────────────────┘ │
└──────────────────────────────────────────────┘
```

### Production (Cloud-Native with Turso)

```
┌───────────────┐       ┌────────────────────┐
│   Cloudflare  │       │   Vercel / Fly.io  │
│   CDN + WAF   │──────▶│   Backend API      │
└───────────────┘       │   (Rust)           │
                        └──────────┬─────────┘
                                   │
                        ┌──────────▼─────────┐
                        │   Turso DB         │
                        │   (Edge-replicated)│
                        └────────────────────┘
                                   │
                        ┌──────────▼─────────┐
                        │   S3 / R2          │
                        │   (File uploads)   │
                        └────────────────────┘
```

---

## Design Decisions

### Why Rust for Backend?

**Pros:**
- Memory safety without garbage collection (prevents entire classes of bugs)
- Strong type system catches errors at compile time
- Excellent async runtime (Tokio) for high concurrency
- Zero-cost abstractions (performance without complexity)
- Growing ecosystem for web development

**Cons:**
- Steeper learning curve than Python/Node.js
- Slower compilation times
- Smaller talent pool

**Decision:** Performance and correctness requirements justify the trade-off.

### Why Axum over Actix-web?

**Axum Advantages:**
- Built on Tokio (same ecosystem as most async crates)
- Extractors make handler signatures clean
- Strong typing prevents runtime errors
- Better ergonomics for middleware
- Maintained by Tokio team (long-term stability)

### Why libSQL over PostgreSQL?

**libSQL Advantages:**
- Zero-config for self-hosting (single file)
- Turso provides free edge-replicated hosting
- SQLite performance sufficient for MVP scale (<100k users)
- FTS5 built-in for full-text search
- Simpler deployment (no separate database server)

**PostgreSQL Migration Path:**
- If scale requires (>1M issues, high write concurrency)
- Schema is straightforward to migrate
- Connection pooling becomes critical

### Why localStorage for Tokens?

**Decision:** Simplicity over maximum security for MVP.

**Trade-offs:**
- **Risk:** XSS attacks can steal tokens from localStorage
- **Mitigation:** Content-Security-Policy headers (planned)
- **Alternative:** httpOnly cookies (requires CSRF tokens, more complexity)
- **Future:** Migrate to httpOnly cookies post-MVP

### Why React over Vue/Svelte?

**React Advantages:**
- Largest ecosystem and community
- More developers familiar with it
- Extensive component libraries (Radix, shadcn/ui)
- Proven at scale

**Decision:** Developer availability and ecosystem maturity matter more than framework size.

### Why Zustand over Redux?

**Zustand Advantages:**
- Minimal boilerplate (no actions, reducers, providers)
- Only 1KB gzipped
- Simple API (just hooks)
- No React Context Provider needed

**Decision:** Auth state is simple enough that Redux overhead is unnecessary.

---

## Future Enhancements

### Phase 1: Security Hardening (Q3 2026)
- [ ] Upload validation (file type, size, MIME, sanitization)
- [ ] JWT secret validation on startup
- [ ] Request body size limits
- [ ] CSRF protection
- [ ] Content-Security-Policy headers

### Phase 2: Performance (Q4 2026)
- [ ] Database connection pooling
- [ ] Redis for rate limiting (distributed)
- [ ] Image optimization and CDN
- [ ] Code splitting and lazy loading
- [ ] Service worker for offline support

### Phase 3: Features (2027)
- [ ] WebSocket for real-time updates
- [ ] Email/SMS notifications
- [ ] Full-text search endpoint
- [ ] Heat map visualization
- [ ] Admin dashboard (user management)
- [ ] Export to CSV/PDF
- [ ] Bulk actions

### Phase 4: Scale (2027+)
- [ ] Multi-region deployment (Turso edge replicas)
- [ ] Horizontal scaling (load balancer + multiple API instances)
- [ ] PostgreSQL migration (if needed)
- [ ] ML-based duplicate detection
- [ ] Predictive maintenance algorithms

---

## References

- [Axum Documentation](https://docs.rs/axum/)
- [libSQL Documentation](https://docs.turso.tech/)
- [React Documentation](https://react.dev/)
- [SQLite FTS5](https://www.sqlite.org/fts5.html)
- [OWASP Top 10](https://owasp.org/www-project-top-ten/)

---

**Maintained by:** [@FirstPrinciples-Sun](https://github.com/FirstPrinciples-Sun)  
**Last Updated:** July 3, 2026
