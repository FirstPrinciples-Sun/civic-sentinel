# Self-Hosting Guide

Deploy Civic Sentinel on your own infrastructure.

## Quick Start (Docker)

```bash
git clone https://github.com/FirstPrinciples-Sun/civic-sentinel.git
cd civic-sentinel
cp .env.example .env
# Edit .env — set a strong JWT_SECRET
docker-compose up -d
```

Visit http://localhost:5173

## Environment Variables

Copy `.env.example` to `.env`:

```env
# Server
SERVER_HOST=0.0.0.0
SERVER_PORT=3000

# Database
DATABASE_URL=sqlite://data/civic-sentinel.db

# Security (generate: openssl rand -base64 32)
JWT_SECRET=your-secret-here

# CORS
CORS_ORIGINS=http://localhost:5173
```

## Production Options

### Fly.io (Free Tier)
```bash
fly launch
fly secrets set JWT_SECRET=$(openssl rand -base64 32)
fly deploy
```

### Railway
1. Fork repo
2. Connect Railway
3. Add env vars
4. Auto-deploy on push

### VPS
```bash
sudo apt install docker.io docker-compose git
git clone https://github.com/FirstPrinciples-Sun/civic-sentinel.git
cd civic-sentinel
docker-compose up -d
```

## Updating

```bash
git pull origin main
docker-compose down
docker-compose up --build -d
```

## Support

- [Issues](../../issues)
- [Discussions](../../discussions)
