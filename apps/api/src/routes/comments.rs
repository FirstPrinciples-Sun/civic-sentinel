use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::db::Database;
use crate::models::*;

pub async fn create_comment(
    State(db): State<Database>,
    Json(payload): Json<CreateCommentRequest>,
) -> (StatusCode, Json<Value>) {
    // Verify the issue exists
    match db.get_issue(payload.issue_id).await {
        Ok(Some(_)) => {}
        Ok(None) => {
            return (StatusCode::NOT_FOUND, Json(json!({
                "success": false,
                "error": "Issue not found"
            })));
        }
        Err(e) => {
            tracing::error!("Failed to verify issue for comment: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                "success": false,
                "error": "Failed to verify issue"
            })));
        }
    }

    let comment = Comment {
        id: Uuid::new_v4(),
        issue_id: payload.issue_id,
        author_id: payload.author_id,
        content: payload.content,
        is_internal: payload.is_internal.unwrap_or(false),
        created_at: chrono::Utc::now(),
    };

    match db.insert_comment(&comment).await {
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
                "error": "Failed to create comment"
            })))
        }
    }
}

pub async fn get_comments(
    State(db): State<Database>,
    Path(issue_id): Path<Uuid>,
) -> (StatusCode, Json<Value>) {
    // Verify the issue exists
    match db.get_issue(issue_id).await {
        Ok(Some(_)) => {}
        Ok(None) => {
            return (StatusCode::NOT_FOUND, Json(json!({
                "success": false,
                "error": "Issue not found"
            })));
        }
        Err(e) => {
            tracing::error!("Failed to verify issue for comments: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                "success": false,
                "error": "Failed to verify issue"
            })));
        }
    }

    match db.get_comments(issue_id).await {
        Ok(comments) => {
            (StatusCode::OK, Json(json!({
                "success": true,
                "data": comments,
                "meta": {
                    "total": comments.len()
                }
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
