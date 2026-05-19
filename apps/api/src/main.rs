use axum::{
    http::StatusCode,
    response::Json,
    routing::{delete, get, patch, post},
    Router,
};
use serde::Serialize;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::info;

mod config;
mod db;
mod middleware;
mod models;
mod routes;
mod services;

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
            "PATCH /api/v1/issues/:id — Update an issue".to_string(),
            "DELETE /api/v1/issues/:id — Delete an issue".to_string(),
            "POST /api/v1/issues/:id/comments — Add a comment".to_string(),
            "GET /api/v1/issues/:id/comments — List comments".to_string(),
            "GET /api/v1/issues/:id/history — Get status history".to_string(),
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
        Ok(cfg) => cfg,
        Err(e) => {
            tracing::warn!("Failed to load config file ({}), using defaults", e);
            config::AppConfig::default()
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

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/v1/info", get(api_info))
        .route(
            "/api/v1/issues",
            get(routes::issues::list_issues).post(routes::issues::create_issue),
        )
        .route(
            "/api/v1/issues/:id",
            get(routes::issues::get_issue)
                .patch(routes::issues::update_issue)
                .delete(routes::issues::delete_issue),
        )
        .route(
            "/api/v1/issues/:id/comments",
            get(routes::comments::get_comments).post(routes::comments::create_comment),
        )
        .route(
            "/api/v1/issues/:id/history",
            get(routes::status_history::get_status_history),
        )
        .route("/api/v1/analytics", get(routes::analytics::get_analytics))
        .with_state(database);

    let addr = SocketAddr::from(([0, 0, 0, 0], app_config.server.port));
    info!("API server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
