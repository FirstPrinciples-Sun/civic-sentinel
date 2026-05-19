# Civic Sentinel — Agent Guide

## Project Overview
Civic Sentinel is an AI-powered civic resilience platform that empowers communities to detect, prioritize, and resolve public issues through crowdsourced intelligence and automated response coordination.

## Tech Stack
- **Backend**: Rust (Axum) + libSQL (Turso)
- **Frontend**: React 18 + TypeScript + Vite + Tailwind CSS
- **WASM**: Rust → WebAssembly for browser-side ML inference
- **AI**: Local inference via candle (Rust ML framework)
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
