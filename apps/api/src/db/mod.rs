use libsql::Builder;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::config::AppConfig;

/// Database connection manager
pub struct Database {
    client: Arc<Mutex<libsql::Connection>>,
}

impl Database {
    /// Initialize database connection
    pub async fn new(config: &AppConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let db_path = &config.database.url;
        
        let db = if db_path.starts_with("http://") || db_path.starts_with("https://") {
            // Remote Turso database
            let token = std::env::var("TURSO_AUTH_TOKEN")
                .map_err(|_| "TURSO_AUTH_TOKEN not set for remote database")?;
            
            Builder::new_remote(db_path.clone(), token)
                .build()
                .await?
        } else {
            // Local SQLite database
            let path = db_path.trim_start_matches("sqlite://");
            Builder::new_local(path).build().await?
        };

        let conn = db.connect()?;
        
        Ok(Self {
            client: Arc::new(Mutex::new(conn)),
        })
    }

    /// Run database migrations
    pub async fn migrate(&self) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.client.lock().await;
        
        // Create users table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                email TEXT UNIQUE NOT NULL,
                password_hash TEXT NOT NULL,
                name TEXT,
                phone TEXT,
                role TEXT NOT NULL DEFAULT 'reporter',
                avatar_url TEXT,
                email_verified BOOLEAN DEFAULT FALSE,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            (),
        ).await?;

        // Create issues table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS issues (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                description TEXT NOT NULL,
                category TEXT NOT NULL,
                priority TEXT NOT NULL DEFAULT 'medium',
                status TEXT NOT NULL DEFAULT 'reported',
                latitude REAL NOT NULL,
                longitude REAL NOT NULL,
                address TEXT,
                reporter_id TEXT,
                assigned_to TEXT,
                media_urls TEXT,
                tags TEXT,
                ai_priority_score INTEGER,
                ai_category TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                resolved_at DATETIME,
                FOREIGN KEY (reporter_id) REFERENCES users(id),
                FOREIGN KEY (assigned_to) REFERENCES users(id)
            )",
            (),
        ).await?;

        // Create comments table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS issue_comments (
                id TEXT PRIMARY KEY,
                issue_id TEXT NOT NULL,
                author_id TEXT NOT NULL,
                content TEXT NOT NULL,
                is_internal BOOLEAN DEFAULT FALSE,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (issue_id) REFERENCES issues(id) ON DELETE CASCADE,
                FOREIGN KEY (author_id) REFERENCES users(id)
            )",
            (),
        ).await?;

        // Create status history table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS issue_status_history (
                id TEXT PRIMARY KEY,
                issue_id TEXT NOT NULL,
                old_status TEXT NOT NULL,
                new_status TEXT NOT NULL,
                changed_by TEXT NOT NULL,
                reason TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (issue_id) REFERENCES issues(id) ON DELETE CASCADE,
                FOREIGN KEY (changed_by) REFERENCES users(id)
            )",
            (),
        ).await?;

        // Create refresh tokens table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS refresh_tokens (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                token_hash TEXT NOT NULL,
                expires_at DATETIME NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                revoked_at DATETIME,
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
            )",
            (),
        ).await?;

        // Create indexes
        conn.execute("CREATE INDEX IF NOT EXISTS idx_issues_status ON issues(status)", ()).await?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_issues_category ON issues(category)", ()).await?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_issues_location ON issues(latitude, longitude)", ()).await?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_issues_created ON issues(created_at)", ()).await?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_users_email ON users(email)", ()).await?;

        // Create FTS (Full Text Search) virtual table
        conn.execute(
            "CREATE VIRTUAL TABLE IF NOT EXISTS issues_fts USING fts5(
                title, description,
                content='issues',
                content_rowid='rowid'
            )",
            (),
        ).await?;

        Ok(())
    }

    /// Get database connection
    pub async fn conn(&self) -> tokio::sync::MutexGuard<'_, libsql::Connection> {
        self.client.lock().await
    }
}
