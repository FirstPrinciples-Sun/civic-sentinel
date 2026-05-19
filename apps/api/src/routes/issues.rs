use axum::{
    extract::Path,
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::models::*;

pub async fn list_issues() -> (StatusCode, Json<Value>) {
    // TODO: Connect to database
    let issues: Vec<Issue> = vec![];
    (StatusCode::OK, Json(json!({
        "success": true,
        "data": issues,
        "meta": {
            "total": 0,
            "page": 1,
            "per_page": 20
        }
    })))
}

pub async fn create_issue(Json(payload): Json<CreateIssueRequest>) -> (StatusCode, Json<Value>) {
    let issue = Issue {
        id: Uuid::new_v4(),
        title: payload.title,
        description: payload.description,
        category: payload.category,
        priority: Priority::Medium, // AI will re-prioritize
        status: IssueStatus::Reported,
        location: payload.location,
        reporter_id: None,
        assigned_to: None,
        media_urls: payload.media_urls.unwrap_or_default(),
        tags: payload.tags.unwrap_or_default(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        resolved_at: None,
    };

    // TODO: Save to database

    (StatusCode::CREATED, Json(json!({
        "success": true,
        "data": issue,
        "message": "Issue reported successfully. Our AI is analyzing priority."
    })))
}

pub async fn get_issue(Path(id): Path<Uuid>) -> (StatusCode, Json<Value>) {
    // TODO: Fetch from database
    (StatusCode::NOT_FOUND, Json(json!({
        "success": false,
        "error": "Issue not found",
        "id": id
    })))
}
