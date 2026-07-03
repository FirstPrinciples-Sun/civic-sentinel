# Civic Sentinel Developer Guide

Welcome to the Civic Sentinel development guide! This document will help you set up your development environment, understand the codebase, and contribute effectively.

**Target Audience:** New contributors and developers joining the project.

---

## Table of Contents

1. [Getting Started](#getting-started)
2. [Project Structure](#project-structure)
3. [Development Workflow](#development-workflow)
4. [Common Tasks](#common-tasks)
5. [Testing](#testing)
6. [Debugging](#debugging)
7. [Troubleshooting](#troubleshooting)
8. [Best Practices](#best-practices)
9. [Resources](#resources)

---

## Getting Started

### Prerequisites

Before you begin, ensure you have the following installed:

**Required:**
- **Rust** 1.75 or later: [Install Rustup](https://rustup.rs/)
- **Node.js** 20 or later: [Install Node.js](https://nodejs.org/)
- **Git**: [Install Git](https://git-scm.com/)

**Optional:**
- **Docker**: [Install Docker](https://docker.com/) (for container deployment)
- **VS Code**: Recommended editor with Rust Analyzer and ESLint extensions

### Initial Setup

1. **Clone the repository:**
   ```bash
   git clone https://github.com/FirstPrinciples-Sun/civic-sentinel.git
   cd civic-sentinel
   ```

2. **Set up environment variables:**
   ```bash
   cp .env.example .env
   ```

3. **Edit `.env` and set a strong JWT secret:**
   ```bash
   # Generate a random secret
   openssl rand -base64 32
   
   # Or use this Node.js one-liner
   node -e "console.log(require('crypto').randomBytes(32).toString('base64'))"
   ```
   
   Paste the output as your `JWT_SECRET` in `.env`:
   ```env
   JWT_SECRET=YOUR_GENERATED_SECRET_HERE
   ```

4. **Install dependencies:**
   ```bash
   # Backend (Rust)
   cd apps/api
   cargo build
   
   # Frontend (Node.js)
   cd ../web
   npm install
   ```

5. **Run database migrations:**
   ```bash
   # Migrations run automatically on first server start
   # The database file will be created at: data/civic-sentinel.db
   ```

6. **Start development servers:**
   
   **Terminal 1 - Backend:**
   ```bash
   cd apps/api
   cargo run
   # API runs on http://localhost:3000
   ```
   
   **Terminal 2 - Frontend:**
   ```bash
   cd apps/web
   npm run dev
   # Frontend runs on http://localhost:5173
   ```

7. **Open your browser:**
   - Frontend: http://localhost:5173
   - API health check: http://localhost:3000/health
   - API info: http://localhost:3000/api/v1/info

---

## Project Structure

```
civic-sentinel/
├── apps/
│   ├── api/                    # Rust backend (Axum)
│   │   ├── src/
│   │   │   ├── main.rs         # Entry point
│   │   │   ├── lib.rs          # Library exports
│   │   │   ├── config.rs       # Configuration
│   │   │   ├── error.rs        # Error types
│   │   │   ├── db/             # Database layer
│   │   │   ├── middleware/     # HTTP middleware
│   │   │   ├── models/         # Domain models
│   │   │   ├── routes/         # API endpoints
│   │   │   └── services/       # Business logic
│   │   ├── Cargo.toml          # Rust dependencies
│   │   └── data/               # SQLite database (gitignored)
│   │
│   └── web/                    # React frontend
│       ├── src/
│       │   ├── main.tsx        # Entry point
│       │   ├── App.tsx         # Root component
│       │   ├── components/     # Reusable UI components
│       │   ├── context/        # React contexts (Auth, Language)
│       │   ├── hooks/          # Custom React hooks
│       │   ├── pages/          # Page components (routes)
│       │   ├── services/       # API client (Axios)
│       │   └── i18n/           # Translations
│       ├── package.json        # Node dependencies
│       └── public/             # Static assets
│
├── crates/
│   └── wasm-analytics/         # WebAssembly module (planned)
│
├── docs/                       # Documentation
├── .github/                    # GitHub workflows and templates
├── .env.example                # Environment template
├── docker-compose.yml          # Docker deployment config
├── Dockerfile                  # Container build
├── README.md                   # Project overview
├── SECURITY.md                 # Security policy
├── ARCHITECTURE.md             # System design
└── CODE_REVIEW_CHECKLIST.md   # Review guidelines
```

---

## Development Workflow

### Branching Strategy

We use **Git Flow** with two main branches:
- `main` — Production-ready code
- `develop` — Integration branch for next release

**Creating a feature branch:**
```bash
git checkout develop
git pull origin develop
git checkout -b feature/your-feature-name
```

**Creating a bugfix branch:**
```bash
git checkout develop
git pull origin develop
git checkout -b fix/bug-description
```

### Making Changes

1. **Write your code** following the style guide (see Best Practices)
2. **Format your code:**
   ```bash
   # Rust
   cd apps/api
   cargo fmt
   
   # TypeScript
   cd apps/web
   npm run lint -- --fix
   ```

3. **Run tests:**
   ```bash
   # Rust
   cargo test --workspace
   
   # Frontend (when tests are added)
   npm run test
   ```

4. **Check for errors:**
   ```bash
   # Rust linting
   cargo clippy -- -D warnings
   
   # TypeScript type checking
   npm run typecheck
   ```

5. **Commit your changes:**
   ```bash
   git add .
   git commit -m "feat(issues): add duplicate detection algorithm"
   ```
   
   Follow **Conventional Commits** format:
   - `feat(scope): description` — New feature
   - `fix(scope): description` — Bug fix
   - `docs(scope): description` — Documentation
   - `style(scope): description` — Formatting (no code change)
   - `refactor(scope): description` — Code restructuring
   - `test(scope): description` — Adding tests
   - `chore(scope): description` — Build/tooling changes

6. **Push your branch:**
   ```bash
   git push origin feature/your-feature-name
   ```

7. **Create a Pull Request:**
   - Go to GitHub
   - Click "New Pull Request"
   - Target: `develop` (not `main`)
   - Fill in the template
   - Request review

### Code Review Process

1. **Automated checks** run via GitHub Actions (build, lint, test)
2. **Reviewer provides feedback** (usually within 2-3 days)
3. **Address comments** and push updates
4. **Approval** required before merge
5. **Squash and merge** into `develop`

---

## Common Tasks

### Adding a New API Endpoint

**Example: Add `/api/v1/issues/:id/attachments` endpoint**

1. **Define the model** in `apps/api/src/models/mod.rs`:
   ```rust
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct Attachment {
       pub id: Uuid,
       pub issue_id: Uuid,
       pub filename: String,
       pub url: String,
       pub created_at: DateTime<Utc>,
   }
   ```

2. **Add database methods** in `apps/api/src/db/mod.rs`:
   ```rust
   pub async fn list_attachments(&self, issue_id: Uuid) -> Result<Vec<Attachment>, Error> {
       // Query implementation
   }
   ```

3. **Create route handler** in `apps/api/src/routes/attachments.rs`:
   ```rust
   pub async fn get_attachments(
       State(state): State<AppState>,
       Path(issue_id): Path<Uuid>,
   ) -> Result<Json<Value>, StatusCode> {
       let attachments = state.db.list_attachments(issue_id).await
           .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
       
       Ok(Json(json!({
           "success": true,
           "data": attachments
       })))
   }
   ```

4. **Register route** in `apps/api/src/main.rs`:
   ```rust
   .route("/api/v1/issues/:id/attachments", get(routes::attachments::get_attachments))
   ```

5. **Test the endpoint:**
   ```bash
   curl http://localhost:3000/api/v1/issues/{issue_id}/attachments
   ```

### Adding a New Frontend Page

**Example: Add a "Settings" page**

1. **Create page component** in `apps/web/src/pages/SettingsPage.tsx`:
   ```tsx
   export function SettingsPage() {
     return (
       <div>
         <h1>Settings</h1>
         {/* Your component code */}
       </div>
     );
   }
   ```

2. **Add route** in `apps/web/src/App.tsx`:
   ```tsx
   import { SettingsPage } from './pages/SettingsPage';
   
   // Inside your Routes component:
   <Route path="/settings" element={<SettingsPage />} />
   ```

3. **Add navigation link** in `apps/web/src/components/Layout.tsx`:
   ```tsx
   <Link to="/settings">Settings</Link>
   ```

### Adding a Database Migration

Since migrations are embedded in code (not separate SQL files), add new tables or columns in `apps/api/src/db/mod.rs`:

```rust
// In the migrate() method:

// Create new table
conn.execute(
    "CREATE TABLE IF NOT EXISTS attachments (
        id TEXT PRIMARY KEY,
        issue_id TEXT NOT NULL,
        filename TEXT NOT NULL,
        url TEXT NOT NULL,
        created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
        FOREIGN KEY (issue_id) REFERENCES issues(id) ON DELETE CASCADE
    )",
    (),
).await?;

// Add new column to existing table
Self::ensure_table_column(
    &conn,
    "users",
    "preferred_language",
    "ALTER TABLE users ADD COLUMN preferred_language TEXT DEFAULT 'en'",
).await?;
```

**Note:** Always use `ensure_table_column()` for adding columns to avoid migration errors on existing databases.

### Running Database Queries Manually

Since we use SQLite, you can query the database directly:

```bash
# Install sqlite3 CLI (if not already installed)
# Ubuntu/Debian: apt install sqlite3
# macOS: brew install sqlite3
# Windows: Download from https://www.sqlite.org/download.html

# Open the database
sqlite3 apps/api/data/civic-sentinel.db

# Run queries
sqlite> SELECT * FROM users LIMIT 10;
sqlite> .tables                    # List all tables
sqlite> .schema issues              # Show table schema
sqlite> .quit                       # Exit
```

### Seeding Test Data

Create a script `apps/api/scripts/seed.sh`:

```bash
#!/bin/bash
# Seed test data for development

sqlite3 apps/api/data/civic-sentinel.db <<EOF
-- Insert test user (password: "password123")
INSERT INTO users (id, email, password_hash, name, role, created_at, updated_at)
VALUES (
  '123e4567-e89b-12d3-a456-426614174000',
  'admin@test.com',
  '\$argon2id\$v=19\$m=65536,t=3,p=4\$...',  -- Hash for "password123"
  'Test Admin',
  'admin',
  datetime('now'),
  datetime('now')
);

-- Insert test issue
INSERT INTO issues (
  id, title, description, category, priority, status,
  latitude, longitude, reporter_id, created_at, updated_at
) VALUES (
  '223e4567-e89b-12d3-a456-426614174000',
  'Broken streetlight on Main St',
  'The streetlight at the corner of Main St and 1st Ave has been out for 3 days.',
  'infrastructure',
  'medium',
  'reported',
  13.7563,
  100.5018,
  '123e4567-e89b-12d3-a456-426614174000',
  datetime('now'),
  datetime('now')
);
EOF

echo "✅ Test data seeded successfully"
```

**Note:** For password hashing, use a proper Argon2 generator or create a user via the `/auth/register` endpoint.

---

## Testing

### Backend Testing (Rust)

**Run all tests:**
```bash
cd apps/api
cargo test --workspace
```

**Run specific test:**
```bash
cargo test test_create_and_get_issue
```

**Run tests with output:**
```bash
cargo test -- --nocapture
```

**Writing a test:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_list_issues_filters_by_status() {
        let db = setup_test_db().await;
        
        // Create test issues with different statuses
        let reported = create_test_issue(&db, IssueStatus::Reported).await;
        let resolved = create_test_issue(&db, IssueStatus::Resolved).await;
        
        // Query with status filter
        let query = IssueListQuery {
            status: Some(IssueStatus::Reported),
            ..Default::default()
        };
        let (issues, total) = db.list_issues(&query).await.unwrap();
        
        // Assert
        assert_eq!(total, 1);
        assert_eq!(issues[0].id, reported.id);
    }
}
```

### Frontend Testing (React)

**Run tests (when implemented):**
```bash
cd apps/web
npm run test
```

**Writing a component test with Vitest:**
```tsx
import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { LoginPage } from './LoginPage';

describe('LoginPage', () => {
  it('renders login form', () => {
    render(<LoginPage />);
    expect(screen.getByLabelText(/email/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/password/i)).toBeInTheDocument();
  });

  it('shows error on invalid credentials', async () => {
    // Test implementation
  });
});
```

---

## Debugging

### Backend Debugging (Rust)

**Add logging:**
```rust
use tracing::{info, warn, error, debug};

info!("Issue created: {:?}", issue.id);
warn!("Rate limit exceeded for IP: {}", ip);
error!("Database query failed: {}", err);
debug!("Request payload: {:?}", request);
```

**Set log level:**
```bash
# .env file
RUST_LOG=debug

# Or set via environment:
RUST_LOG=debug cargo run
```

**Using rust-lldb (macOS/Linux):**
```bash
rust-lldb target/debug/civic-sentinel-api
(lldb) b main.rs:65     # Set breakpoint
(lldb) run              # Start program
(lldb) continue         # Continue execution
(lldb) print variable   # Inspect variable
```

**Using VS Code debugger:**
1. Install "rust-analyzer" extension
2. Add `.vscode/launch.json`:
   ```json
   {
     "version": "0.2.0",
     "configurations": [
       {
         "type": "lldb",
         "request": "launch",
         "name": "Debug API",
         "cargo": {
           "args": ["build", "--bin=civic-sentinel-api"]
         },
         "args": [],
         "cwd": "${workspaceFolder}/apps/api"
       }
     ]
   }
   ```
3. Set breakpoints and press F5

### Frontend Debugging (React)

**Browser DevTools:**
- **Console:** View logs and errors
- **Network tab:** Inspect API requests/responses
- **React DevTools:** Inspect component tree and state

**Adding debug logs:**
```tsx
console.log('User data:', user);
console.warn('Token expiring soon');
console.error('API request failed:', error);
```

**VS Code debugger:**
1. Install "Debugger for Chrome" extension
2. Add `.vscode/launch.json`:
   ```json
   {
     "version": "0.2.0",
     "configurations": [
       {
         "type": "chrome",
         "request": "launch",
         "name": "Debug Frontend",
         "url": "http://localhost:5173",
         "webRoot": "${workspaceFolder}/apps/web/src"
       }
     ]
   }
   ```
3. Start dev server, set breakpoints, press F5

---

## Troubleshooting

### Common Issues

**1. "JWT_SECRET is too short" error**
- **Cause:** Default JWT_SECRET in `.env` is weak
- **Fix:** Generate a strong secret (see Initial Setup step 3)

**2. "Database locked" error**
- **Cause:** Multiple processes accessing SQLite simultaneously
- **Fix:** Ensure only one API server is running

**3. "Port 3000 already in use"**
- **Cause:** Another process using port 3000
- **Fix:** 
  ```bash
  # Find process
  lsof -i :3000      # macOS/Linux
  netstat -ano | findstr :3000  # Windows
  
  # Kill process or change port in .env:
  SERVER_PORT=3001
  ```

**4. CORS errors in browser**
- **Cause:** Frontend running on different origin than API expects
- **Fix:** Add frontend URL to `CORS_ORIGINS` in `.env`:
  ```env
  CORS_ORIGINS=http://localhost:5173,http://localhost:3000
  ```

**5. "Failed to fetch" errors**
- **Cause:** API server not running or wrong URL
- **Fix:** 
  1. Check API is running: `curl http://localhost:3000/health`
  2. Verify `VITE_API_URL` in `.env`:
     ```env
     VITE_API_URL=http://localhost:3000
     ```

**6. Rust compilation errors on Windows**
- **Cause:** Missing Visual Studio Build Tools
- **Fix:** Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022)
  - Select "Desktop development with C++"

**7. "Cannot find module" errors in frontend**
- **Cause:** Missing dependencies
- **Fix:** 
  ```bash
  cd apps/web
  rm -rf node_modules package-lock.json
  npm install
  ```

---

## Best Practices

### Rust (Backend)

**Error Handling:**
```rust
// ❌ Bad: Using unwrap()
let user = db.get_user(id).await.unwrap();

// ✅ Good: Propagate error
let user = db.get_user(id).await?;

// ✅ Good: Handle error explicitly
let user = match db.get_user(id).await {
    Ok(u) => u,
    Err(e) => {
        tracing::error!("Failed to fetch user: {}", e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
};
```

**Async Best Practices:**
```rust
// ❌ Bad: Blocking operation in async function
async fn slow_handler() {
    std::thread::sleep(Duration::from_secs(5));  // Blocks thread!
}

// ✅ Good: Use async sleep
async fn good_handler() {
    tokio::time::sleep(Duration::from_secs(5)).await;
}
```

**Database Queries:**
```rust
// ❌ Bad: N+1 queries
for issue in issues {
    let comments = db.get_comments(issue.id).await?;  // Query in loop!
}

// ✅ Good: Batch load
let issue_ids: Vec<Uuid> = issues.iter().map(|i| i.id).collect();
let all_comments = db.get_comments_batch(&issue_ids).await?;
```

### TypeScript (Frontend)

**Type Safety:**
```tsx
// ❌ Bad: Using 'any'
function formatIssue(issue: any) {
  return issue.title;  // No autocomplete or type checking
}

// ✅ Good: Use proper types
function formatIssue(issue: Issue) {
  return issue.title;  // Type-safe!
}
```

**React Hooks:**
```tsx
// ❌ Bad: Missing dependencies
useEffect(() => {
  fetchIssues(statusFilter);
}, []);  // statusFilter not in deps!

// ✅ Good: Include all dependencies
useEffect(() => {
  fetchIssues(statusFilter);
}, [statusFilter]);
```

**Error Handling:**
```tsx
// ❌ Bad: Unhandled promise rejection
const handleSubmit = async () => {
  await api.post('/issues', data);  // Error not caught!
};

// ✅ Good: Try-catch
const handleSubmit = async () => {
  try {
    await api.post('/issues', data);
    toast.success('Issue created!');
  } catch (error) {
    toast.error('Failed to create issue');
    console.error(error);
  }
};
```

### Security

**Input Validation:**
```rust
// ✅ Always validate user input
#[derive(Deserialize, Validate)]
struct CreateIssueRequest {
    #[validate(length(min = 3, max = 200))]
    title: String,
    
    #[validate(length(min = 10, max = 5000))]
    description: String,
}
```

**SQL Safety:**
```rust
// ❌ Bad: String concatenation (SQL injection risk)
let sql = format!("SELECT * FROM users WHERE email = '{}'", email);

// ✅ Good: Parameterized query
conn.query("SELECT * FROM users WHERE email = ?1", [email]).await?;
```

**Authentication:**
```rust
// ✅ Always check auth on protected routes
pub async fn delete_issue(
    Extension(user_id): Extension<Uuid>,  // JWT middleware extracts this
    State(state): State<AppState>,
    Path(issue_id): Path<Uuid>,
) -> Result<Json<Value>, StatusCode> {
    // Verify user has permission
    // Then proceed
}
```

---

## Resources

### Documentation
- [Rust Book](https://doc.rust-lang.org/book/)
- [Axum Documentation](https://docs.rs/axum/)
- [React Documentation](https://react.dev/)
- [SQLite Documentation](https://www.sqlite.org/docs.html)

### Tools
- [Rust Playground](https://play.rust-lang.org/) — Test Rust code online
- [TypeScript Playground](https://www.typescriptlang.org/play) — Test TS code online
- [Postman](https://www.postman.com/) — API testing
- [DB Browser for SQLite](https://sqlitebrowser.org/) — Visual database browser

### Community
- [GitHub Issues](https://github.com/FirstPrinciples-Sun/civic-sentinel/issues) — Bug reports and feature requests
- [GitHub Discussions](https://github.com/FirstPrinciples-Sun/civic-sentinel/discussions) — Questions and ideas

---

## Getting Help

**Questions about:**
- **Code structure:** Check [ARCHITECTURE.md](./ARCHITECTURE.md)
- **Security:** Check [SECURITY.md](./SECURITY.md)
- **Code review:** Check [CODE_REVIEW_CHECKLIST.md](./CODE_REVIEW_CHECKLIST.md)

**Still stuck?**
- Open a [GitHub Discussion](https://github.com/FirstPrinciples-Sun/civic-sentinel/discussions)
- Create an [Issue](https://github.com/FirstPrinciples-Sun/civic-sentinel/issues)

---

**Happy coding! 🚀**

---

**Maintained by:** [@FirstPrinciples-Sun](https://github.com/FirstPrinciples-Sun)  
**Last Updated:** July 3, 2026
