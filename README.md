# 🛡️ Civic Sentinel

> **AI-Powered Civic Resilience Platform** — A tool for communities to report, track, and resolve public issues together.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

---

## 🌍 About

Civic Sentinel helps people in any community report problems they see around them — broken roads, safety concerns, environmental issues, and more — and ensures those reports reach the right people who can fix them.

Everything is open source and free. No hidden costs. No premium tiers.

### What It Does
- 📍 **Report Issues** — Citizens report problems with photos and location
- 🧠 **AI Assistance** — On-device analysis helps categorize and prioritize (privacy-first)
- ⚡ **Smart Routing** — Reports go to the correct department or responder
- 📊 **Track Progress** — Everyone can see status updates transparently

---

## 🏗️ Technology

This project is built with:

| Component | Technology |
|-----------|-----------|
| **Backend** | Rust (Axum) |
| **Frontend** | React 18 + TypeScript + Tailwind CSS |
| **AI (Browser)** | Rust compiled to WebAssembly |
| **Database** | SQLite / libSQL (Turso) |
| **Build** | Vite |
| **Deploy** | Docker |

We chose these tools because they are reliable, well-maintained, and have strong communities.

---

## 📦 Getting Started

### Prerequisites
- [Rust](https://rustup.rs/) 1.75+
- [Node.js](https://nodejs.org/) 20+
- [Docker](https://docker.com/) (optional)

### Quick Start
```bash
git clone https://github.com/FirstPrinciples-Sun/civic-sentinel.git
cd civic-sentinel
cp .env.example .env
# Edit .env — set JWT_SECRET to a strong random string
docker-compose up
```

Visit http://localhost:5173

### Development Mode
```bash
# Terminal 1 — Backend
cd apps/api && cargo run

# Terminal 2 — Frontend
cd apps/web && npm install && npm run dev
```

---

## ✨ Features

### Issue Reporting
- Report via web form with photos and location
- Works on mobile and desktop
- Anonymous reporting supported

### AI Analysis (Privacy-First)
- Runs directly in the user's browser via WebAssembly
- No data sent to external AI services
- Helps suggest category and urgency

### Routing & Tracking
- Issues routed to appropriate responders
- Status updates visible to everyone
- Historical record of all actions

### Transparency
- Public dashboard showing all issues
- Open data API for researchers
- Community impact metrics

---

## 🗺️ Roadmap

### Phase 1: Foundation
- [x] Project structure
- [x] Basic API and frontend
- [x] Docker setup
- [ ] Database integration
- [ ] User authentication
- [ ] Full issue lifecycle

### Phase 2: Intelligence
- [ ] Browser-based AI classification
- [ ] Duplicate detection
- [ ] Priority suggestions
- [ ] Real-time updates

### Phase 3: Community
- [ ] Mobile PWA
- [ ] LINE integration (Thailand)
- [ ] Multi-language support
- [ ] Public API

### Phase 4: Ecosystem
- [ ] Plugin system
- [ ] Third-party integrations
- [ ] White-label option

---

## 🤝 Contributing

This project is built by the community, for the community. Everyone is welcome.

### How to Contribute
1. 🍴 Fork the repository
2. 🌿 Create a branch from `develop`
3. 💻 Make your changes
4. 📤 Submit a Pull Request

See [CONTRIBUTING.md](.github/CONTRIBUTING.md) for details.

### Code of Conduct
Be respectful. Be helpful. Assume good intent.

---

## 📜 License

Dual-licensed under:
- [MIT License](LICENSE) — use freely
- [Apache-2.0 License](LICENSE-APACHE) — patent protection

**Free for everyone. Forever.**

---

## 🙏 Thanks

- The Rust, React, and open source communities
- Everyone who reports issues, contributes code, or shares feedback
- Communities around the world working to make things better

---

## 🔗 Links

- **GitHub**: [@FirstPrinciples-Sun](https://github.com/FirstPrinciples-Sun)
- **Issues**: [Report a bug or request a feature](../../issues)
- **Discussions**: [Join the conversation](../../discussions)

---

> *"Small actions, multiplied by many people, transform communities."*

**Built with care by FirstPrinciples-Sun and contributors.**
