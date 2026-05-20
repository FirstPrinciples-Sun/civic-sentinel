use axum::{
    extract::{Extension, Json, Path, Query, State},
    http::StatusCode,
};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::middleware::auth::{Claims, UserRole};
use crate::models::*;
use crate::services::rule_based_analyzer::RuleBasedAnalyzer;
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
    let category = payload.category.unwrap_or_else(|| {
        RuleBasedAnalyzer::classify_category(&payload.title, &payload.description)
    });
    let priority = RuleBasedAnalyzer::score_priority(&payload.title, &payload.description);

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
    Extension(claims): Extension<Claims>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateIssueRequest>,
) -> (StatusCode, Json<Value>) {
    // Role check: Admin or Responder only
    if !matches!(claims.role, UserRole::Admin | UserRole::Responder) {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({
                "success": false,
                "error": "Insufficient permissions. Admin or Responder role required."
            })),
        );
    }
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
    let mut status_changed = false;
    let mut history_entry = None;

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

            status_changed = true;
            let changed_by = Uuid::parse_str(&claims.sub).unwrap_or_else(|_| Uuid::nil());
            history_entry = Some(StatusHistoryEntry {
                id: Uuid::new_v4(),
                issue_id: id,
                old_status: old_status.clone(),
                new_status: new_status.clone(),
                changed_by,
                reason: payload.reason.clone(),
                created_at: chrono::Utc::now(),
            });
        }
    }

    match state.db.update_issue(id, &payload, &current).await {
        Ok(_) => {
            if status_changed {
                if let Some(entry) = history_entry {
                    if let Err(e) = state.db.insert_status_history(&entry).await {
                        tracing::error!("Failed to log status history: {}", e);
                    }
                }
            }
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
    Extension(claims): Extension<Claims>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> (StatusCode, Json<Value>) {
    // Role check: Admin only
    if claims.role != UserRole::Admin {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({
                "success": false,
                "error": "Insufficient permissions. Admin role required."
            })),
        );
    }
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
pub fn is_valid_status_transition(
    from: &IssueStatus,
    to: &IssueStatus,
    reason: Option<&str>,
) -> bool {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_status_transitions() {
        // Forward progressions always valid
        assert!(is_valid_status_transition(
            &IssueStatus::Reported,
            &IssueStatus::UnderReview,
            None
        ));
        assert!(is_valid_status_transition(
            &IssueStatus::UnderReview,
            &IssueStatus::InProgress,
            None
        ));
        assert!(is_valid_status_transition(
            &IssueStatus::InProgress,
            &IssueStatus::Resolved,
            None
        ));
        assert!(is_valid_status_transition(
            &IssueStatus::Resolved,
            &IssueStatus::Closed,
            None
        ));
    }

    #[test]
    fn test_backward_transitions_require_reason() {
        // Without reason: fail
        assert!(!is_valid_status_transition(
            &IssueStatus::Resolved,
            &IssueStatus::Reported,
            None
        ));
        assert!(!is_valid_status_transition(
            &IssueStatus::Closed,
            &IssueStatus::InProgress,
            None
        ));

        // With reason: pass
        assert!(is_valid_status_transition(
            &IssueStatus::Resolved,
            &IssueStatus::Reported,
            Some("Reopened due to complaint")
        ));
        assert!(is_valid_status_transition(
            &IssueStatus::Closed,
            &IssueStatus::UnderReview,
            Some("Investigating further")
        ));
    }

    #[test]
    fn test_same_status_is_always_valid() {
        assert!(is_valid_status_transition(
            &IssueStatus::InProgress,
            &IssueStatus::InProgress,
            None
        ));
    }

    #[test]
    fn test_invalid_transitions() {
        // These should always be false
        assert!(!is_valid_status_transition(
            &IssueStatus::Closed,
            &IssueStatus::Resolved,
            None
        ));
    }

    use crate::config::AppConfig;
    use crate::db::Database;
    use std::sync::Arc;

    async fn setup_test_state() -> AppState {
        let mut config = AppConfig::default();
        config.database.url = format!("sqlite://test-{}.db", Uuid::new_v4());
        let db = Database::new(&config).await.unwrap();
        db.migrate().await.unwrap();
        AppState {
            db,
            config: Arc::new(config),
        }
    }

    #[tokio::test]
    async fn test_create_and_list_issues_routes() {
        let state = setup_test_state().await;

        // 1. Test create issue
        let create_payload = CreateIssueRequest {
            title: "Road Pothole".to_string(),
            description: "Deep pothole on Main Road".to_string(),
            category: Some(IssueCategory::Infrastructure),
            location: GeoLocation {
                latitude: 13.7563,
                longitude: 100.5018,
                address: Some("Main Road".to_string()),
            },
            media_urls: None,
            tags: None,
        };

        let (status, Json(res_val)) =
            create_issue(State(state.clone()), Json(create_payload)).await;
        assert_eq!(status, StatusCode::CREATED);
        assert!(res_val["success"].as_bool().unwrap());
        let created_id_str = res_val["data"]["id"].as_str().unwrap();
        let created_id = Uuid::parse_str(created_id_str).unwrap();

        // 2. Test list issues
        let list_query = IssueListQuery {
            status: None,
            category: None,
            priority: None,
            sort: None,
            page: Some(1),
            limit: Some(10),
        };
        let (status, Json(list_val)) =
            list_issues(State(state.clone()), axum::extract::Query(list_query)).await;
        assert_eq!(status, StatusCode::OK);
        assert!(list_val["success"].as_bool().unwrap());
        let issues_array = list_val["data"].as_array().unwrap();
        assert_eq!(issues_array.len(), 1);
        assert_eq!(issues_array[0]["id"].as_str().unwrap(), created_id_str);

        // 3. Test get issue
        let (status, Json(get_val)) = get_issue(State(state.clone()), Path(created_id)).await;
        assert_eq!(status, StatusCode::OK);
        assert!(get_val["success"].as_bool().unwrap());
        assert_eq!(get_val["data"]["title"].as_str().unwrap(), "Road Pothole");

        // 4. Test update issue (requiring auth)
        let admin_id = Uuid::new_v4();
        state.db.conn().await.execute(
            "INSERT INTO users (id, email, password_hash, name, role, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            (
                admin_id.to_string(),
                "admin@civic.org".to_string(),
                "hashedpassword".to_string(),
                "Admin User".to_string(),
                "admin",
                chrono::Utc::now().to_rfc3339(),
                chrono::Utc::now().to_rfc3339(),
            )
        ).await.unwrap();

        // Admin claims
        let admin_claims = Claims {
            sub: admin_id.to_string(),
            email: "admin@civic.org".to_string(),
            role: UserRole::Admin,
            iat: chrono::Utc::now().timestamp(),
            exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp(),
        };

        let update_payload_1 = UpdateIssueRequest {
            title: None,
            description: None,
            status: Some(IssueStatus::UnderReview),
            priority: None,
            assigned_to: None,
            reason: Some("Reviewing pothole".to_string()),
        };

        let (status, Json(update_val_1)) = update_issue(
            Extension(admin_claims.clone()),
            State(state.clone()),
            Path(created_id),
            Json(update_payload_1),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert!(update_val_1["success"].as_bool().unwrap());
        assert_eq!(
            update_val_1["data"]["status"].as_str().unwrap(),
            "underreview"
        );

        let update_payload_2 = UpdateIssueRequest {
            title: None,
            description: None,
            status: Some(IssueStatus::InProgress),
            priority: None,
            assigned_to: None,
            reason: Some("Starting work".to_string()),
        };

        let (status, Json(update_val_2)) = update_issue(
            Extension(admin_claims.clone()),
            State(state.clone()),
            Path(created_id),
            Json(update_payload_2),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert!(update_val_2["success"].as_bool().unwrap());
        assert_eq!(
            update_val_2["data"]["status"].as_str().unwrap(),
            "inprogress"
        );

        // Verify status history was written
        let history = state.db.get_status_history(created_id).await.unwrap();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].new_status, IssueStatus::InProgress);
        assert_eq!(history[1].new_status, IssueStatus::UnderReview);

        // 5. Test delete issue (Admin role required)
        let reporter_claims = Claims {
            sub: Uuid::new_v4().to_string(),
            email: "user@civic.org".to_string(),
            role: UserRole::Reporter,
            iat: chrono::Utc::now().timestamp(),
            exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp(),
        };

        // Try deleting as reporter -> Forbidden
        let (status, _) = delete_issue(
            Extension(reporter_claims),
            State(state.clone()),
            Path(created_id),
        )
        .await;
        assert_eq!(status, StatusCode::FORBIDDEN);

        // Delete as Admin -> Success
        let (status, Json(delete_val)) = delete_issue(
            Extension(admin_claims),
            State(state.clone()),
            Path(created_id),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert!(delete_val["success"].as_bool().unwrap());

        // Verify deleted issue is no longer returned in get
        let (status, _) = get_issue(State(state.clone()), Path(created_id)).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }
}
