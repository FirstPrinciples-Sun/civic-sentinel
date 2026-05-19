# Git Branch Strategy

## Branches

- **main** — Production ready, protected
- **develop** — Integration branch, protected
- **feature/*** — New features, merge to develop
- **bugfix/*** — Bug fixes, merge to develop
- **release/*** — Release preparation
- **hotfix/*** — Critical production fixes

## Commit Convention

```
type(scope): description

Types: feat, fix, docs, style, refactor, test, chore, security
```

Examples:
```
feat(auth): add login page
fix(api): resolve duplicate issue detection
docs(readme): update deployment guide
```

## Pull Requests

- All changes via PR
- Target: `develop` for features, `main` for hotfixes
- CI must pass
- At least 1 review for docs, 2 for features

## Getting Started

```bash
# Fork, then clone
git clone https://github.com/YOUR_USERNAME/civic-sentinel.git
cd civic-sentinel
git checkout develop
git checkout -b feature/my-feature
# Make changes, commit, push, open PR
```
