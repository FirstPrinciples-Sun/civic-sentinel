use axum::{extract::{Json, State}, http::StatusCode};
use serde_json::{json, Value};

use crate::AppState;

pub async fn get_analytics(
    State(state): State<AppState>,
) -> (StatusCode, Json<Value>) {
    match state.db.get_analytics_summary().await {
        Ok(summary) => {
            let impact_score = if summary.total_issues > 0 {
                (summary.resolved_issues as f64 / summary.total_issues as f64) * 100.0
            } else {
                0.0
            };

            (StatusCode::OK, Json(json!({
                "success": true,
                "data": {
                    "total_issues": summary.total_issues,
                    "open_issues": summary.open_issues,
                    "resolved_issues": summary.resolved_issues,
                    "avg_resolution_hours": summary.avg_resolution_hours,
                    "issues_by_category": summary.issues_by_category,
                    "issues_by_priority": summary.issues_by_priority,
                    "impact_score": impact_score
                }
            })))
        }
        Err(e) => {
            tracing::error!("Failed to get analytics: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                "success": false,
                "error": "Failed to retrieve analytics"
            })))
        }
    }
}
