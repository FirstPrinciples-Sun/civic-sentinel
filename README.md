# 🛡️ Civic Sentinel

> **AI-Powered Civic Resilience Platform** — Empowering communities to detect, prioritize, and resolve public issues through crowdsourced intelligence and automated response coordination.

[![Rust CI](https://github.com/FirstPrinciples-Sun/civic-sentinel/actions/workflows/rust-ci.yml/badge.svg)](https://github.com/FirstPrinciples-Sun/civic-sentinel/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust Version](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)
[![React Version](https://img.shields.io/badge/react-18%2B-blue.svg)](https://react.dev)

---

## 🌍 Mission

**Every community deserves to be heard. Every issue deserves a response.**

Civic Sentinel bridges the gap between citizens and responders by:
- 🔍 **Detecting** community issues in real-time through crowdsourced reports
- 🧠 **Analyzing** urgency and impact using on-device AI (WebAssembly)
- ⚡ **Routing** issues to the right responders automatically
- 📊 **Tracking** resolution progress with transparent metrics
- 🤝 **Coordinating** multi-agency responses seamlessly

---

## 🚀 Tech Stack

### Backend — `apps/api/`
| Technology | Purpose |
|-----------|---------|
| **Rust** | Systems programming language with zero-cost abstractions |
| **Axum** | Async web framework (Tokio ecosystem) — handles 100K+ concurrent connections |
| **libSQL (Turso)** | Distributed SQLite — edge-ready, zero-config database |
| **Tokio** | Async runtime — the engine behind Axum |
| **Serde** | Serialization framework |
| **Tower** | Middleware, timeouts, rate limiting |

### Frontend — `apps/web/`
| Technology | Purpose |
|-----------|---------|
| **React 18** | Concurrent rendering, Suspense |
| **TypeScript** | Type-safe JavaScript |
| **Vite** | Next-gen build tool (100x faster than CRA) |
| **Tailwind CSS** | Utility-first CSS |
| **React Query** | Server state management |
| **Zustand** | Client state management |

### WebAssembly — `crates/wasm-analytics/`
| Technology | Purpose |
|-----------|---------|
| **Rust → WASM** | Browser-native ML inference |
| **candle** | Minimalist ML framework in Rust |
| **wasm-bindgen** | Rust/JavaScript interop |
| **web-sys** | Web API bindings |

### Infrastructure
| Technology | Purpose |
|-----------|---------|
| **Docker** | Containerization |
| **GitHub Actions** | CI/CD |
| **Fly.io / Railway** | Edge deployment |

---

## 📦 Quick Start

### Prerequisites
- [Rust](https://rustup.rs/) 1.75+
- [Node.js](https://nodejs.org/) 20+
- [Docker](https://docker.com/) (optional)

### One-Command Setup
```bash
git clone https://github.com/FirstPrinciples-Sun/civic-sentinel.git
cd civic-sentinel
docker-compose up
```

### Development Mode
```bash
# Terminal 1 — Backend
cd apps/api && cargo run

# Terminal 2 — Frontend
cd apps/web && npm install && npm run dev

# Terminal 3 — WASM (optional)
cd crates/wasm-analytics && wasm-pack build --target web
```

---

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      CLIENT LAYER                            │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │   React Web  │  │  Mobile PWA  │  │  Admin Dashboard │  │
│  └──────────────┘  └──────────────┘  └──────────────────┘  │
└────────────────────┬────────────────────────────────────────┘
                     │ HTTPS / WebSocket
┌────────────────────▼────────────────────────────────────────┐
│                    API GATEWAY                               │
│         Rust (Axum) — 100K+ concurrent                       │
│  ┌─────────────┐  ┌──────────────┐  ┌──────────────────┐   │
│  │   REST API  │  │ WebSocket    │  │   Rate Limiter   │   │
│  └─────────────┘  └──────────────┘  └──────────────────┘   │
└────────────────────┬────────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────────┐
│                   SERVICE LAYER                              │
│  ┌──────────────┐ ┌─────────────┐ ┌──────────────────────┐ │
│  │ Issue Service│ │ AI Service  │ │ Notification Service │ │
│  └──────────────┘ └─────────────┘ └──────────────────────┘ │
│  ┌──────────────┐ ┌─────────────┐ ┌──────────────────────┐ │
│  │ User Service │ │ Map Service │ │ Analytics Service    │ │
│  └──────────────┘ └─────────────┘ └──────────────────────┘ │
└────────────────────┬────────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────────┐
│                   DATA LAYER                                 │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  libSQL (Turso) — Distributed SQLite at the Edge     │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────────┐
│              AI / WASM LAYER (Browser)                       │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  Rust → WebAssembly — On-device issue classification │  │
│  │  candle ML — Privacy-preserving local inference      │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

---

## ✨ Features

### 🎯 Issue Detection
- **Multi-channel reporting**: Web, PWA, LINE, SMS
- **Media attachments**: Photos, videos, geolocation
- **Real-time validation**: AI-powered spam detection
- **Anonymous reporting**: Protect whistleblowers

### 🧠 AI Prioritization
- **Urgency scoring**: AI ranks issues by severity
- **Category classification**: Auto-tag (infrastructure, safety, environment)
- **Duplicate detection**: Merge similar reports
- **Trend analysis**: Predict emerging issues

### ⚡ Smart Routing
- **Responder matching**: Route to correct department
- **SLA tracking**: Monitor response times
- **Escalation chains**: Auto-escalate overdue issues
- **Resource optimization**: Balance workload

### 📊 Transparency Dashboard
- **Public status page**: Track all issues
- **Impact metrics**: Issues resolved, response times
- **Heat maps**: Visualize problem areas
- **Open data API**: Raw data for researchers

---

## 🗺️ Roadmap

### Phase 1: Foundation (Now)
- [x] Project scaffolding
- [x] Core API (Rust + Axum)
- [x] Basic React frontend
- [ ] Issue CRUD operations
- [ ] User authentication
- [ ] Database schema

### Phase 2: Intelligence (Month 2)
- [ ] WASM analytics module
- [ ] AI categorization engine
- [ ] Priority scoring algorithm
- [ ] Duplicate detection
- [ ] Real-time notifications

### Phase 3: Scale (Month 3)
- [ ] Multi-tenant support
- [ ] Mobile PWA
- [ ] LINE integration
- [ ] Offline-first architecture
- [ ] Public API

### Phase 4: Ecosystem (Month 4+)
- [ ] Plugin system
- [ ] Third-party integrations
- [ ] Community marketplace
- [ ] White-label solution
- [ ] Internationalization

---

## 🤝 Contributing

We believe **civic tech should be built by the community, for the community**.

### How to Contribute
1. 🍴 Fork the repository
2. 🌿 Create a feature branch (`git checkout -b feature/amazing-feature`)
3. 💻 Write code with tests
4. 📤 Push to your fork (`git push origin feature/amazing-feature`)
5. 📬 Open a Pull Request

### Development Setup
```bash
# Install Rust components
rustup component add rustfmt clippy

# Install Node dependencies
cd apps/web && npm install

# Run tests
cargo test --workspace
npm run test

# Format code
cargo fmt
npm run lint
```

### Code Standards
- **Rust**: Follow `cargo clippy` and `rustfmt`
- **TypeScript**: Follow ESLint config
- **Commits**: Use [Conventional Commits](https://www.conventionalcommits.org/)
- **Documentation**: Every public API must be documented

---

## 📜 License

This project is dual-licensed under:
- [MIT License](LICENSE) — for maximum flexibility
- [Apache-2.0 License](LICENSE-APACHE) — for patent protection

**We believe civic technology should be free and open for everyone.**

---

## 🙏 Acknowledgments

- [Tokio](https://tokio.rs/) team for the amazing async ecosystem
- [Turso](https://turso.tech/) for edge-ready SQLite
- [candle](https://github.com/huggingface/candle) for Rust ML
- Every contributor who believes in open civic technology

---

## 🔗 Connect

- **GitHub**: [@FirstPrinciples-Sun](https://github.com/FirstPrinciples-Sun)
- **Issues**: [Report a bug or request a feature](../../issues)
- **Discussions**: [Join the conversation](../../discussions)

---

> *"The best way to predict the future is to build it — together."*

**Built with ❤️ by FirstPrinciples-Sun and the open source community.**
