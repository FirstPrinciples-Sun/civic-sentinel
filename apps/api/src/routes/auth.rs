use axum::{
    extract::{Json, State},
    http::StatusCode,
};
use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use rand::Rng;
use serde_json::{json, Value};
use uuid::Uuid;
use validator::Validate;

use crate::{
    db::Database,
    middleware::auth::{Claims, UserRole},
    models::user::{AuthResponse, LoginRequest, RefreshTokenRequest, RegisterRequest, UserResponse},
};

#[derive(Clone)]
pub struct AuthState {
    pub db: std::sync::Arc<Database>,
    pub jwt_secret: String,
    pub jwt_expiration_hours: i64,
    pub refresh_token_expiration_days: i64,
}

fn generate_refresh_token() -> String {
    let mut rng = rand::thread_rng();
    (0..64)
        .map(|_| rng.sample(rand::distributions::Alphanumeric) as char)
        .collect()
}

fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    use argon2::{
        password_hash::{rand_core::OsRng, SaltString},
        Argon2, PasswordHasher,
    };
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
    Ok(password_hash.to_string())
}

fn verify_password(password: &str, hash: &str) -> Result<(), argon2::password_hash::Error> {
    use argon2::{Argon2, PasswordHash, PasswordVerifier};
    let parsed_hash = PasswordHash::new(hash)?;
    Argon2::default().verify_password(password.as_bytes(), &parsed_hash)
}

fn generate_jwt(
    user_id: Uuid,
    email: &str,
    role: &UserRole,
    secret: &str,
    expiration_hours: i64,
) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let claims = Claims {
        sub: user_id.to_string(),
        email: email.to_string(),
        role: role.clone(),
        iat: now.timestamp(),
        exp: (now + Duration::hours(expiration_hours)).timestamp(),
    };
    encode(
        &Header::new(jsonwebtoken::Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

fn parse_role(role_str: &str) -> UserRole {
    match role_str {
        "admin" => UserRole::Admin,
        "responder" => UserRole::Responder,
        "reporter" => UserRole::Reporter,
        _ => UserRole::Viewer,
    }
}

pub async fn register(
    State(state): State<AuthState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<Value>)> {
    if let Err(errors) = payload.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "success": false,
                "error": "Validation failed",
                "details": errors.field_errors()
            })),
        ));
    }

    let conn = state.db.conn().await;

    // Check if user already exists
    let mut rows = conn
        .query("SELECT id FROM users WHERE email = ?", [payload.email.clone()])
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "success": false,
                    "error": format!("Database error: {}", e)
                })),
            )
        })?;

    if rows
        .next()
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "success": false,
                    "error": format!("Database error: {}", e)
                })),
            )
        })?
        .is_some()
    {
        return Err((
            StatusCode::CONFLICT,
            Json(json!({
                "success": false,
                "error": "Email already registered"
            })),
        ));
    }

    let user_id = Uuid::new_v4();
    let password_hash = hash_password(&payload.password).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "error": format!("Password hashing failed: {}", e)
            })),
        )
    })?;

    let now = Utc::now();
    conn.execute(
        "INSERT INTO users (id, email, password_hash, name, role, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
        [
            user_id.to_string(),
            payload.email.clone(),
            password_hash,
            payload.name.clone(),
            "reporter".to_string(),
            now.to_rfc3339(),
            now.to_rfc3339(),
        ],
    )
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "error": format!("Failed to create user: {}", e)
            })),
        )
    })?;

    let access_token = generate_jwt(
        user_id,
        &payload.email,
        &UserRole::Reporter,
        &state.jwt_secret,
        state.jwt_expiration_hours,
    )
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "error": format!("JWT generation failed: {}", e)
            })),
        )
    })?;

    let refresh_token = generate_refresh_token();
    let refresh_expires = now + Duration::days(state.refresh_token_expiration_days);

    conn.execute(
        "INSERT INTO refresh_tokens (id, user_id, token_hash, expires_at, created_at) VALUES (?, ?, ?, ?, ?)",
        [
            Uuid::new_v4().to_string(),
            user_id.to_string(),
            refresh_token.clone(),
            refresh_expires.to_rfc3339(),
            now.to_rfc3339(),
        ],
    )
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "error": format!("Failed to store refresh token: {}", e)
            })),
        )
    })?;

    Ok(Json(AuthResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: state.jwt_expiration_hours * 3600,
        user: UserResponse {
            id: user_id,
            email: payload.email,
            name: Some(payload.name),
            role: UserRole::Reporter,
        },
    }))
}

