use libsql::Builder;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::config::AppConfig;
use crate::models::*;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Database connection manager
#[derive(Clone)]
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
                deleted_at DATETIME,
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

    /// Insert a new issue
    pub async fn insert_issue(&self, issue: &Issue) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.client.lock().await;
        conn.execute(
            "INSERT INTO issues (id, title, description, category, priority, status, latitude, longitude, address, reporter_id, assigned_to, media_urls, tags, created_at, updated_at, resolved_at, deleted_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
            (
                issue.id.to_string(),
                &issue.title,
                &issue.description,
                serde_json::to_string(&issue.category)?,
                serde_json::to_string(&issue.priority)?,
                serde_json::to_string(&issue.status)?,
                issue.location.latitude,
                issue.location.longitude,
                issue.location.address.as_ref().map(|s| s.as_str()),
                issue.reporter_id.map(|id| id.to_string()),
                issue.assigned_to.map(|id| id.to_string()),
                serde_json::to_string(&issue.media_urls)?,
                serde_json::to_string(&issue.tags)?,
                issue.created_at.to_rfc3339(),
                issue.updated_at.to_rfc3339(),
                issue.resolved_at.map(|dt| dt.to_rfc3339()),
                issue.deleted_at.map(|dt| dt.to_rfc3339()),
            ),
        ).await?;
        Ok(())
    }

    /// Fetch an issue by ID
    pub async fn get_issue(&self, id: Uuid) -> Result<Option<Issue>, Box<dyn std::error::Error>> {
        let conn = self.client.lock().await;
        let mut rows = conn.query(
            "SELECT id, title, description, category, priority, status, latitude, longitude, address, reporter_id, assigned_to, media_urls, tags, created_at, updated_at, resolved_at, deleted_at
             FROM issues WHERE id = ?1 AND deleted_at IS NULL",
            [id.to_string()],
        ).await?;

        if let Some(row) = rows.next().await? {
            Ok(Some(self.row_to_issue(&row)?))
        } else {
            Ok(None)
        }
    }

    /// Update an issue
    pub async fn update_issue(&self, id: Uuid, updates: &UpdateIssueRequest) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.client.lock().await;
        let mut sets = vec![];
        let mut params: Vec<libsql::Value> = vec![];

        if let Some(title) = &updates.title {
            sets.push("title = ?".to_string());
            params.push(title.clone().into());
        }
        if let Some(description) = &updates.description {
            sets.push("description = ?".to_string());
            params.push(description.clone().into());
        }
        if let Some(status) = &updates.status {
            sets.push("status = ?".to_string());
            params.push(serde_json::to_string(status)?.into());
        }
        if let Some(priority) = &updates.priority {
            sets.push("priority = ?".to_string());
            params.push(serde_json::to_string(priority)?.into());
        }
        if let Some(assigned_to) = &updates.assigned_to {
            sets.push("assigned_to = ?".to_string());
            params.push(assigned_to.to_string().into());
        }

        sets.push("updated_at = ?".to_string());
        params.push(chrono::Utc::now().to_rfc3339().into());

        if sets.is_empty() {
            return Ok(());
        }

        let sql = format!("UPDATE issues SET {} WHERE id = ? AND deleted_at IS NULL", sets.join(", "));
        params.push(id.to_string().into());

        conn.execute(&sql, libsql::params_from_iter(params)).await?;
        Ok(())
    }

    /// Soft delete an issue
    pub async fn delete_issue(&self, id: Uuid) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.client.lock().await;
        conn.execute(
            "UPDATE issues SET deleted_at = ?1, updated_at = ?2 WHERE id = ?3 AND deleted_at IS NULL",
            [chrono::Utc::now().to_rfc3339(), chrono::Utc::now().to_rfc3339(), id.to_string()],
        ).await?;
        Ok(())
    }

    /// List issues with optional filters
    pub async fn list_issues(&self, query: &IssueListQuery) -> Result<(Vec<Issue>, i64), Box<dyn std::error::Error>> {
        let conn = self.client.lock().await;
        let mut where_clauses = vec!["deleted_at IS NULL".to_string()];
        let mut params: Vec<libsql::Value> = vec![];

        if let Some(status) = &query.status {
            where_clauses.push("status = ?".to_string());
            params.push(serde_json::to_string(status)?.into());
        }
        if let Some(category) = &query.category {
            where_clauses.push("category = ?".to_string());
            params.push(serde_json::to_string(category)?.into());
        }
        if let Some(priority) = &query.priority {
            where_clauses.push("priority = ?".to_string());
            params.push(serde_json::to_string(priority)?.into());
        }

        let sort = match query.sort.as_deref() {
            Some("updated_at") => "updated_at DESC",
            Some("priority") => "priority DESC",
            _ => "created_at DESC",
        };

        let page = query.page.unwrap_or(1).max(1);
        let limit = query.limit.unwrap_or(20).max(1).min(100);
        let offset = (page - 1) * limit;

        let count_sql = format!("SELECT COUNT(*) FROM issues WHERE {}", where_clauses.join(" AND "));
        let mut count_rows = conn.query(&count_sql, libsql::params_from_iter(params.clone())).await?;
        let total = if let Some(row) = count_rows.next().await? {
            row.get::<i64>(0)?
        } else {
            0
        };

        let sql = format!(
            "SELECT id, title, description, category, priority, status, latitude, longitude, address, reporter_id, assigned_to, media_urls, tags, created_at, updated_at, resolved_at, deleted_at
             FROM issues WHERE {} ORDER BY {} LIMIT ? OFFSET ?",
            where_clauses.join(" AND "),
            sort
        );
        params.push((limit as i64).into());
        params.push((offset as i64).into());

        let mut rows = conn.query(&sql, libsql::params_from_iter(params)).await?;
        let mut issues = vec![];
        while let Some(row) = rows.next().await? {
            issues.push(self.row_to_issue(&row)?);
        }

        Ok((issues, total))
    }

    /// Insert a comment
    pub async fn insert_comment(&self, comment: &Comment) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.client.lock().await;
        conn.execute(
            "INSERT INTO issue_comments (id, issue_id, author_id, content, is_internal, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            [
                comment.id.to_string(),
                comment.issue_id.to_string(),
                comment.author_id.to_string(),
                comment.content.clone(),
                if comment.is_internal { "1" } else { "0" }.to_string(),
                comment.created_at.to_rfc3339(),
            ],
        ).await?;
        Ok(())
    }

    /// Get comments for an issue
    pub async fn get_comments(&self, issue_id: Uuid) -> Result<Vec<Comment>, Box<dyn std::error::Error>> {
        let conn = self.client.lock().await;
        let mut rows = conn.query(
            "SELECT id, issue_id, author_id, content, is_internal, created_at
             FROM issue_comments WHERE issue_id = ?1 ORDER BY created_at DESC",
            [issue_id.to_string()],
        ).await?;

        let mut comments = vec![];
        while let Some(row) = rows.next().await? {
            comments.push(self.row_to_comment(&row)?);
        }
        Ok(comments)
    }

    /// Insert a status history entry
    pub async fn insert_status_history(&self, entry: &StatusHistoryEntry) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.client.lock().await;
        conn.execute(
            "INSERT INTO issue_status_history (id, issue_id, old_status, new_status, changed_by, reason, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            [
                entry.id.to_string(),
                entry.issue_id.to_string(),
                serde_json::to_string(&entry.old_status)?,
                serde_json::to_string(&entry.new_status)?,
                entry.changed_by.to_string(),
                entry.reason.clone().unwrap_or_default(),
                entry.created_at.to_rfc3339(),
            ],
        ).await?;
        Ok(())
    }

    /// Get status history for an issue
    pub async fn get_status_history(&self, issue_id: Uuid) -> Result<Vec<StatusHistoryEntry>, Box<dyn std::error::Error>> {
        let conn = self.client.lock().await;
        let mut rows = conn.query(
            "SELECT id, issue_id, old_status, new_status, changed_by, reason, created_at
             FROM issue_status_history WHERE issue_id = ?1 ORDER BY created_at DESC",
            [issue_id.to_string()],
        ).await?;

        let mut history = vec![];
        while let Some(row) = rows.next().await? {
            history.push(self.row_to_status_history(&row)?);
        }
        Ok(history)
    }

    fn row_to_issue(&self, row: &libsql::Row) -> Result<Issue, Box<dyn std::error::Error>> {
        Ok(Issue {
            id: row.get::<String>(0)?.parse()?,
            title: row.get::<String>(1)?,
            description: row.get::<String>(2)?,
            category: serde_json::from_str(&row.get::<String>(3)?)?,
            priority: serde_json::from_str(&row.get::<String>(4)?)?,
            status: serde_json::from_str(&row.get::<String>(5)?)?,
            location: GeoLocation {
                latitude: row.get::<f64>(6)?,
                longitude: row.get::<f64>(7)?,
                address: row.get::<Option<String>>(8)?,
            },
            reporter_id: row.get::<Option<String>>(9)?.map(|s| s.parse()).transpose()?,
            assigned_to: row.get::<Option<String>>(10)?.map(|s| s.parse()).transpose()?,
            media_urls: serde_json::from_str(&row.get::<String>(11)?)?,
            tags: serde_json::from_str(&row.get::<String>(12)?)?,
            created_at: row.get::<String>(13)?.parse::<DateTime<Utc>>()?,
            updated_at: row.get::<String>(14)?.parse::<DateTime<Utc>>()?,
            resolved_at: row.get::<Option<String>>(15)?.map(|s| s.parse()).transpose()?,
            deleted_at: row.get::<Option<String>>(16)?.map(|s| s.parse()).transpose()?,
        })
    }

    fn row_to_comment(&self, row: &libsql::Row) -> Result<Comment, Box<dyn std::error::Error>> {
        Ok(Comment {
            id: row.get::<String>(0)?.parse()?,
            issue_id: row.get::<String>(1)?.parse()?,
            author_id: row.get::<String>(2)?.parse()?,
            content: row.get::<String>(3)?,
            is_internal: row.get::<i32>(4)? != 0,
            created_at: row.get::<String>(5)?.parse::<DateTime<Utc>>()?,
        })
    }

    fn row_to_status_history(&self, row: &libsql::Row) -> Result<StatusHistoryEntry, Box<dyn std::error::Error>> {
        Ok(StatusHistoryEntry {
            id: row.get::<String>(0)?.parse()?,
            issue_id: row.get::<String>(1)?.parse()?,
            old_status: serde_json::from_str(&row.get::<String>(2)?)?,
            new_status: serde_json::from_str(&row.get::<String>(3)?)?,
            changed_by: row.get::<String>(4)?.parse()?,
            reason: row.get::<Option<String>>(5)?,
            created_at: row.get::<String>(6)?.parse::<DateTime<Utc>>()?,
        })
    }
}
