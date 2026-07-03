# Civic Sentinel

> An open-source platform for communities to report, track, and resolve civic issues.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

---

## Current Status

MVP is functionally complete with recent security hardening. Backend compiles, runs, and serves all endpoints. Frontend builds and includes auth pages. End-to-end flow (register → login → create issue → view analytics) is verified working.

**Last updated:** July 3, 2026  
**Current phase:** MVP complete — security improvements in progress

---

## What It Is

Civic Sentinel is a web application that lets people in a community report problems they encounter — broken roads, safety concerns, utility outages, and similar issues. Reports are stored, categorized, and tracked through a resolution workflow visible to everyone.

The goal is to give small communities, neighborhoods, or organizations a free, self-hosted alternative to expensive issue-management software.

---

## Technology

| Component | Technology | Rationale |
|-----------|-----------|-----------|
| Backend API | Rust (Axum) | Strong type system, reliable async runtime |
| Frontend | React 18 + TypeScript + Tailwind CSS | Widely used, good developer experience |
| Build tool | Vite | Fast dev server, modern bundling |
| Database | SQLite / libSQL (Turso) | Zero-config for self-hosting; optional cloud replication |
| Containerization | Docker + Docker Compose | Single-command deployment |
| CI/CD | GitHub Actions | Automated testing and builds |

A WebAssembly analytics module (Rust → WASM) is planned for browser-side issue analysis, but is not yet integrated.

---

## What Works Now

- **Backend API** — Rust (Axum) compiles with `cargo clippy -D warnings`, runs with `cargo run`
- **Database** — SQLite/libSQL with migrations for users, issues, comments, status history, refresh tokens
- **Auth system** — Register, login, refresh token, logout with JWT + argon2 password hashing
- **Phone OTP Verification** — SMS-based phone verification flow for identity confirmation
- **Protected routes** — POST/PATCH/DELETE issues/comments require valid JWT (returns 401 otherwise)
- **Issue CRUD** — Create, list (with pagination/filter), get by ID, update status, soft delete
- **Advanced Triage System** — Automatic priority scoring based on verification, corroboration, and age
- **Duplicate Detection** — Geographic proximity + similarity matching with corroboration counting
- **Verification Scoring** — Trust scoring (0-100) based on media, identity, account age, resolution history
- **Analytics** — Real aggregation from database (total, open, resolved, by category, by priority, impact score)
- **Middleware** — Security headers + **improved rate limiting** (token bucket with cleanup) + CORS configured
- **Frontend** — React + Vite builds clean; pages: Home, Report, Map, Dashboard, Login, Register, Admin, Issue Detail
- **Issue detail UI** — `/issues/:id` page with comments and status history timeline
- **Photo uploads** — multipart image upload endpoint + report form image uploader with preview
- **Auth context** — localStorage persistence, JWT auto-attach via Axios interceptor, protected route guards
- **Docker** — Multi-stage Dockerfile + docker-compose.yml ready
- **CI/CD** — GitHub Actions for Rust build and web build

## Recent Security Improvements (July 2026)

- ✅ **SQL Injection Protection** — Enum-based filtering prevents SQL injection in list queries
- ✅ **Rate Limiter Fix** — Shared instance in AppState with automatic cleanup (fixes memory leak)
- ✅ **IP Extraction** — Improved client IP detection supporting X-Forwarded-For and X-Real-IP headers
- ✅ **Upload Validation** — Comprehensive file validation (MIME type whitelist, magic bytes verification, filename sanitization, size limits)
- ✅ **JWT Secret Validation** — Startup validation rejects default/weak secrets before server starts
- ✅ **Request Body Limits** — 2MB limit for JSON requests, 15MB for file uploads (prevents DoS attacks)

## What Is Planned Next

### High Priority
- [x] Upload validation (file type whitelist, size limits, MIME validation, filename sanitization) ✅ **COMPLETED**
- [x] JWT secret validation on startup (reject default/weak secrets) ✅ **COMPLETED**
- [x] Request body size limits (prevent DoS via large payloads) ✅ **COMPLETED**
- [ ] Admin role guards on backend (restrict sensitive operations)
- [ ] Full-text search via SQLite FTS5 (table exists, needs endpoint integration)
- [ ] Input validation (length limits on title/description, enhanced validation rules)