pub async fn login(
    State(state): State<AuthState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<Value>)> {
    if let Err(errors) = payload.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "success": false,
                "error": "Validation failed",
                "details": errors.field_errors()
            })),
        ));
    }

    let conn = state.db.conn().await;
    let mut rows = conn
        .query(
            "SELECT id, email, password_hash, name, role FROM users WHERE email = ?",
            [payload.email.clone()],
        )
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "success": false,
                    "error": format!("Database error: {}", e)
                })),
            )
        })?;

    let row = rows.next().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "error": format!("Database error: {}", e)
            })),
        )
    })?;

    let row = match row {
        Some(r) => r,
        None => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "success": false,
                    "error": "Invalid email or password"
                })),
            ));
        }
    };

    let user_id: String = row.get(0).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "error": format!("Data parsing error: {}", e)
            })),
        )
    })?;
    let email: String = row.get(1).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "error": format!("Data parsing error: {}", e)
            })),
        )
    })?;
    let password_hash: String = row.get(2).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "error": format!("Data parsing error: {}", e)
            })),
        )
    })?;
    let name: Option<String> = row.get(3).ok();
    let role_str: String = row.get(4).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "error": format!("Data parsing error: {}", e)
            })),
        )
    })?;

    verify_password(&payload.password, &password_hash).map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "success": false,
                "error": "Invalid email or password"
            })),
        )
    })?;

    let role = parse_role(&role_str);
    let user_uuid = Uuid::parse_str(&user_id).unwrap_or_else(|_| Uuid::new_v4());

    let access_token = generate_jwt(
        user_uuid,
        &email,
        &role,
        &state.jwt_secret,
        state.jwt_expiration_hours,
    )
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "error": format!("JWT generation failed: {}", e)
            })),
        )
    })?;

    let refresh_token = generate_refresh_token();
    let refresh_expires = Utc::now() + Duration::days(state.refresh_token_expiration_days);

    conn.execute(
        "INSERT INTO refresh_tokens (id, user_id, token_hash, expires_at, created_at) VALUES (?, ?, ?, ?, ?)",
        [
            Uuid::new_v4().to_string(),
            user_id,
            refresh_token.clone(),
            refresh_expires.to_rfc3339(),
            Utc::now().to_rfc3339(),
        ],
    )
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "error": format!("Failed to store refresh token: {}", e)
            })),
        )
    })?;

    Ok(Json(AuthResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: state.jwt_expiration_hours * 3600,
        user: UserResponse {
            id: user_uuid,
            email,
            name,
            role,
        },
    }))
}

pub async fn refresh(
    State(state): State<AuthState>,
    Json(payload): Json<RefreshTokenRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<Value>)> {
    let conn = state.db.conn().await;
    let mut rows = conn
        .query(
            "SELECT user_id, expires_at, revoked_at FROM refresh_tokens WHERE token_hash = ?",
            [payload.refresh_token.clone()],
        )
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "success": false,
                    "error": format!("Database error: {}", e)
                })),
            )
        })?;

    let row = rows.next().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "error": format!("Database error: {}", e)
            })),
        )
    })?;

    let row = match row {
        Some(r) => r,
        None => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "success": false,
                    "error": "Invalid refresh token"
                })),
            ));
        }
    };

    let user_id: String = row.get(0).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "error": format!("Data parsing error: {}", e)
            })),
        )
    })?;
    let expires_at_str: String = row.get(1).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "error": format!("Data parsing error: {}", e)
            })),
        )
    })?;
    let revoked_at: Option<String> = row.get(2).ok();

    let expires_at: DateTime<Utc> = expires_at_str.parse().map_err(|e: chrono::ParseError| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "error": format!("Date parsing error: {}", e)
            })),
        )
    })?;

    if Utc::now() > expires_at {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "success": false,
                "error": "Refresh token expired"
            })),
        ));
    }

    if revoked_at.is_some() {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "success": false,
                "error": "Refresh token revoked"
            })),
        ));
    }

    // Fetch user details
    let mut user_rows = conn
        .query(
            "SELECT id, email, name, role FROM users WHERE id = ?",
            [user_id.clone()],
        )
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "success": false,
                    "error": format!("Database error: {}", e)
                })),
            )
        })?;

    let user_row = user_rows.next().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "error": format!("Database error: {}", e)
            })),
        )
    })?;

    let user_row = match user_row {
        Some(r) => r,
        None => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "success": false,
                    "error": "User not found"
                })),
            ));
        }
    };

    let id: String = user_row.get(0).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "error": format!("Data parsing error: {}", e)
            })),
        )
    })?;
    let email: String = user_row.get(1).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "error": format!("Data parsing error: {}", e)
            })),
        )
    })?;
    let name: Option<String> = user_row.get(2).ok();
    let role_str: String = user_row.get(3).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "error": format!("Data parsing error: {}", e)
            })),
        )
    })?;

    let role = parse_role(&role_str);
    let user_uuid = Uuid::parse_str(&id).unwrap_or_else(|_| Uuid::new_v4());

    let access_token = generate_jwt(
        user_uuid,
        &email,
        &role,
        &state.jwt_secret,
        state.jwt_expiration_hours,
    )
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "error": format!("JWT generation failed: {}", e)
            })),
        )
    })?;

    let new_refresh_token = generate_refresh_token();
    let refresh_expires = Utc::now() + Duration::days(state.refresh_token_expiration_days);

    // Revoke old token and insert new one
    conn.execute(
        "UPDATE refresh_tokens SET revoked_at = ? WHERE token_hash = ?",
        [Utc::now().to_rfc3339(), payload.refresh_token.clone()],
    )
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "error": format!("Failed to revoke old token: {}", e)
            })),
        )
    })?;

    conn.execute(
        "INSERT INTO refresh_tokens (id, user_id, token_hash, expires_at, created_at) VALUES (?, ?, ?, ?, ?)",
        [
            Uuid::new_v4().to_string(),
            user_id,
            new_refresh_token.clone(),
            refresh_expires.to_rfc3339(),
            Utc::now().to_rfc3339(),
        ],
    )
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "error": format!("Failed to store refresh token: {}", e)
            })),
        )
    })?;

    Ok(Json(AuthResponse {
        access_token,
        refresh_token: new_refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: state.jwt_expiration_hours * 3600,
        user: UserResponse {
            id: user_uuid,
            email,
            name,
            role,
        },
    }))
}

pub async fn logout(
    State(state): State<AuthState>,
    Json(payload): Json<RefreshTokenRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let conn = state.db.conn().await;
    conn.execute(
        "UPDATE refresh_tokens SET revoked_at = ? WHERE token_hash = ?",
        [Utc::now().to_rfc3339(), payload.refresh_token],
    )
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "error": format!("Failed to revoke token: {}", e)
            })),
        )
    })?;

    Ok(Json(json!({
        "success": true,
        "message": "Logged out successfully"
    })))
}
