use axum::{
    extract::Json,
    http::StatusCode,
    routing::{get, patch, post},
    Router,
};
use civic_sentinel_api::{config, db, middleware, routes, AppState};
use serde::Serialize;
use std::path::PathBuf;
use std::net::SocketAddr;
use std::sync::Arc;
use axum::http::HeaderValue;
use tower_http::cors::{AllowOrigin, Any, CorsLayer};
use tower_http::services::ServeDir;
use tracing::info;

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
            "POST /api/v1/auth/otp/request — Request phone OTP".to_string(),
            "POST /api/v1/auth/otp/verify — Verify phone OTP".to_string(),
            "POST /api/v1/uploads — Upload issue image".to_string(),
            "POST /api/v1/issues — Report an issue (public)".to_string(),
            "GET /api/v1/issues — List issues".to_string(),
            "GET /api/v1/issues/:id — Get issue details".to_string(),
            "PATCH /api/v1/issues/:id — Update issue (auth required)".to_string(),
            "DELETE /api/v1/issues/:id — Delete issue (auth required)".to_string(),
            "GET /api/v1/analytics — Impact analytics".to_string(),
        ],
    })
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    info!("Starting Civic Sentinel API...");

    let app_config = match config::AppConfig::load() {
        Ok(cfg) => Arc::new(cfg),
        Err(e) => {
            tracing::warn!("Failed to load config file ({}), using defaults", e);
            Arc::new(config::AppConfig::default())
        }
    };

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
    let uploads_dir = std::env::var("UPLOAD_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("uploads"));
    if let Err(e) = std::fs::create_dir_all(&uploads_dir) {
        tracing::warn!("Failed to pre-create uploads directory ({}): {}", uploads_dir.display(), e);
    }

    let public_routes = Router::new()
        .route("/health", get(health_check))
        .route("/api/v1/info", get(api_info))
        .route("/api/v1/auth/register", post(routes::auth::register))
        .route("/api/v1/auth/login", post(routes::auth::login))
        .route("/api/v1/auth/refresh", post(routes::auth::refresh))
        .route("/api/v1/auth/logout", post(routes::auth::logout))
        .route("/api/v1/auth/otp/request", post(routes::auth::request_phone_otp))
        .route("/api/v1/auth/otp/verify", post(routes::auth::verify_phone_otp))
        .route("/api/v1/uploads", post(routes::uploads::upload_issue_media))
        .route(
            "/api/v1/issues",
            get(routes::issues::list_issues).post(routes::issues::create_issue),
        )
        .route("/api/v1/issues/:id", get(routes::issues::get_issue))
        .route(
            "/api/v1/issues/:id/comments",
            get(routes::comments::get_comments),
        )
        .route(
            "/api/v1/issues/:id/history",
            get(routes::status_history::get_status_history),
        )
        .route("/api/v1/analytics", get(routes::analytics::get_analytics))
        .nest_service("/uploads", ServeDir::new(uploads_dir.clone()));

    let protected_routes = Router::new()
        .route(
            "/api/v1/issues/:id",
            patch(routes::issues::update_issue).delete(routes::issues::delete_issue),
        )
        .route(
            "/api/v1/issues/:id/comments",
            post(routes::comments::create_comment),
        )
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
        .layer({
            let cors_origins = &state.config.security.cors_origins;
            let origin_layer = if cors_origins.iter().any(|o| o == "*") {
                CorsLayer::new().allow_origin(Any)
            } else {
                let origins: Vec<HeaderValue> = cors_origins
                    .iter()
                    .filter_map(|o| o.parse::<HeaderValue>().ok())
                    .collect();
                CorsLayer::new().allow_origin(AllowOrigin::list(origins))
            };
            origin_layer.allow_methods(Any).allow_headers(Any)
        })
        .with_state(state.clone());

    let addr = SocketAddr::from(([0, 0, 0, 0], state.config.server.port));
    info!("API server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
