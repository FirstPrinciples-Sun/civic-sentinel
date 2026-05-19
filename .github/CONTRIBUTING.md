# Contributing to Civic Sentinel

First off, thank you for considering contributing to Civic Sentinel! 🎉

## How Can I Contribute?

### Reporting Bugs
- Use the [Bug Report template](../../issues/new?template=bug_report.md)
- Include steps to reproduce
- Include your environment details

### Suggesting Features
- Use the [Feature Request template](../../issues/new?template=feature_request.md)
- Explain the community impact
- Be specific about the use case

### Pull Requests
1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Make your changes
4. Add tests
5. Run the test suite
6. Commit with clear messages
7. Push to your fork
8. Open a Pull Request

## Development Setup

### Prerequisites
- Rust 1.75+
- Node.js 20+
- Docker (optional)

### Backend
```bash
cd apps/api
cargo build
cargo test
```

### Frontend
```bash
cd apps/web
npm install
npm run dev
```

### WASM
```bash
cd crates/wasm-analytics
wasm-pack build --target web
```

## Code Style

### Rust
- Follow `cargo fmt` formatting
- Address all `cargo clippy` warnings
- Write documentation for public APIs
- Add tests for new features

### TypeScript/React
- Use ESLint configuration
- Write unit tests for utilities
- Document complex components

## Commit Messages
We follow [Conventional Commits](https://www.conventionalcommits.org/):
- `feat:` New feature
- `fix:` Bug fix
- `docs:` Documentation changes
- `refactor:` Code refactoring
- `test:` Adding tests
- `chore:` Maintenance tasks

## Questions?
- Open a [Discussion](../../discussions)
- Email: rtchanaphon@gmail.com

Thank you for making Civic Sentinel better! 🛡️
