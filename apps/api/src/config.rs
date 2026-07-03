use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use std::env;

/// Application configuration loaded from environment variables
/// and optional config files
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub auth: AuthConfig,
    pub security: SecurityConfig,
    pub notification: NotificationConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub jwt_expiration_hours: i64,
    pub refresh_token_expiration_days: i64,
    pub argon2_memory: u32,
    pub argon2_iterations: u32,
    pub argon2_parallelism: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SecurityConfig {
    pub rate_limit_requests: u32,
    pub rate_limit_window_seconds: u64,
    pub cors_origins: Vec<String>,
    pub allowed_hosts: Vec<String>,
    pub enable_hmac: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NotificationConfig {
    pub smtp_host: Option<String>,
    pub smtp_port: Option<u16>,
    pub smtp_user: Option<String>,
    pub smtp_password: Option<String>,
    pub line_channel_token: Option<String>,
    pub webhook_url: Option<String>,
}

impl AppConfig {
    /// Load configuration from files and environment variables
    /// Priority: ENV vars > config files > defaults
    pub fn load() -> Result<Self, ConfigError> {
        let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());
        let env_prefix = env::var("ENV_PREFIX").unwrap_or_else(|_| "CIVIC".into());

        let config = Config::builder()
            // Default config
            .add_source(File::with_name("config/default").required(false))
            // Environment-specific config
            .add_source(File::with_name(&format!("config/{}", run_mode)).required(false))
            // Local overrides (gitignored)
            .add_source(File::with_name("config/local").required(false))
            // Environment variables with prefix
            .add_source(Environment::with_prefix(&env_prefix).separator("_"))
            .build()?;

        config.try_deserialize()
    }

    /// Get database URL from environment or default
    pub fn database_url(&self) -> String {
        env::var("DATABASE_URL").unwrap_or_else(|_| self.database.url.clone())
    }

    /// Validate JWT secret meets security requirements
    /// Returns error if secret is insecure
    pub fn validate_jwt_secret(&self) -> Result<(), String> {
        let secret = env::var("JWT_SECRET").unwrap_or_else(|_| self.auth.jwt_secret.clone());

        // Check minimum length
        if secret.len() < 32 {
            return Err(format!(
                "JWT_SECRET must be at least 32 characters long (current: {} chars). \
                 Generate a secure secret with: openssl rand -base64 32",
                secret.len()
            ));
        }

        // Check if it's the default insecure value
        let insecure_defaults = [
            "change-me-in-production-32-char-min",
            "your-secret-key-here-must-be-32-chars",
            "insecure-jwt-secret-change-me-now-32",
        ];

        if insecure_defaults.contains(&secret.as_str()) {
            return Err(
                "JWT_SECRET is set to a default insecure value. \
                 Change it immediately! Generate with: openssl rand -base64 32".to_string()
            );
        }

        // Warn if secret looks weak (all same character, sequential, etc.)
        if secret.chars().all(|c| c == secret.chars().next().unwrap()) {
            return Err(
                "JWT_SECRET appears to be weak (repeated characters). \
                 Use a cryptographically random secret: openssl rand -base64 32".to_string()
            );
        }

        Ok(())
    }

    /// Get JWT secret (must be set in production)
    pub fn jwt_secret(&self) -> String {
        env::var("JWT_SECRET").unwrap_or_else(|_| self.auth.jwt_secret.clone())
    }
}

/// Default configuration for development
impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 3000,
                workers: num_cpus::get(),
            },
            database: DatabaseConfig {
                url: "sqlite://data/civic-sentinel.db".to_string(),
                max_connections: 10,
                timeout_seconds: 30,
            },
            auth: AuthConfig {
                jwt_secret: "change-me-in-production-32-char-min".to_string(),
                jwt_expiration_hours: 24,
                refresh_token_expiration_days: 7,
                argon2_memory: 65536,
                argon2_iterations: 3,
                argon2_parallelism: 4,
            },
            security: SecurityConfig {
                rate_limit_requests: 100,
                rate_limit_window_seconds: 60,
                cors_origins: vec!["http://localhost:5173".to_string()],
                allowed_hosts: vec!["localhost".to_string()],
                enable_hmac: false,
            },
            notification: NotificationConfig {
                smtp_host: None,
                smtp_port: None,
                smtp_user: None,
                smtp_password: None,
                line_channel_token: None,
                webhook_url: None,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_secret_too_short() {
        let mut config = AppConfig::default();
        config.auth.jwt_secret = "short".to_string();

        let result = config.validate_jwt_secret();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("at least 32 characters"));
    }

    #[test]
    fn test_jwt_secret_default_value() {
        let mut config = AppConfig::default();
        config.auth.jwt_secret = "change-me-in-production-32-char-min".to_string();

        let result = config.validate_jwt_secret();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("default insecure value"));
    }

    #[test]
    fn test_jwt_secret_weak_repeated_chars() {
        let mut config = AppConfig::default();
        config.auth.jwt_secret = "a".repeat(32);

        let result = config.validate_jwt_secret();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("weak"));
    }

    #[test]
    fn test_jwt_secret_valid() {
        let mut config = AppConfig::default();
        config.auth.jwt_secret = "a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6".to_string();

        let result = config.validate_jwt_secret();
        assert!(result.is_ok());
    }

    #[test]
    fn test_jwt_secret_valid_base64() {
        let mut config = AppConfig::default();
        config.auth.jwt_secret = "dGVzdC1zZWNyZXQta2V5LXRoYXQtaXMtbG9uZy1lbm91Z2g=".to_string();

        let result = config.validate_jwt_secret();
        assert!(result.is_ok());
    }
}
