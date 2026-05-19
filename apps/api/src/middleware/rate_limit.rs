use axum::{
    extract::{ConnectInfo, Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;

use crate::config::AppConfig;

/// Rate limiter using token bucket algorithm
#[derive(Clone)]
pub struct RateLimiter {
    buckets: Arc<RwLock<HashMap<String, TokenBucket>>>,
    config: Arc<AppConfig>,
}

#[derive(Clone)]
struct TokenBucket {
    tokens: u32,
    last_update: Instant,
}

impl RateLimiter {
    pub fn new(config: Arc<AppConfig>) -> Self {
        Self {
            buckets: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    pub async fn is_allowed(&self, key: &str) -> bool {
        let mut buckets = self.buckets.write().await;
        let now = Instant::now();
        let window = Duration::from_secs(self.config.security.rate_limit_window_seconds);
        let max_requests = self.config.security.rate_limit_requests;

        let bucket = buckets.entry(key.to_string()).or_insert(TokenBucket {
            tokens: max_requests,
            last_update: now,
        });

        // Add tokens based on time elapsed
        let elapsed = now.duration_since(bucket.last_update);
        let tokens_to_add = (elapsed.as_secs_f64() / window.as_secs_f64() * max_requests as f64) as u32;
        bucket.tokens = (bucket.tokens + tokens_to_add).min(max_requests);
        bucket.last_update = now;

        if bucket.tokens > 0 {
            bucket.tokens -= 1;
            true
        } else {
            false
        }
    }

    /// Cleanup old buckets periodically
    pub async fn cleanup(&self) {
        let mut buckets = self.buckets.write().await;
        let now = Instant::now();
        let max_age = Duration::from_secs(300); // 5 minutes
        
        buckets.retain(|_, bucket| {
            now.duration_since(bucket.last_update) < max_age
        });
    }
}

/// Rate limiting middleware
pub async fn rate_limit_middleware(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(limiter): State<Arc<RateLimiter>>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let key = addr.ip().to_string();

    if !limiter.is_allowed(&key).await {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    Ok(next.run(request).await)
}
