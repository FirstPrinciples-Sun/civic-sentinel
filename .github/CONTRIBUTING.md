# Contributing

Thank you for considering contributing to Civic Sentinel.

## How to Contribute

### Reporting Bugs
- Use the [Bug Report template](../../issues/new?template=bug_report.md)
- Include steps to reproduce
- Include environment details

### Suggesting Features
- Use the [Feature Request template](../../issues/new?template=feature_request.md)
- Explain the community impact

### Pull Requests
1. Fork the repository
2. Create a branch from `develop`
3. Make your changes
4. Add tests
5. Submit a PR

## Development Setup

```bash
# Rust
cd apps/api
cargo build
cargo test

# Frontend
cd apps/web
npm install
npm run dev

# WASM
cd crates/wasm-analytics
wasm-pack build --target web
```

## Code Style

- Rust: `cargo fmt` and `cargo clippy`
- TypeScript: Follow ESLint config
- Commits: Use [Conventional Commits](https://www.conventionalcommits.org/)

## Questions?

- [Discussions](../../discussions)
- Email: rtchanaphon@gmail.com
