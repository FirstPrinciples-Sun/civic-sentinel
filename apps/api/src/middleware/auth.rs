use axum::{
    body::Body,
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::config::AppConfig;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,        // User ID
    pub email: String,      // User email
    pub role: UserRole,     // User role
    pub iat: i64,           // Issued at
    pub exp: i64,           // Expiration
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    Admin,
    Responder,
    Reporter,
    Viewer,
}

/// Authentication middleware
/// Extracts JWT from Authorization header and validates it
pub async fn auth_middleware(
    State(config): State<Arc<AppConfig>>,
    mut request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "));

    let token = match auth_header {
        Some(token) => token,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    let validation = Validation::new(Algorithm::HS256);
    let decoding_key = DecodingKey::from_secret(config.jwt_secret().as_bytes());

    let claims = match decode::<Claims>(token, &decoding_key, &validation) {
        Ok(token_data) => token_data.claims,
        Err(_) => return Err(StatusCode::UNAUTHORIZED),
    };

    // Check expiration
    let now = chrono::Utc::now().timestamp();
    if claims.exp < now {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Add claims to request extensions for downstream handlers
    request.extensions_mut().insert(claims);

    Ok(next.run(request).await)
}

/// Extract claims from request extensions
pub fn get_claims(request: &Request<Body>) -> Option<&Claims> {
    request.extensions().get::<Claims>()
}
