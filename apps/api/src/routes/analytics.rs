use axum::{http::StatusCode, response::Json};
use serde_json::{json, Value};

pub async fn get_analytics() -> (StatusCode, Json<Value>) {
    // TODO: Calculate from database
    (StatusCode::OK, Json(json!({
        "success": true,
        "data": {
            "total_issues": 0,
            "open_issues": 0,
            "resolved_issues": 0,
            "avg_resolution_hours": 0.0,
            "issues_by_category": [],
            "issues_by_priority": [],
            "impact_score": 0.0
        }
    })))
}