### Medium Priority
- [ ] Email/LINE notifications for status changes
- [ ] Heat map visualization on Map page
- [ ] Database connection pooling (replace single connection with pool)
- [ ] Integration tests (auth flow, issue CRUD, role enforcement)
- [ ] CSRF protection for state-changing requests

### Future Enhancements
- [ ] Deploy a public demo instance
- [ ] WebSocket for real-time dashboard updates
- [ ] Code splitting and lazy loading
- [ ] Image optimization and thumbnail generation
- [ ] PWA support with offline capabilities

---

## MVP Scope

The minimum viable product targets the following functionality:

1. **Issue reporting** — A web form where anyone can submit an issue with title, description, category, and location.
2. **Issue CRUD** — View a list of issues, open an issue detail page, and update its status.
3. **Basic admin dashboard** — A simple view showing issue counts by status and category.
4. **Status tracking** — Issues move through statuses: Reported → Under Review → In Progress → Resolved.
5. **Category and priority fields** — Each issue has a category (Infrastructure, Safety, Environment, etc.) and a priority level (Low, Medium, High, Critical).

Features outside the MVP scope — such as LINE integration, SMS alerts, AI spam detection, heat maps, SLA routing, public open-data API, or predictive maintenance — are noted in the roadmap but are not claimed as implemented.

---

## Architecture

### Current MVP Architecture

```
┌──────────────────────────────────────────────────────────┐
│                    React Frontend                         │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐ │
│  │  Home    │  │ Report   │  │  Map     │  │ Dashboard│ │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘ │
└───────┼─────────────┼─────────────┼─────────────┼───────┘
        │             │             │             │
        └─────────────┴─────────────┴─────────────┘
                            │ HTTPS
┌───────────────────────────▼──────────────────────────────┐
│               Rust API (Axum)                             │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐              │
│  │  Issues  │  │  Health  │  │  Info    │              │
│  │  routes  │  │  check   │  │  route   │              │
│  └────┬─────┘  └──────────┘  └──────────┘              │
└───────┼──────────────────────────────────────────────────┘
        │ (planned — not yet connected)
┌───────▼──────────────────────────────────────────────────┐
│            SQLite / libSQL Database                       │
│   users · issues · comments · status_history             │
└──────────────────────────────────────────────────────────┘
```

### Future Target Architecture

```
┌──────────────────────────────────────────────────────────┐
│                    React Frontend                         │
│          (PWA, multi-language, offline support)           │
└─────────────────────────┬────────────────────────────────┘
                          │ HTTPS / WebSocket
┌─────────────────────────▼────────────────────────────────┐
│               Rust API (Axum)                             │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐              │
│  │  Issues  │  │   Auth   │  │Analytics │              │
│  │  Auth    │  │  JWT     │  │  Export  │              │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘              │
└───────┼─────────────┼─────────────┼──────────────────────┘
        │             │             │
        └─────────────┴─────────────┘
                    │
┌───────────────────▼──────────────────────────────────────┐
│            libSQL (Turso) — Edge Replicas                 │
│   users · issues · comments · status_history · fts5       │
└──────────────────────────────────────────────────────────┘
        │
┌───────▼──────────────────────────────────────────────────┐
│   WebAssembly Module (Browser)                            │
│   Text analysis · Category suggestion · Similarity        │
└──────────────────────────────────────────────────────────┘
```

---

## Quick Start

