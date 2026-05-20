# Civic Sentinel — Agent Guide

## Project Overview
Civic Sentinel is a civic issue reporting and tracking platform that helps communities report, prioritize, and resolve public issues through crowdsourced reports and organized response coordination.

## Tech Stack
- **Backend**: Rust (Axum) + libSQL (Turso)
- **Frontend**: React 18 + TypeScript + Vite + Tailwind CSS
- **WASM**: Rust → WebAssembly for browser-side rule-based text analysis (keyword scoring)
- **AI / ML**: Planned — no ML/LLM inference is implemented yet
- **Infrastructure**: Docker + GitHub Actions

## Build Commands
```bash
# Full stack
docker-compose up

# Backend only
cd apps/api && cargo run

# Frontend only
cd apps/web && npm run dev

# WASM
cd crates/wasm-analytics && wasm-pack build --target web
```

## Testing
```bash
# Rust
cargo test --workspace

# TypeScript
npm run test
```
