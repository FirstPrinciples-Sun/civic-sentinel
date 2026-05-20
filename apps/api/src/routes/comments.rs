use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::models::*;
use crate::AppState;

pub async fn get_comments(
    State(state): State<AppState>,
    Path(issue_id): Path<Uuid>,
) -> (StatusCode, Json<Value>) {
    match state.db.get_comments(issue_id).await {
        Ok(comments) => {
            (StatusCode::OK, Json(json!({
                "success": true,
                "data": comments
            })))
        }
        Err(e) => {
            tracing::error!("Failed to get comments: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                "success": false,
                "error": "Failed to retrieve comments"
            })))
        }
    }
}

pub async fn create_comment(
    State(state): State<AppState>,
    Path(issue_id): Path<Uuid>,
    Json(payload): Json<CreateCommentRequest>,
) -> (StatusCode, Json<Value>) {
    let comment = Comment {
        id: Uuid::new_v4(),
        issue_id,
        author_id: payload.author_id,
        content: payload.content,
        is_internal: payload.is_internal.unwrap_or(false),
        created_at: chrono::Utc::now(),
    };

    match state.db.insert_comment(&comment).await {
        Ok(_) => {
            (StatusCode::CREATED, Json(json!({
                "success": true,
                "data": comment,
                "message": "Comment added successfully"
            })))
        }
        Err(e) => {
            tracing::error!("Failed to create comment: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                "success": false,
                "error": "Failed to add comment"
            })))
        }
    }
}
