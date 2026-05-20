use axum::{
    extract::{Extension, Json, Path, State},
    http::StatusCode,
};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::middleware::auth::{Claims, UserRole};
use crate::models::*;
use crate::AppState;

pub async fn get_comments(
    State(state): State<AppState>,
    Path(issue_id): Path<Uuid>,
) -> (StatusCode, Json<Value>) {
    match state.db.get_comments(issue_id).await {
        Ok(comments) => (
            StatusCode::OK,
            Json(json!({
                "success": true,
                "data": comments
            })),
        ),
        Err(e) => {
            tracing::error!("Failed to get comments: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "success": false,
                    "error": "Failed to retrieve comments"
                })),
            )
        }
    }
}

pub async fn create_comment(
    Extension(claims): Extension<Claims>,
    State(state): State<AppState>,
    Path(issue_id): Path<Uuid>,
    Json(payload): Json<CreateCommentRequest>,
) -> (StatusCode, Json<Value>) {
    // Role check: Admin, Responder, or Reporter
    if !matches!(
        claims.role,
        UserRole::Admin | UserRole::Responder | UserRole::Reporter
    ) {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({
                "success": false,
                "error": "Insufficient permissions. Admin, Responder, or Reporter role required."
            })),
        );
    }

    let trimmed = payload.content.trim();
    if trimmed.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "success": false,
                "error": "Comment content cannot be empty"
            })),
        );
    }

    let author_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "success": false,
                    "error": "Invalid authentication context"
                })),
            )
        }
    };

    // Internal comments are reserved for admins/responders.
    let is_internal = if matches!(claims.role, UserRole::Admin | UserRole::Responder) {
        payload.is_internal.unwrap_or(false)
    } else {
        false
    };

    let comment = Comment {
        id: Uuid::new_v4(),
        issue_id,
        author_id,
        content: trimmed.to_string(),
        is_internal,
        created_at: chrono::Utc::now(),
    };

    match state.db.insert_comment(&comment).await {
        Ok(_) => (
            StatusCode::CREATED,
            Json(json!({
                "success": true,
                "data": comment,
                "message": "Comment added successfully"
            })),
        ),
        Err(e) => {
            tracing::error!("Failed to create comment: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "success": false,
                    "error": "Failed to add comment"
                })),
            )
        }
    }
}
