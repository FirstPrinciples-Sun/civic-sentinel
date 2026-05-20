use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::AppState;

pub async fn get_status_history(
    State(state): State<AppState>,
    Path(issue_id): Path<Uuid>,
) -> (StatusCode, Json<Value>) {
    match state.db.get_status_history(issue_id).await {
        Ok(history) => {
            (StatusCode::OK, Json(json!({
                "success": true,
                "data": history
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
