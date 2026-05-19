# Civic Sentinel

> An open-source platform for communities to report, track, and resolve civic issues.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

---

## Current Status

This project is under active development. The API and frontend scaffolding are in place, along with a Docker-based deployment setup and CI/CD pipelines. The database layer, authentication, and end-to-end issue lifecycle are partially implemented but not yet wired into the UI.

**Last updated:** May 2026  
**Current phase:** MVP scaffolding & core API development

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

These parts are implemented and functional at the API or UI level:

- **Project structure** — Rust workspace, React app, WASM crate, Docker setup
- **Health check endpoint** — `GET /health` returns service status
- **Issue model and routes** — `GET /api/v1/info`, `POST /api/v1/issues`, `GET /api/v1/issues/:id`
- **React frontend shell** — Routing, layout, and page stubs for Home, Report, Map, and Dashboard
- **Docker Compose setup** — Builds and runs the backend and frontend containers
- **CI pipelines** — GitHub Actions for Rust formatting, clippy, tests, and builds
- **Middleware scaffolding** — JWT auth, rate limiting, and security headers (not all wired to routes yet)
- **Database schema** — SQL migrations for users, issues, comments, status history, and refresh tokens

## What Is Planned Next

- [ ] Wire the database into API handlers (currently returns empty/mock data)
- [ ] Implement user registration and login (UI + API)
- [ ] Complete issue CRUD: create, read, update status, list with filters
- [ ] Connect the report form to the API
- [ ] Populate the dashboard with real data
- [ ] Add file upload for issue photos
- [ ] Integrate the WebAssembly module for browser-side text analysis
- [ ] Deploy a public demo instance

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

## Contact

- **Issues & bugs:** [GitHub Issues](../../issues)
- **Questions & ideas:** [GitHub Discussions](../../discussions)
- **Maintainer:** [@FirstPrinciples-Sun](https://github.com/FirstPrinciples-Sun)
