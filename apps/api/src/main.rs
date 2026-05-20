#![allow(dead_code, clippy::duplicate_mod)]

use axum::{
    extract::Json,
    http::StatusCode,
    routing::{delete, get, patch, post},
    Router,
};
use serde::Serialize;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

mod config;
mod db;
mod middleware;
mod models;
mod routes;
mod services;

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub db: db::Database,
    pub config: Arc<config::AppConfig>,
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    version: String,
    timestamp: String,
}

#[derive(Serialize)]
struct ApiInfo {
    name: String,
    version: String,
    description: String,
    endpoints: Vec<String>,
}

async fn health_check() -> (StatusCode, Json<HealthResponse>) {
    let response = HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    (StatusCode::OK, Json(response))
}

async fn api_info() -> Json<ApiInfo> {
    Json(ApiInfo {
        name: "Civic Sentinel API".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        description: "Civic issue reporting and tracking platform".to_string(),
        endpoints: vec![
            "GET /health — Health check".to_string(),
            "GET /api/v1/info — API information".to_string(),
            "POST /api/v1/issues — Report an issue".to_string(),
            "GET /api/v1/issues — List issues".to_string(),
            "GET /api/v1/issues/:id — Get issue details".to_string(),
            "GET /api/v1/analytics — Impact analytics".to_string(),
        ],
    })
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("Starting Civic Sentinel API...");

    // Load configuration
    let app_config = match config::AppConfig::load() {
        Ok(cfg) => Arc::new(cfg),
        Err(e) => {
            tracing::warn!("Failed to load config file ({}), using defaults", e);
            Arc::new(config::AppConfig::default())
        }
    };

    // Initialize database
    let database = match db::Database::new(&app_config).await {
        Ok(db) => {
            info!("Database connected successfully");
            if let Err(e) = db.migrate().await {
                tracing::error!("Database migration failed: {}", e);
                std::process::exit(1);
            }
            info!("Database migrations completed");
            db
        }
        Err(e) => {
            tracing::error!("Database connection failed: {}", e);
            std::process::exit(1);
        }
    };

    let state = AppState {
        db: database,
        config: app_config,
    };

    // Build router
    let public_routes = Router::new()
        .route("/health", get(health_check))
        .route("/api/v1/info", get(api_info))
        .route("/api/v1/auth/register", post(routes::auth::register))
        .route("/api/v1/auth/login", post(routes::auth::login))
        .route("/api/v1/auth/refresh", post(routes::auth::refresh))
        .route("/api/v1/auth/logout", post(routes::auth::logout))
        .route("/api/v1/issues", get(routes::issues::list_issues))
        .route("/api/v1/issues/:id", get(routes::issues::get_issue))
        .route("/api/v1/issues/:id/comments", get(routes::comments::get_comments))
        .route("/api/v1/issues/:id/history", get(routes::status_history::get_status_history))
        .route("/api/v1/analytics", get(routes::analytics::get_analytics));

    let protected_routes = Router::new()
        .route("/api/v1/issues", post(routes::issues::create_issue))
        .route("/api/v1/issues/:id", patch(routes::issues::update_issue).delete(routes::issues::delete_issue))
        .route("/api/v1/issues/:id/comments", post(routes::comments::create_comment))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            middleware::auth::auth_middleware,
        ));

    let app = public_routes
        .merge(protected_routes)
        .layer(axum::middleware::from_fn(
            middleware::security_headers::security_headers_middleware,
        ))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            middleware::rate_limit::rate_limit_middleware,
        ))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    info!("API server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