### Prerequisites
- [Rust](https://rustup.rs/) 1.75+
- [Node.js](https://nodejs.org/) 20+
- [Docker](https://docker.com/) (optional)

### Run with Docker

```bash
git clone https://github.com/FirstPrinciples-Sun/civic-sentinel.git
cd civic-sentinel
cp .env.example .env
# Edit .env and set JWT_SECRET to a random string (32+ characters)
docker-compose up
```

The frontend will be available at http://localhost:5173 and the API at http://localhost:3000.

### Run Locally

Backend:
```bash
cd apps/api
cargo run
```

Frontend:
```bash
cd apps/web
npm install
npm run dev
```

---

## Development

### Backend
```bash
cd apps/api
cargo fmt          # Format
cargo clippy       # Lint
cargo test         # Test
cargo run          # Run dev server
```

### Frontend
```bash
cd apps/web
npm run lint       # Lint
npm run typecheck  # Type check
npm run dev        # Dev server
npm run build      # Production build
```

### WebAssembly (planned)
```bash
cd crates/wasm-analytics
wasm-pack build --target web
```

---

## Security & Production Checklist

Before deploying to production, ensure you:

- [ ] **Change JWT_SECRET** to a strong random value (32+ characters): `openssl rand -base64 32`
- [ ] **Set strong DATABASE_URL** and enable Turso auth token if using remote database
- [ ] **Review CORS_ORIGINS** — remove `*` wildcard, specify exact frontend URLs
- [ ] **Enable HTTPS** — use reverse proxy (nginx/Caddy) or managed platform SSL
- [ ] **Configure rate limits** — adjust `RATE_LIMIT_REQUESTS` and `RATE_LIMIT_WINDOW_SECONDS` for your traffic
- [ ] **Set up backups** — regular SQLite database backups or use Turso's built-in replication
- [ ] **Review logs** — set `RUST_LOG=info` for production, `debug` for troubleshooting
- [ ] **Validate uploads** — ensure `UPLOAD_DIR` has proper permissions and disk space monitoring
- [ ] **Test authentication flow** — verify JWT refresh, OTP delivery, and session management
- [ ] **Monitor resources** — track API response times, database query performance, memory usage

---

## Deployment

See [docs/deployment/SELF_HOSTING.md](docs/deployment/SELF_HOSTING.md) for options:
- Docker Compose (local or VPS)
- Fly.io
- Railway

---

## Contributing

Contributions are welcome. Please read [CONTRIBUTING.md](.github/CONTRIBUTING.md) for guidelines.

Quick start for contributors:
1. Fork the repo
2. Create a branch from `develop`
3. Make changes
4. Open a pull request to `develop`

---

## License

[MIT License](LICENSE)

Free to use, modify, and distribute. No attribution required, though it is appreciated.

---

## Changelog

### July 3, 2026 - Security Hardening Phase 1 Complete ✅
**Critical Security Fixes:**
- Fixed SQL injection vulnerability in issue list filtering (enum-based validation)
- Improved rate limiter efficiency and memory management (shared instance with automatic cleanup)
- Enhanced client IP extraction supporting multiple proxy headers
- **NEW:** Comprehensive upload validation with magic bytes verification, MIME type whitelist, and filename sanitization
- **NEW:** JWT secret validation on startup - rejects default/weak secrets before server starts
- **NEW:** Request body size limits (2MB JSON, 15MB uploads) to prevent DoS attacks

**Technical Improvements:**
- Rate limiter now stored in AppState (prevents recreation on every request)
- Background cleanup task for rate limiter buckets (runs every 5 minutes)
- Improved logging for security events
- Added comprehensive test coverage for security features

**Documentation:**
- Added SECURITY.md with complete security policy
- Added CODE_REVIEW_CHECKLIST.md for consistent code reviews
- Added ARCHITECTURE.md with system design documentation
- Added DEVELOPER_GUIDE.md for onboarding new contributors
- Added API_REFERENCE.md with complete endpoint documentation
- Added comprehensive security documentation and production checklist

### May 20, 2026 - MVP Complete
- Initial MVP release with full CRUD operations
- Authentication system with JWT and refresh tokens
- Phone OTP verification
- Advanced triage and verification scoring
- Frontend with React 18 and Tailwind CSS

---

## Contact

- **Issues & bugs:** [GitHub Issues](../../issues)
- **Questions & ideas:** [GitHub Discussions](../../discussions)
- **Maintainer:** [@FirstPrinciples-Sun](https://github.com/FirstPrinciples-Sun)
