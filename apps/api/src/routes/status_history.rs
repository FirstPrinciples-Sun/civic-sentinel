use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::db::Database;

pub async fn get_status_history(
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
            tracing::error!("Failed to verify issue for status history: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                "success": false,
                "error": "Failed to verify issue"
            })));
        }
    }

    match db.get_status_history(issue_id).await {
        Ok(history) => {
            (StatusCode::OK, Json(json!({
                "success": true,
                "data": history,
                "meta": {
                    "total": history.len()
                }
            })))
        }
        Err(e) => {
            tracing::error!("Failed to get status history: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                "success": false,
                "error": "Failed to retrieve status history"
            })))
        }
    }
}
