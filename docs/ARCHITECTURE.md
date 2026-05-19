# Civic Sentinel Architecture

## Overview
Civic Sentinel is a modern civic tech platform built on a **Rust + React + WebAssembly** stack, designed for high performance, privacy, and scalability.

## System Architecture

```
┌────────────────────────────────────────────────────────────┐
│                     CLIENT LAYER                            │
│  ┌──────────────┐  ┌──────────────┐  ┌─────────────────┐  │
│  │  React Web   │  │  Mobile PWA  │  │  Admin Portal   │  │
│  │  (Vite+TS)   │  │  (Tauri/RN)  │  │  (React)        │  │
│  └──────┬───────┘  └──────┬───────┘  └────────┬────────┘  │
└─────────┼─────────────────┼───────────────────┼───────────┘
          │                 │                   │
          └─────────────────┴───────────────────┘
                            │ HTTPS/WSS
┌───────────────────────────▼────────────────────────────────┐
│                   API GATEWAY (Rust/Axum)                   │
│  ┌────────────┐  ┌────────────┐  ┌─────────────────────┐  │
│  │ REST API   │  │ WebSocket  │  │ Rate Limiter        │  │
│  │            │  │ Real-time  │  │ (Tower)             │  │
│  └────────────┘  └────────────┘  └─────────────────────┘  │
└───────────────────────────┬────────────────────────────────┘
                            │
┌───────────────────────────▼────────────────────────────────┐
│                   SERVICE LAYER                             │
│  ┌──────────────┐ ┌─────────────┐ ┌─────────────────────┐ │
│  │Issue Service │ │AI Service   │ │Notification Service │ │
│  │CRUD + Search │ │Classification│ │Email/SMS/LINE       │ │
│  └──────────────┘ └─────────────┘ └─────────────────────┘ │
│  ┌──────────────┐ ┌─────────────┐ ┌─────────────────────┐ │
│  │User Service  │ │Map Service  │ │Analytics Service    │ │
│  │Auth + Profile│ │Geo queries  │ │Dashboard data       │ │
│  └──────────────┘ └─────────────┘ └─────────────────────┘ │
└───────────────────────────┬────────────────────────────────┘
                            │
┌───────────────────────────▼────────────────────────────────┐
│                   DATA LAYER                                │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  libSQL (Turso) — Distributed SQLite at the Edge    │   │
│  │  • Issues table                                     │   │
│  │  • Users table                                      │   │
│  │  • Analytics views                                  │   │
│  │  • Geo-indexed queries                              │   │
│  └─────────────────────────────────────────────────────┘   │
└────────────────────────────────────────────────────────────┘
```

## Key Design Decisions

### 1. Rust Backend (Axum)
- **Why**: Memory safety, performance (handles 100K+ concurrent connections), modern async
- **Trade-off**: Steeper learning curve, but eliminates entire classes of bugs

### 2. WebAssembly Analytics
- **Why**: Privacy-preserving ML — user data never leaves their device
- **Use case**: Real-time issue classification, duplicate detection
- **Trade-off**: Limited to browser capabilities, but zero latency

### 3. libSQL (Turso)
- **Why**: Edge-ready SQLite — zero config, instant replicas, incredibly fast
- **Trade-off**: Not for massive scale (use PostgreSQL when you hit millions of users)

### 4. React 18 + Vite
- **Why**: Concurrent rendering, Suspense, fastest build tooling
- **Pattern**: Server state (React Query) + Client state (Zustand)

## Data Flow

### Issue Reporting Flow
```
User → React Form → Validation (Zod) → POST /api/v1/issues
  → Rust API → AI Classification → libSQL INSERT
  → WebSocket Broadcast → Real-time Map Update
```

### AI Analysis Flow
```
User types → WASM Module (browser) → Text Analysis
  → Priority Score → Category Detection → UI Update
  → If submitted → Rust API validates → Database
```

## Security Model
- **Authentication**: JWT with Argon2 password hashing
- **Authorization**: Role-based (Reporter, Responder, Admin)
- **Data**: PII encrypted at rest
- **API**: Rate limiting, input validation, CORS
- **WASM**: Runs in browser sandbox, no network access

## Scalability Path
1. **Current**: Single server + SQLite (handles 1K users)
2. **Phase 2**: Horizontal scaling + Turso replicas (handles 10K users)
3. **Phase 3**: PostgreSQL + Redis cache (handles 100K users)
4. **Phase 4**: Microservices + Kubernetes (handles 1M+ users)

## Performance Targets
- API response: <50ms p99
- WASM analysis: <10ms
- Page load: <2s (3G)
- Time to interactive: <3s
