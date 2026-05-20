use axum::{
    extract::{Json, Path, Query, State},
    http::StatusCode,
};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::models::*;
use crate::services::ai_classifier::AIClassifier;
use crate::AppState;

pub async fn list_issues(
    State(state): State<AppState>,
    Query(query): Query<IssueListQuery>,
) -> (StatusCode, Json<Value>) {
    match state.db.list_issues(&query).await {
        Ok((issues, total)) => {
            let page = query.page.unwrap_or(1);
            let limit = query.limit.unwrap_or(20);
            (
                StatusCode::OK,
                Json(json!({
                    "success": true,
                    "data": issues,
                    "meta": {
                        "total": total,
                        "page": page,
                        "per_page": limit,
                        "total_pages": (total as f64 / limit as f64).ceil() as u32
                    }
                })),
            )
        }
        Err(e) => {
            tracing::error!("Failed to list issues: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "success": false,
                    "error": "Failed to retrieve issues"
                })),
            )
        }
    }
}

pub async fn create_issue(
    State(state): State<AppState>,
    Json(payload): Json<CreateIssueRequest>,
) -> (StatusCode, Json<Value>) {
    let category = payload
        .category
        .unwrap_or_else(|| AIClassifier::classify_category(&payload.title, &payload.description));
    let priority = AIClassifier::score_priority(&payload.title, &payload.description);

    let issue = Issue {
        id: Uuid::new_v4(),
        title: payload.title,
        description: payload.description,
        category,
        priority,
        status: IssueStatus::Reported,
        location: payload.location,
        reporter_id: None,
        assigned_to: None,
        media_urls: payload.media_urls.unwrap_or_default(),
        tags: payload.tags.unwrap_or_default(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        resolved_at: None,
        deleted_at: None,
    };

    match state.db.insert_issue(&issue).await {
        Ok(_) => (
            StatusCode::CREATED,
            Json(json!({
                "success": true,
                "data": issue,
                "message": "Issue reported successfully."
            })),
        ),
        Err(e) => {
            tracing::error!("Failed to create issue: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "success": false,
                    "error": "Failed to create issue"
                })),
            )
        }
    }
}

pub async fn get_issue(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> (StatusCode, Json<Value>) {
    match state.db.get_issue(id).await {
        Ok(Some(issue)) => (
            StatusCode::OK,
            Json(json!({
                "success": true,
                "data": issue
            })),
        ),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "success": false,
                "error": "Issue not found",
                "id": id
            })),
        ),
        Err(e) => {
            tracing::error!("Failed to get issue: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "success": false,
                    "error": "Failed to retrieve issue"
                })),
            )
        }
    }
}

pub async fn update_issue(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateIssueRequest>,
) -> (StatusCode, Json<Value>) {
    // Fetch current issue
    let current = match state.db.get_issue(id).await {
        Ok(Some(issue)) => issue,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "success": false,
                    "error": "Issue not found",
                    "id": id
                })),
            );
        }
        Err(e) => {
            tracing::error!("Failed to fetch issue for update: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "success": false,
                    "error": "Failed to retrieve issue"
                })),
            );
        }
    };

    // Validate status transition if status is being updated
    if let Some(new_status) = &payload.status {
        let old_status = &current.status;
        if old_status != new_status {
            if !is_valid_status_transition(old_status, new_status, payload.reason.as_deref()) {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "success": false,
                        "error": format!("Invalid status transition from {:?} to {:?}. A reason is required.", old_status, new_status)
                    })),
                );
            }

            // Log status change history
            let history_entry = StatusHistoryEntry {
                id: Uuid::new_v4(),
                issue_id: id,
                old_status: old_status.clone(),
                new_status: new_status.clone(),
                changed_by: payload.assigned_to.unwrap_or_else(Uuid::nil),
                reason: payload.reason.clone(),
                created_at: chrono::Utc::now(),
            };

            if let Err(e) = state.db.insert_status_history(&history_entry).await {
                tracing::error!("Failed to log status history: {}", e);
            }
        }
    }

    match state.db.update_issue(id, &payload, &current).await {
        Ok(_) => {
            // Fetch updated issue
            match state.db.get_issue(id).await {
                Ok(Some(issue)) => (
                    StatusCode::OK,
                    Json(json!({
                        "success": true,
                        "data": issue,
                        "message": "Issue updated successfully"
                    })),
                ),
                _ => (
                    StatusCode::OK,
                    Json(json!({
                        "success": true,
                        "message": "Issue updated successfully"
                    })),
                ),
            }
        }
        Err(e) => {
            tracing::error!("Failed to update issue: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "success": false,
                    "error": "Failed to update issue"
                })),
            )
        }
    }
}

pub async fn delete_issue(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> (StatusCode, Json<Value>) {
    match state.db.get_issue(id).await {
        Ok(Some(_)) => {}
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "success": false,
                    "error": "Issue not found",
                    "id": id
                })),
            );
        }
        Err(e) => {
            tracing::error!("Failed to fetch issue for delete: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "success": false,
                    "error": "Failed to retrieve issue"
                })),
            );
        }
    }

    match state.db.delete_issue(id).await {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({
                "success": true,
                "message": "Issue deleted successfully",
                "id": id
            })),
        ),
        Err(e) => {
            tracing::error!("Failed to delete issue: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "success": false,
                    "error": "Failed to delete issue"
                })),
            )
        }
    }
}

/// Validate if a status transition is allowed.
/// Backward transitions generally require a reason.
fn is_valid_status_transition(from: &IssueStatus, to: &IssueStatus, reason: Option<&str>) -> bool {
    if from == to {
        return true;
    }

    let has_reason = reason.map(|r| !r.trim().is_empty()).unwrap_or(false);

    match (from, to) {
        // Forward progressions — always valid
        (IssueStatus::Reported, IssueStatus::UnderReview) => true,
        (IssueStatus::Reported, IssueStatus::Escalated) => true,
        (IssueStatus::UnderReview, IssueStatus::InProgress) => true,
        (IssueStatus::UnderReview, IssueStatus::Escalated) => true,
        (IssueStatus::InProgress, IssueStatus::Resolved) => true,
        (IssueStatus::InProgress, IssueStatus::Escalated) => true,
        (IssueStatus::Escalated, IssueStatus::InProgress) => true,
        (IssueStatus::Escalated, IssueStatus::Resolved) => true,
        (IssueStatus::Resolved, IssueStatus::Closed) => true,

        // Backward or reopen transitions — require reason
        (IssueStatus::Resolved, IssueStatus::Reported) => has_reason,
        (IssueStatus::Resolved, IssueStatus::UnderReview) => has_reason,
        (IssueStatus::Resolved, IssueStatus::InProgress) => has_reason,
        (IssueStatus::Closed, _) => has_reason,
        (IssueStatus::UnderReview, IssueStatus::Reported) => has_reason,
        (IssueStatus::InProgress, IssueStatus::Reported) => has_reason,
        (IssueStatus::InProgress, IssueStatus::UnderReview) => has_reason,
        (IssueStatus::Escalated, IssueStatus::Reported) => has_reason,
        (IssueStatus::Escalated, IssueStatus::UnderReview) => has_reason,

        _ => false,
    }
}
