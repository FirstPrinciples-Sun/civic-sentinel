#![allow(dead_code, clippy::duplicate_mod)]

pub mod config;
pub mod db;
pub mod error;
pub mod middleware;
pub mod models;
pub mod routes;
pub mod services;

use std::sync::Arc;

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub db: db::Database,
    pub config: Arc<config::AppConfig>,
}
