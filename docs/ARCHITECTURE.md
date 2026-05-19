# Civic Sentinel Architecture

## Overview
Civic Sentinel is a civic issue reporting and tracking platform. This document explains how the system is organized.

## System Components

```
┌────────────────────────────────────────────────────────────┐
│                     CLIENT LAYER                            │
│  ┌──────────────┐  ┌──────────────┐  ┌─────────────────┐  │
│  │  React Web   │  │  Mobile PWA  │  │  Admin Portal   │  │
│  └──────┬───────┘  └──────┬───────┘  └────────┬────────┘  │
└─────────┼─────────────────┼───────────────────┼───────────┘
          │                 │                   │
          └─────────────────┴───────────────────┘
                            │ HTTPS
┌───────────────────────────▼────────────────────────────────┐
│                   API (Rust/Axum)                           │
│  ┌────────────┐  ┌────────────┐  ┌─────────────────────┐  │
│  │ REST API   │  │ WebSocket  │  │ Rate Limiter        │  │
│  └────────────┘  └────────────┘  └─────────────────────┘  │
└───────────────────────────┬────────────────────────────────┘
                            │
┌───────────────────────────▼────────────────────────────────┐
│                   SERVICE LAYER                             │
│  ┌──────────────┐ ┌─────────────┐ ┌─────────────────────┐ │
│  │Issue Service │ │AI Service   │ │Notification Service │ │
│  └──────────────┘ └─────────────┘ └─────────────────────┘ │
└───────────────────────────┬────────────────────────────────┘
                            │
┌───────────────────────────▼────────────────────────────────┐
│                   DATA LAYER                                │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  SQLite / libSQL (Turso)                            │   │
│  └─────────────────────────────────────────────────────┘   │
└────────────────────────────────────────────────────────────┘
```

## Backend (Rust)

Built with:
- **Axum** — Async web framework
- **Tokio** — Async runtime
- **libSQL** — Database connectivity

Handles:
- HTTP API requests
- WebSocket connections for real-time updates
- Rate limiting per IP
- JWT authentication
- Input validation

## Frontend (React)

Built with:
- **React 18** — UI framework
- **TypeScript** — Type safety
- **Tailwind CSS** — Styling
- **Vite** — Build tool

Pages:
- Home — Overview and navigation
- Report — Submit new issues
- Map — View issues on a map
- Dashboard — Analytics and status

## WebAssembly Module

A Rust module compiled to WebAssembly that runs in the browser:
- Analyzes issue text locally (no server round-trip)
- Suggests category and priority
- Detects similar issues
- Keeps user data private

## Database

Uses SQLite by default (simple, zero-config). For production:
- **libSQL (Turso)** — Distributed SQLite with replication

Tables:
- `users` — Accounts and roles
- `issues` — Issue reports
- `issue_comments` — Discussion threads
- `issue_status_history` — Status change log
- `refresh_tokens` — Session management

## Security

- Passwords hashed with Argon2
- JWT tokens for authentication
- Rate limiting on all endpoints
- Security headers on all responses
- CORS configured per environment
- Input validation with Zod (frontend) and validator (backend)

## Deployment

Simple options:
1. **Docker Compose** — One command, everything runs
2. **Fly.io** — Cloud hosting with free tier
3. **Railway** — Easy cloud deployment
4. **VPS** — Any Linux server

See [deployment docs](deployment/SELF_HOSTING.md) for details.
