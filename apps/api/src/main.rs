use axum::{
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

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
        description: "AI-powered civic resilience platform".to_string(),
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
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set tracing subscriber");

    info!("🛡️ Starting Civic Sentinel API...");

    // Build router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/v1/info", get(api_info))
        .route("/api/v1/issues", get(routes::issues::list_issues).post(routes::issues::create_issue))
        .route("/api/v1/issues/:id", get(routes::issues::get_issue))
        .route("/api/v1/analytics", get(routes::analytics::get_analytics));

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    info!("🚀 API server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
