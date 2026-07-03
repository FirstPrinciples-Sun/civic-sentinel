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
    pub async fn new(config: &AppConfig) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let db_path = &config.database.url;

        let db = if db_path.starts_with("http://") || db_path.starts_with("https://") {
            // Remote Turso database
            let token = std::env::var("TURSO_AUTH_TOKEN")
                .map_err(|_| "TURSO_AUTH_TOKEN not set for remote database")?;

            Builder::new_remote(db_path.clone(), token).build().await?
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
    pub async fn migrate(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
                phone_verified BOOLEAN DEFAULT FALSE,
                phone_verified_at DATETIME,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            (),
        )
        .await?;

        Self::ensure_table_column(
            &conn,
            "users",
            "phone_verified",
            "ALTER TABLE users ADD COLUMN phone_verified BOOLEAN DEFAULT FALSE",
        )
        .await?;
        Self::ensure_table_column(
            &conn,
            "users",
            "phone_verified_at",
            "ALTER TABLE users ADD COLUMN phone_verified_at DATETIME",
        )
        .await?;

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
                verification_score INTEGER NOT NULL DEFAULT 35,
                verification_state TEXT NOT NULL DEFAULT 'needs_review',
                duplicate_of TEXT,
                corroboration_count INTEGER NOT NULL DEFAULT 0,
                triage_score INTEGER NOT NULL DEFAULT 0,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                resolved_at DATETIME,
                deleted_at DATETIME,
                FOREIGN KEY (reporter_id) REFERENCES users(id),
                FOREIGN KEY (assigned_to) REFERENCES users(id)
            )",
            (),
        )
        .await?;

        Self::ensure_table_column(&conn, "issues", "verification_score", "ALTER TABLE issues ADD COLUMN verification_score INTEGER NOT NULL DEFAULT 35").await?;
        Self::ensure_table_column(&conn, "issues", "verification_state", "ALTER TABLE issues ADD COLUMN verification_state TEXT NOT NULL DEFAULT 'needs_review'").await?;
        Self::ensure_table_column(&conn, "issues", "duplicate_of", "ALTER TABLE issues ADD COLUMN duplicate_of TEXT").await?;
        Self::ensure_table_column(&conn, "issues", "corroboration_count", "ALTER TABLE issues ADD COLUMN corroboration_count INTEGER NOT NULL DEFAULT 0").await?;
        Self::ensure_table_column(&conn, "issues", "triage_score", "ALTER TABLE issues ADD COLUMN triage_score INTEGER NOT NULL DEFAULT 0").await?;

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
        )
        .await?;

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
        )
        .await?;

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
        )
        .await?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS phone_otp_challenges (
                id TEXT PRIMARY KEY,
                phone TEXT NOT NULL,
                code_hash TEXT NOT NULL,
                expires_at DATETIME NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                consumed_at DATETIME,
                attempts INTEGER NOT NULL DEFAULT 0
            )",
            (),
        )
        .await?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_phone_otp_phone ON phone_otp_challenges(phone)",
            (),
        )
        .await?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS phone_verification_tokens (
                id TEXT PRIMARY KEY,
                phone TEXT NOT NULL,
                token_hash TEXT NOT NULL,
                expires_at DATETIME NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                consumed_at DATETIME
            )",
            (),
        )
        .await?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_phone_verification_tokens_phone ON phone_verification_tokens(phone)",
            (),
        )
        .await?;

        // Create indexes
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_issues_status ON issues(status)",
            (),
        )
        .await?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_issues_category ON issues(category)",
            (),
        )
        .await?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_issues_location ON issues(latitude, longitude)",
            (),
        )
        .await?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_issues_created ON issues(created_at)",
            (),
        )
        .await?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_issues_verification_state ON issues(verification_state)",
            (),
        )
        .await?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_issues_duplicate_of ON issues(duplicate_of)",
            (),
        )
        .await?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_issues_triage ON issues(triage_score)",
            (),
        )
        .await?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_users_email ON users(email)",
            (),
        )
        .await?;

        // Create FTS (Full Text Search) virtual table
        conn.execute(
            "CREATE VIRTUAL TABLE IF NOT EXISTS issues_fts USING fts5(
                title, description,
                content='issues',
                content_rowid='rowid'
            )",
            (),
        )
        .await?;

        Ok(())
    }

    /// Get database connection
    pub async fn conn(&self) -> tokio::sync::MutexGuard<'_, libsql::Connection> {
        self.client.lock().await
    }

    async fn ensure_table_column(
        conn: &libsql::Connection,
        table_name: &str,
        column_name: &str,
        alter_sql: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let pragma_sql = format!("PRAGMA table_info({})", table_name);
        let mut rows = conn.query(pragma_sql.as_str(), ()).await?;
        let mut exists = false;

        while let Some(row) = rows.next().await? {
            let current_name: String = row.get::<String>(1)?;
            if current_name == column_name {
                exists = true;
                break;
            }
        }

        if !exists {
            conn.execute(alter_sql, ()).await?;
        }

        Ok(())
    }

    /// Insert a new issue (split into insert + score metadata update to satisfy tuple param limits)
    pub async fn insert_issue(
        &self,
        issue: &Issue,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let conn = self.client.lock().await;
        conn.execute(
            "INSERT INTO issues (id, title, description, category, priority, status, latitude, longitude, address, reporter_id, assigned_to, media_urls, tags, created_at, updated_at, resolved_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
            (
                issue.id.to_string(),
                issue.title.clone(),
                issue.description.clone(),
                issue.category.as_str().to_string(),
                issue.priority.as_str().to_string(),
                issue.status.as_str().to_string(),
                issue.location.latitude,
                issue.location.longitude,
                issue.location.address.clone(),
                issue.reporter_id.map(|id| id.to_string()),
                issue.assigned_to.map(|id| id.to_string()),
                serde_json::to_string(&issue.media_urls)?,
                serde_json::to_string(&issue.tags)?,
                issue.created_at.to_rfc3339(),
                issue.updated_at.to_rfc3339(),
                issue.resolved_at.map(|dt| dt.to_rfc3339()),
            ),
        ).await?;

        conn.execute(
            "UPDATE issues
             SET verification_score = ?1,
                 verification_state = ?2,
                 duplicate_of = ?3,
                 corroboration_count = ?4,
                 triage_score = ?5
             WHERE id = ?6",
            (
                issue.verification_score,
                issue.verification_state.as_str(),
                issue.duplicate_of.map(|id| id.to_string()),
                issue.corroboration_count,
                issue.triage_score,
                issue.id.to_string(),
            ),
        )
        .await?;
        Ok(())
    }

    /// Fetch an issue by ID
    pub async fn get_issue(
        &self,
        id: Uuid,
    ) -> Result<Option<Issue>, Box<dyn std::error::Error + Send + Sync>> {
        let conn = self.client.lock().await;
        let mut rows = conn.query(
            "SELECT id, title, description, category, priority, status, latitude, longitude, address, reporter_id, assigned_to, media_urls, tags, verification_score, verification_state, duplicate_of, corroboration_count, triage_score, created_at, updated_at, resolved_at, deleted_at
             FROM issues WHERE id = ?1 AND deleted_at IS NULL",
            [id.to_string()],
        ).await?;

        if let Some(row) = rows.next().await? {
            Ok(Some(self.row_to_issue(&row)?))
        } else {
            Ok(None)
        }
    }

    /// Update an issue using current values as defaults for unchanged fields
    pub async fn update_issue(
        &self,
        id: Uuid,
        updates: &UpdateIssueRequest,
        current: &Issue,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let conn = self.client.lock().await;
        let title = updates.title.as_ref().unwrap_or(&current.title);
        let description = updates.description.as_ref().unwrap_or(&current.description);
        let status = updates.status.as_ref().unwrap_or(&current.status);
        let priority = updates.priority.as_ref().unwrap_or(&current.priority);
        let assigned_to = updates
            .assigned_to
            .as_ref()
            .or(current.assigned_to.as_ref());

        // Set resolved_at when transitioning to Resolved or Closed for the first time, and clear if reopened
        let resolved_at = if matches!(status, IssueStatus::Resolved | IssueStatus::Closed) {
            if current.resolved_at.is_none() {
                Some(chrono::Utc::now().to_rfc3339())
            } else {
                current.resolved_at.as_ref().map(|dt| dt.to_rfc3339())
            }
        } else {
            None
        };

        conn.execute(
            "UPDATE issues SET title = ?1, description = ?2, status = ?3, priority = ?4, assigned_to = ?5, updated_at = ?6, resolved_at = ?7 WHERE id = ?8 AND deleted_at IS NULL",
            (
                title.as_str(),
                description.as_str(),
                status.as_str(),
                priority.as_str(),
                assigned_to.map(|uid| uid.to_string()),
                chrono::Utc::now().to_rfc3339(),
                resolved_at,
                id.to_string(),
            ),
        ).await?;
        Ok(())
    }

    /// Soft delete an issue
    pub async fn delete_issue(
        &self,
        id: Uuid,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let conn = self.client.lock().await;
        conn.execute(
            "UPDATE issues SET deleted_at = ?1, updated_at = ?2 WHERE id = ?3 AND deleted_at IS NULL",
            (
                chrono::Utc::now().to_rfc3339(),
                chrono::Utc::now().to_rfc3339(),
                id.to_string(),
            ),
        ).await?;
        Ok(())
    }

    /// List issues with SQL-level filters, sorting, and pagination.
    /// Uses safe enum-based filtering to prevent SQL injection.
    pub async fn list_issues(
        &self,
        query: &IssueListQuery,
    ) -> Result<(Vec<Issue>, i64), Box<dyn std::error::Error + Send + Sync>> {
        let conn = self.client.lock().await;

        // Build WHERE clause - all values are from validated enums, safe from injection
        let mut where_clauses = vec!["deleted_at IS NULL".to_string()];

        // Status filter - only accepts IssueStatus enum values
        if let Some(status) = &query.status {
            where_clauses.push(format!("status = '{}'", status.as_str()));
        }
        // Category filter - only accepts IssueCategory enum values
        if let Some(category) = &query.category {
            where_clauses.push(format!("category = '{}'", category.as_str()));
        }
        // Priority filter - only accepts Priority enum values
        if let Some(priority) = &query.priority {
            where_clauses.push(format!("priority = '{}'", priority.as_str()));
        }
        // Verification state - only accepts VerificationState enum values
        if let Some(verification_state) = &query.verification_state {
            where_clauses.push(format!(
                "verification_state = '{}'",
                verification_state.as_str()
            ));
        }

        let where_sql = where_clauses.join(" AND ");

        // Sort SQL - using whitelisted values only, no user input
        let sort_sql = match query.sort.as_deref() {
            Some("updated_at") => "updated_at DESC, created_at DESC",
            Some("priority") => {
                "CASE priority
                    WHEN 'critical' THEN 4
                    WHEN 'high' THEN 3
                    WHEN 'medium' THEN 2
                    ELSE 1
                 END DESC, created_at DESC"
            }
            Some("triage") => {
                "(
                    CASE priority
                      WHEN 'critical' THEN 80
                      WHEN 'high' THEN 60
                      WHEN 'medium' THEN 40
                      ELSE 20
                    END
                    + ((100 - verification_score) / 2)
                    + MIN(corroboration_count * 5, 20)
                    + MIN(CAST((strftime('%s','now') - strftime('%s', created_at)) / 21600 AS INTEGER), 40)
                 ) DESC, created_at DESC"
            }
            _ => "created_at DESC",
        };

        let page = query.page.unwrap_or(1).max(1) as usize;
        let limit = query.limit.unwrap_or(20).clamp(1, 100) as usize;
        let offset = (page - 1) * limit;

        // Count query
        let count_sql = format!("SELECT COUNT(*) FROM issues WHERE {}", where_sql);
        let mut count_rows = conn.query(count_sql.as_str(), ()).await?;
        let total = if let Some(row) = count_rows.next().await? {
            row.get::<i64>(0)?
        } else {
            0
        };

        // Main query - LIMIT and OFFSET are safe integers
        let sql = format!(
            "SELECT id, title, description, category, priority, status, latitude, longitude, address, reporter_id, assigned_to, media_urls, tags, verification_score, verification_state, duplicate_of, corroboration_count, triage_score, created_at, updated_at, resolved_at, deleted_at
             FROM issues
             WHERE {}
             ORDER BY {}
             LIMIT {} OFFSET {}",
            where_sql, sort_sql, limit, offset
        );

        let mut rows = conn.query(sql.as_str(), ()).await?;
        let mut issues = Vec::new();
        let now = chrono::Utc::now();

        while let Some(row) = rows.next().await? {
            let mut issue = self.row_to_issue(&row)?;
            issue.triage_score = Self::calculate_triage_score(
                &issue.priority,
                issue.verification_score,
                issue.corroboration_count,
                issue.created_at,
                now,
            );
            issues.push(issue);
        }

        Ok((issues, total))
    }

    pub async fn list_duplicate_candidates(
        &self,
        latitude: f64,
        longitude: f64,
        delta: f64,
        active_since: DateTime<Utc>,
    ) -> Result<Vec<Issue>, Box<dyn std::error::Error + Send + Sync>> {
        let conn = self.client.lock().await;
        let min_lat = latitude - delta;
        let max_lat = latitude + delta;
        let min_lng = longitude - delta;
        let max_lng = longitude + delta;

        let mut rows = conn
            .query(
                "SELECT id, title, description, category, priority, status, latitude, longitude, address, reporter_id, assigned_to, media_urls, tags, verification_score, verification_state, duplicate_of, corroboration_count, triage_score, created_at, updated_at, resolved_at, deleted_at
                 FROM issues
                 WHERE deleted_at IS NULL
                   AND status IN ('reported', 'underreview', 'inprogress', 'escalated')
                   AND created_at >= ?1
                   AND latitude BETWEEN ?2 AND ?3
                   AND longitude BETWEEN ?4 AND ?5
                 ORDER BY created_at DESC",
                (
                    active_since.to_rfc3339(),
                    min_lat,
                    max_lat,
                    min_lng,
                    max_lng,
                ),
            )
            .await?;

        let mut issues = Vec::new();
        while let Some(row) = rows.next().await? {
            issues.push(self.row_to_issue(&row)?);
        }

        Ok(issues)
    }

    pub async fn increment_corroboration_count(
        &self,
        issue_id: Uuid,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let conn = self.client.lock().await;
        conn.execute(
            "UPDATE issues
             SET corroboration_count = corroboration_count + 1,
                 updated_at = ?1
             WHERE id = ?2 AND deleted_at IS NULL",
            (chrono::Utc::now().to_rfc3339(), issue_id.to_string()),
        )
        .await?;
        Ok(())
    }

    pub async fn update_issue_scores(
        &self,
        issue_id: Uuid,
        verification_score: i32,
        verification_state: VerificationState,
        triage_score: i32,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let conn = self.client.lock().await;
        conn.execute(
            "UPDATE issues
             SET verification_score = ?1,
                 verification_state = ?2,
                 triage_score = ?3,
                 updated_at = ?4
             WHERE id = ?5 AND deleted_at IS NULL",
            (
                verification_score,
                verification_state.as_str(),
                triage_score,
                chrono::Utc::now().to_rfc3339(),
                issue_id.to_string(),
            ),
        )
        .await?;
        Ok(())
    }

    pub async fn get_reporter_trust_signals(
        &self,
        reporter_id: Uuid,
    ) -> Result<(i64, i64, bool, bool), Box<dyn std::error::Error + Send + Sync>> {
        let conn = self.client.lock().await;

        let mut account_rows = conn
            .query(
                "SELECT created_at, phone_verified
                 FROM users
                 WHERE id = ?1",
                [reporter_id.to_string()],
            )
            .await?;

        let (account_age_days, phone_verified) = if let Some(row) = account_rows.next().await? {
            let created_at = row.get::<String>(0)?.parse::<DateTime<Utc>>()?;
            let phone_verified_value: i64 = row.get::<i64>(1).unwrap_or(0);
            (
                (chrono::Utc::now() - created_at).num_days(),
                phone_verified_value != 0,
            )
        } else {
            (0, false)
        };

        let mut resolved_rows = conn
            .query(
                "SELECT COUNT(*)
                 FROM issues
                 WHERE reporter_id = ?1
                   AND deleted_at IS NULL
                   AND status IN ('resolved', 'closed')",
                [reporter_id.to_string()],
            )
            .await?;
        let resolved_count = if let Some(row) = resolved_rows.next().await? {
            row.get::<i64>(0)?
        } else {
            0
        };

        let mut reopened_rows = conn
            .query(
                "SELECT COUNT(*)
                 FROM issue_status_history h
                 JOIN issues i ON i.id = h.issue_id
                 WHERE i.reporter_id = ?1
                   AND i.deleted_at IS NULL
                   AND h.old_status IN ('resolved', 'closed')
                   AND h.new_status NOT IN ('closed', 'resolved')",
                [reporter_id.to_string()],
            )
            .await?;
        let reopened_count = if let Some(row) = reopened_rows.next().await? {
            row.get::<i64>(0)?
        } else {
            0
        };

        Ok((resolved_count, account_age_days, reopened_count == 0, phone_verified))
    }

    pub fn calculate_triage_score(
        priority: &Priority,
        verification_score: i32,
        corroboration_count: i32,
        created_at: DateTime<Utc>,
        now: DateTime<Utc>,
    ) -> i32 {
        let priority_weight = match priority {
            Priority::Critical => 80,
            Priority::High => 60,
            Priority::Medium => 40,
            Priority::Low => 20,
        };
        let trust_review_weight = (100 - verification_score).max(0) / 2;
        let corroboration_boost = (corroboration_count.max(0) * 5).min(20);
        let age_hours = (now - created_at).num_hours().max(0);
        let age_boost = ((age_hours / 6) as i32).min(40);

        priority_weight + trust_review_weight + corroboration_boost + age_boost
    }

    pub async fn create_phone_otp_challenge(
        &self,
        challenge_id: Uuid,
        phone: &str,
        code_hash: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let conn = self.client.lock().await;
        conn.execute(
            "INSERT INTO phone_otp_challenges (id, phone, code_hash, expires_at, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            (
                challenge_id.to_string(),
                phone.to_string(),
                code_hash.to_string(),
                expires_at.to_rfc3339(),
                chrono::Utc::now().to_rfc3339(),
            ),
        )
        .await?;
        Ok(())
    }

    pub async fn get_latest_active_phone_otp_challenge(
        &self,
        phone: &str,
    ) -> Result<Option<(Uuid, String, DateTime<Utc>, i64)>, Box<dyn std::error::Error + Send + Sync>> {
        let conn = self.client.lock().await;
        let mut rows = conn
            .query(
                "SELECT id, code_hash, expires_at, attempts
                 FROM phone_otp_challenges
                 WHERE phone = ?1
                   AND consumed_at IS NULL
                 ORDER BY created_at DESC
                 LIMIT 1",
                [phone.to_string()],
            )
            .await?;

        if let Some(row) = rows.next().await? {
            let id = row.get::<String>(0)?.parse()?;
            let code_hash = row.get::<String>(1)?;
            let expires_at = row.get::<String>(2)?.parse::<DateTime<Utc>>()?;
            let attempts = row.get::<i64>(3)?;
            Ok(Some((id, code_hash, expires_at, attempts)))
        } else {
            Ok(None)
        }
    }

    pub async fn mark_phone_otp_attempt(
        &self,
        challenge_id: Uuid,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let conn = self.client.lock().await;
        conn.execute(
            "UPDATE phone_otp_challenges
             SET attempts = attempts + 1
             WHERE id = ?1",
            [challenge_id.to_string()],
        )
        .await?;
        Ok(())
    }

    pub async fn consume_phone_otp_challenge(
        &self,
        challenge_id: Uuid,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let conn = self.client.lock().await;
        conn.execute(
            "UPDATE phone_otp_challenges
             SET consumed_at = ?1
             WHERE id = ?2",
            (
                chrono::Utc::now().to_rfc3339(),
                challenge_id.to_string(),
            ),
        )
        .await?;
        Ok(())
    }

    pub async fn create_phone_verification_token(
        &self,
        token_id: Uuid,
        phone: &str,
        token_hash: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let conn = self.client.lock().await;
        conn.execute(
            "INSERT INTO phone_verification_tokens (id, phone, token_hash, expires_at, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            (
                token_id.to_string(),
                phone.to_string(),
                token_hash.to_string(),
                expires_at.to_rfc3339(),
                chrono::Utc::now().to_rfc3339(),
            ),
        )
        .await?;
        Ok(())
    }

    pub async fn consume_phone_verification_token(
        &self,
        phone: &str,
        token_hash: &str,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let conn = self.client.lock().await;
        let mut rows = conn
            .query(
                "SELECT id, expires_at
                 FROM phone_verification_tokens
                 WHERE phone = ?1
                   AND token_hash = ?2
                   AND consumed_at IS NULL
                 ORDER BY created_at DESC
                 LIMIT 1",
                (phone.to_string(), token_hash.to_string()),
            )
            .await?;

        let Some(row) = rows.next().await? else {
            return Ok(false);
        };

        let token_id: String = row.get::<String>(0)?;
        let expires_at = row.get::<String>(1)?.parse::<DateTime<Utc>>()?;
        if chrono::Utc::now() > expires_at {
            return Ok(false);
        }

        conn.execute(
            "UPDATE phone_verification_tokens
             SET consumed_at = ?1
             WHERE id = ?2",
            (chrono::Utc::now().to_rfc3339(), token_id),
        )
        .await?;

        Ok(true)
    }

    pub async fn mark_user_phone_verified(
        &self,
        user_id: Uuid,
        phone: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let conn = self.client.lock().await;
        conn.execute(
            "UPDATE users
             SET phone = ?1,
                 phone_verified = 1,
                 phone_verified_at = ?2,
                 updated_at = ?3
             WHERE id = ?4",
            (
                phone.to_string(),
                chrono::Utc::now().to_rfc3339(),
                chrono::Utc::now().to_rfc3339(),
                user_id.to_string(),
            ),
        )
        .await?;
        Ok(())
    }

    /// Insert a comment
    pub async fn insert_comment(
        &self,
        comment: &Comment,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let conn = self.client.lock().await;
        conn.execute(
            "INSERT INTO issue_comments (id, issue_id, author_id, content, is_internal, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            (
                comment.id.to_string(),
                comment.issue_id.to_string(),
                comment.author_id.to_string(),
                comment.content.as_str(),
                if comment.is_internal { "1" } else { "0" },
                comment.created_at.to_rfc3339(),
            ),
        )
        .await?;
        Ok(())
    }

    /// Get comments for an issue
    pub async fn get_comments(
        &self,
        issue_id: Uuid,
    ) -> Result<Vec<Comment>, Box<dyn std::error::Error + Send + Sync>> {
        let conn = self.client.lock().await;
        let mut rows = conn
            .query(
                "SELECT id, issue_id, author_id, content, is_internal, created_at
             FROM issue_comments WHERE issue_id = ?1 ORDER BY created_at DESC",
                [issue_id.to_string()],
            )
            .await?;

        let mut comments = vec![];
        while let Some(row) = rows.next().await? {
            comments.push(self.row_to_comment(&row)?);
        }
        Ok(comments)
    }

    /// Insert a status history entry
    pub async fn insert_status_history(
        &self,
        entry: &StatusHistoryEntry,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let conn = self.client.lock().await;
        conn.execute(
            "INSERT INTO issue_status_history (id, issue_id, old_status, new_status, changed_by, reason, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            (
                entry.id.to_string(),
                entry.issue_id.to_string(),
                entry.old_status.as_str(),
                entry.new_status.as_str(),
                entry.changed_by.to_string(),
                entry.reason.as_deref(),
                entry.created_at.to_rfc3339(),
            ),
        ).await?;
        Ok(())
    }

    /// Get status history for an issue
    pub async fn get_status_history(
        &self,
        issue_id: Uuid,
    ) -> Result<Vec<StatusHistoryEntry>, Box<dyn std::error::Error + Send + Sync>> {
        let conn = self.client.lock().await;
        let mut rows = conn
            .query(
                "SELECT id, issue_id, old_status, new_status, changed_by, reason, created_at
             FROM issue_status_history WHERE issue_id = ?1 ORDER BY created_at DESC",
                [issue_id.to_string()],
            )
            .await?;

        let mut history = vec![];
        while let Some(row) = rows.next().await? {
            history.push(self.row_to_status_history(&row)?);
        }
        Ok(history)
    }

    fn row_to_issue(
        &self,
        row: &libsql::Row,
    ) -> Result<Issue, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Issue {
            id: row.get::<String>(0)?.parse()?,
            title: row.get::<String>(1)?,
            description: row.get::<String>(2)?,
            category: row.get::<String>(3)?.parse()?,
            priority: row.get::<String>(4)?.parse()?,
            status: row.get::<String>(5)?.parse()?,
            location: GeoLocation {
                latitude: row.get::<f64>(6)?,
                longitude: row.get::<f64>(7)?,
                address: row.get::<Option<String>>(8)?,
            },
            reporter_id: row
                .get::<Option<String>>(9)?
                .map(|s| s.parse())
                .transpose()?,
            assigned_to: row
                .get::<Option<String>>(10)?
                .map(|s| s.parse())
                .transpose()?,
            media_urls: serde_json::from_str(&row.get::<String>(11)?)?,
            tags: serde_json::from_str(&row.get::<String>(12)?)?,
            verification_score: row.get::<i32>(13)?,
            verification_state: row.get::<String>(14)?.parse()?,
            duplicate_of: row
                .get::<Option<String>>(15)?
                .map(|s| s.parse())
                .transpose()?,
            corroboration_count: row.get::<i32>(16)?,
            triage_score: row.get::<i32>(17)?,
            created_at: row.get::<String>(18)?.parse::<DateTime<Utc>>()?,
            updated_at: row.get::<String>(19)?.parse::<DateTime<Utc>>()?,
            resolved_at: row
                .get::<Option<String>>(20)?
                .map(|s| s.parse())
                .transpose()?,
            deleted_at: row
                .get::<Option<String>>(21)?
                .map(|s| s.parse())
                .transpose()?,
        })
    }

    fn row_to_comment(
        &self,
        row: &libsql::Row,
    ) -> Result<Comment, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Comment {
            id: row.get::<String>(0)?.parse()?,
            issue_id: row.get::<String>(1)?.parse()?,
            author_id: row.get::<String>(2)?.parse()?,
            content: row.get::<String>(3)?,
            is_internal: row.get::<i32>(4)? != 0,
            created_at: row.get::<String>(5)?.parse::<DateTime<Utc>>()?,
        })
    }

    fn row_to_status_history(
        &self,
        row: &libsql::Row,
    ) -> Result<StatusHistoryEntry, Box<dyn std::error::Error + Send + Sync>> {
        Ok(StatusHistoryEntry {
            id: row.get::<String>(0)?.parse()?,
            issue_id: row.get::<String>(1)?.parse()?,
            old_status: row.get::<String>(2)?.parse()?,
            new_status: row.get::<String>(3)?.parse()?,
            changed_by: row.get::<String>(4)?.parse()?,
            reason: row.get::<Option<String>>(5)?,
            created_at: row.get::<String>(6)?.parse::<DateTime<Utc>>()?,
        })
    }

    /// Get analytics summary
    pub async fn get_analytics_summary(
        &self,
    ) -> Result<AnalyticsSummary, Box<dyn std::error::Error + Send + Sync>> {
        let conn = self.client.lock().await;

        // Total issues
        let mut total_rows = conn
            .query(
                "SELECT COUNT(*) as total FROM issues WHERE deleted_at IS NULL",
                (),
            )
            .await?;
        let total_issues: i64 = if let Some(row) = total_rows.next().await? {
            row.get::<i64>(0)?
        } else {
            0
        };

        // Resolved issues
        let mut resolved_rows = conn.query("SELECT COUNT(*) as count FROM issues WHERE status = 'resolved' AND deleted_at IS NULL", ()).await?;
        let resolved_issues: i64 = if let Some(row) = resolved_rows.next().await? {
            row.get::<i64>(0)?
        } else {
            0
        };

        // Closed issues
        let mut closed_rows = conn.query("SELECT COUNT(*) as count FROM issues WHERE status = 'closed' AND deleted_at IS NULL", ()).await?;
        let closed_issues: i64 = if let Some(row) = closed_rows.next().await? {
            row.get::<i64>(0)?
        } else {
            0
        };

        let open_issues = total_issues - resolved_issues - closed_issues;

        // Avg resolution time
        let mut avg_rows = conn.query(
            "SELECT AVG(CAST((strftime('%s', resolved_at) - strftime('%s', created_at)) AS REAL) / 3600.0) as avg_hours 
             FROM issues WHERE resolved_at IS NOT NULL AND deleted_at IS NULL",
            (),
        ).await?;
        let avg_resolution_hours: f64 = if let Some(row) = avg_rows.next().await? {
            row.get::<Option<f64>>(0)?.unwrap_or(0.0)
        } else {
            0.0
        };

        // By category
        let mut cat_rows = conn.query(
            "SELECT category, COUNT(*) as count FROM issues WHERE deleted_at IS NULL GROUP BY category",
            (),
        ).await?;
        let mut issues_by_category = Vec::new();
        while let Some(row) = cat_rows.next().await? {
            let cat_str: String = row.get::<String>(0)?;
            let count: i64 = row.get::<i64>(1)?;
            issues_by_category.push(CategoryCount {
                category: cat_str.parse()?,
                count,
            });
        }

        // By priority
        let mut pri_rows = conn.query(
            "SELECT priority, COUNT(*) as count FROM issues WHERE deleted_at IS NULL GROUP BY priority",
            (),
        ).await?;
        let mut issues_by_priority = Vec::new();
        while let Some(row) = pri_rows.next().await? {
            let pri_str: String = row.get::<String>(0)?;
            let count: i64 = row.get::<i64>(1)?;
            issues_by_priority.push(PriorityCount {
                priority: pri_str.parse()?,
                count,
            });
        }

        Ok(AnalyticsSummary {
            total_issues,
            open_issues,
            resolved_issues,
            avg_resolution_hours,
            issues_by_category,
            issues_by_priority,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;

    async fn setup_db() -> Database {
        let mut config = AppConfig::default();
        config.database.url = format!("sqlite://test-{}.db", Uuid::new_v4());
        let db = Database::new(&config).await.unwrap();
        db.migrate().await.unwrap();
        db
    }

    fn sample_issue() -> Issue {
        Issue {
            id: Uuid::new_v4(),
            title: "Pothole".to_string(),
            description: "Large pothole on Main St".to_string(),
            category: IssueCategory::Infrastructure,
            priority: Priority::Medium,
            status: IssueStatus::Reported,
            location: GeoLocation {
                latitude: 13.7563,
                longitude: 100.5018,
                address: Some("Bangkok".to_string()),
            },
            reporter_id: None,
            assigned_to: None,
            media_urls: vec![],
            tags: vec![],
            verification_score: 35,
            verification_state: VerificationState::NeedsReview,
            duplicate_of: None,
            corroboration_count: 0,
            triage_score: 0,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            resolved_at: None,
            deleted_at: None,
        }
    }

    #[tokio::test]
    async fn test_create_and_get_issue() {
        let db = setup_db().await;
        let issue = sample_issue();
        db.insert_issue(&issue).await.unwrap();

        let (issues, total) = db.list_issues(&IssueListQuery::default()).await.unwrap();
        assert_eq!(total, 1);
        assert_eq!(issues[0].title, "Pothole");
        assert_eq!(issues[0].status, IssueStatus::Reported);
    }

    #[tokio::test]
    async fn test_update_issue_status() {
        let db = setup_db().await;
        let issue = sample_issue();
        db.insert_issue(&issue).await.unwrap();

        let (issues, _) = db.list_issues(&IssueListQuery::default()).await.unwrap();
        let id = issues[0].id;

        let update = UpdateIssueRequest {
            title: None,
            description: None,
            status: Some(IssueStatus::InProgress),
            priority: None,
            assigned_to: None,
            reason: Some("Started repair".to_string()),
        };
        db.update_issue(id, &update, &issues[0]).await.unwrap();

        let updated = db.get_issue(id).await.unwrap().unwrap();
        assert_eq!(updated.status, IssueStatus::InProgress);
    }

    #[tokio::test]
    async fn test_resolved_at_set_on_close() {
        let db = setup_db().await;
        let issue = sample_issue();
        db.insert_issue(&issue).await.unwrap();

        let (issues, _) = db.list_issues(&IssueListQuery::default()).await.unwrap();
        let id = issues[0].id;

        let update1 = UpdateIssueRequest {
            title: None,
            description: None,
            status: Some(IssueStatus::InProgress),
            priority: None,
            assigned_to: None,
            reason: Some("Started".to_string()),
        };
        db.update_issue(id, &update1, &issues[0]).await.unwrap();

        let current = db.get_issue(id).await.unwrap().unwrap();

        let update2 = UpdateIssueRequest {
            title: None,
            description: None,
            status: Some(IssueStatus::Resolved),
            priority: None,
            assigned_to: None,
            reason: Some("Done".to_string()),
        };
        db.update_issue(id, &update2, &current).await.unwrap();

        let resolved = db.get_issue(id).await.unwrap().unwrap();
        assert_eq!(resolved.status, IssueStatus::Resolved);
        assert!(resolved.resolved_at.is_some(), "resolved_at should be set");
    }

    #[tokio::test]
    async fn test_resolved_at_cleared_on_reopen() {
        let db = setup_db().await;
        let issue = sample_issue();
        db.insert_issue(&issue).await.unwrap();

        let (issues, _) = db.list_issues(&IssueListQuery::default()).await.unwrap();
        let id = issues[0].id;

        let update1 = UpdateIssueRequest {
            title: None,
            description: None,
            status: Some(IssueStatus::Resolved),
            priority: None,
            assigned_to: None,
            reason: Some("Done".to_string()),
        };
        db.update_issue(id, &update1, &issues[0]).await.unwrap();

        let resolved = db.get_issue(id).await.unwrap().unwrap();
        assert!(resolved.resolved_at.is_some());

        let update2 = UpdateIssueRequest {
            title: None,
            description: None,
            status: Some(IssueStatus::InProgress),
            priority: None,
            assigned_to: None,
            reason: Some("Reopened".to_string()),
        };
        db.update_issue(id, &update2, &resolved).await.unwrap();

        let reopened = db.get_issue(id).await.unwrap().unwrap();
        assert_eq!(reopened.status, IssueStatus::InProgress);
        assert!(
            reopened.resolved_at.is_none(),
            "resolved_at should be cleared"
        );
    }

    #[tokio::test]
    async fn test_soft_delete_issue() {
        let db = setup_db().await;
        let issue = sample_issue();
        db.insert_issue(&issue).await.unwrap();

        let (issues, total) = db.list_issues(&IssueListQuery::default()).await.unwrap();
        assert_eq!(total, 1);

        db.delete_issue(issues[0].id).await.unwrap();
        let (_, total_after) = db.list_issues(&IssueListQuery::default()).await.unwrap();
        assert_eq!(total_after, 0);
    }

    #[tokio::test]
    async fn test_analytics_summary() {
        let db = setup_db().await;
        let issue = sample_issue();
        db.insert_issue(&issue).await.unwrap();

        let summary = db.get_analytics_summary().await.unwrap();
        assert_eq!(summary.total_issues, 1);
        assert_eq!(summary.open_issues, 1);
    }
}
