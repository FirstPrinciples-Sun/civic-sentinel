use axum::{
    extract::{Extension, Json, Path, Query, State},
    http::{HeaderMap, StatusCode},
};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde_json::{json, Value};
use uuid::Uuid;
use chrono::{Duration, Utc};
use sha2::{Digest, Sha256};

use crate::middleware::auth::{Claims, UserRole};
use crate::models::*;
use crate::services::rule_based_analyzer::RuleBasedAnalyzer;
use crate::AppState;

fn hash_phone_verification_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn verification_state_from_score(score: i32) -> VerificationState {
    if score >= 70 {
        VerificationState::Trusted
    } else if score >= 40 {
        VerificationState::NeedsReview
    } else {
        VerificationState::Suspicious
    }
}

fn calculate_verification_score(
    has_media: bool,
    has_address: bool,
    identity_verified: bool,
    account_age_days: i64,
    resolved_count: i64,
    no_reopen_history: bool,
    has_duplicate_signal: bool,
) -> i32 {
    let mut score = 0;

    if has_media {
        score += 35;
    }
    if has_address {
        score += 15;
    }
    if identity_verified {
        score += 15;
    }
    if account_age_days > 7 {
        score += 10;
    }
    if has_duplicate_signal {
        score += 15;
    }
    if resolved_count >= 2 && no_reopen_history {
        score += 10;
    }

    score.clamp(0, 100)
}

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
    headers: HeaderMap,
    Json(payload): Json<CreateIssueRequest>,
) -> (StatusCode, Json<Value>) {
    let CreateIssueRequest {
        title,
        description,
        category,
        location,
        media_urls,
        tags,
        otp_phone,
        otp_token,
    } = payload;

    let media_urls = media_urls.unwrap_or_default();
    if media_urls.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "success": false,
                "error": "At least one photo is required to submit an issue."
            })),
        );
    }

    let category = category.unwrap_or_else(|| {
        RuleBasedAnalyzer::classify_category(&title, &description)
    });
    let priority = RuleBasedAnalyzer::score_priority(&title, &description);
    let now = Utc::now();

    // Optionally extract reporter_id from JWT if Authorization header is present
    let reporter_id = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .and_then(|token| {
            let validation = Validation::new(Algorithm::HS256);
            let key = DecodingKey::from_secret(state.config.auth.jwt_secret.as_bytes());
            decode::<Claims>(token, &key, &validation)
                .ok()
                .map(|data| data.claims)
        })
        .and_then(|claims| Uuid::parse_str(&claims.sub).ok());

    let mut issue = Issue {
        id: Uuid::new_v4(),
        title,
        description,
        category,
        priority,
        status: IssueStatus::Reported,
        location,
        reporter_id,
        assigned_to: None,
        media_urls,
        tags: tags.unwrap_or_default(),
        verification_score: 35,
        verification_state: VerificationState::NeedsReview,
        duplicate_of: None,
        corroboration_count: 0,
        triage_score: 0,
        created_at: now,
        updated_at: now,
        resolved_at: None,
        deleted_at: None,
    };

    let otp_verified = match (otp_phone.as_deref(), otp_token.as_deref()) {
        (Some(phone), Some(token)) => {
            let token_hash = hash_phone_verification_token(token);
            match state
                .db
                .consume_phone_verification_token(phone, &token_hash)
                .await
            {
                Ok(true) => true,
                Ok(false) => {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(json!({
                            "success": false,
                            "error": "Invalid or expired OTP verification token."
                        })),
                    );
                }
                Err(e) => {
                    tracing::error!("Failed to validate OTP verification token: {}", e);
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({
                            "success": false,
                            "error": "Failed to validate OTP verification token"
                        })),
                    );
                }
            }
        }
        (None, None) => false,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "Both otp_phone and otp_token are required together."
                })),
            );
        }
    };

    let active_since = now - Duration::days(30);
    let duplicate_candidates = match state
        .db
        .list_duplicate_candidates(
            issue.location.latitude,
            issue.location.longitude,
            0.002,
            active_since,
        )
        .await
    {
        Ok(candidates) => candidates,
        Err(e) => {
            tracing::error!("Failed to load duplicate candidates: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "success": false,
                    "error": "Failed to evaluate duplicate issue candidates"
                })),
            );
        }
    };

    let duplicate_match_id = RuleBasedAnalyzer::detect_duplicate(&issue, &duplicate_candidates);
    let duplicate_root_id = duplicate_match_id.and_then(|match_id| {
        duplicate_candidates
            .iter()
            .find(|candidate| candidate.id == match_id)
            .map(|candidate| candidate.duplicate_of.unwrap_or(candidate.id))
    });
    issue.duplicate_of = duplicate_root_id;

    let (resolved_count, account_age_days, no_reopen_history, phone_verified) =
        if let Some(reporter_id) = issue.reporter_id {
            match state.db.get_reporter_trust_signals(reporter_id).await {
                Ok(signals) => signals,
                Err(e) => {
                    tracing::error!("Failed to load reporter trust signals: {}", e);
                    (0, 0, false, false)
                }
            }
        } else {
            (0, 0, false, false)
        };

    let identity_verified = issue.reporter_id.is_some() || otp_verified || phone_verified;
    let has_duplicate_signal = issue.duplicate_of.is_some();
    issue.verification_score = calculate_verification_score(
        !issue.media_urls.is_empty(),
        issue
            .location
            .address
            .as_ref()
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false),
        identity_verified,
        account_age_days,
        resolved_count,
        no_reopen_history,
        has_duplicate_signal,
    );
    issue.verification_state = verification_state_from_score(issue.verification_score);
    issue.triage_score = crate::db::Database::calculate_triage_score(
        &issue.priority,
        issue.verification_score,
        issue.corroboration_count,
        issue.created_at,
        now,
    );

    match state.db.insert_issue(&issue).await {
        Ok(_) => {
            if let Some(root_id) = issue.duplicate_of {
                if let Err(e) = state.db.increment_corroboration_count(root_id).await {
                    tracing::error!("Failed to increment corroboration count for {}: {}", root_id, e);
                } else if let Ok(Some(root_issue)) = state.db.get_issue(root_id).await {
                    let root_has_duplicate_signal = root_issue.corroboration_count > 0
                        || root_issue.duplicate_of.is_some();
                    let (root_resolved_count, root_account_age_days, root_no_reopen_history, root_phone_verified) =
                        if let Some(root_reporter_id) = root_issue.reporter_id {
                            state
                                .db
                                .get_reporter_trust_signals(root_reporter_id)
                                .await
                                .unwrap_or((0, 0, false, false))
                        } else {
                            (0, 0, false, false)
                        };
                    let root_identity_verified = root_issue.reporter_id.is_some() || root_phone_verified;
                    let root_verification_score = calculate_verification_score(
                        !root_issue.media_urls.is_empty(),
                        root_issue
                            .location
                            .address
                            .as_ref()
                            .map(|value| !value.trim().is_empty())
                            .unwrap_or(false),
                        root_identity_verified,
                        root_account_age_days,
                        root_resolved_count,
                        root_no_reopen_history,
                        root_has_duplicate_signal,
                    );
                    let root_verification_state = verification_state_from_score(root_verification_score);
                    let root_triage_score = crate::db::Database::calculate_triage_score(
                        &root_issue.priority,
                        root_verification_score,
                        root_issue.corroboration_count,
                        root_issue.created_at,
                        Utc::now(),
                    );
                    if let Err(e) = state
                        .db
                        .update_issue_scores(
                            root_id,
                            root_verification_score,
                            root_verification_state,
                            root_triage_score,
                        )
                        .await
                    {
                        tracing::error!("Failed to refresh root issue score {}: {}", root_id, e);
                    }
                }
            }

            (
                StatusCode::CREATED,
                Json(json!({
                    "success": true,
                    "data": issue,
                    "message": "Issue reported successfully."
                })),
            )
        }
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
            media_urls: Some(vec!["/uploads/test-photo.jpg".to_string()]),
            tags: None,
            otp_phone: None,
            otp_token: None,
        };

        let (status, Json(res_val)) = create_issue(
            State(state.clone()),
            axum::http::HeaderMap::new(),
            Json(create_payload),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);
        assert!(res_val["success"].as_bool().unwrap());
        let created_id_str = res_val["data"]["id"].as_str().unwrap();
        let created_id = Uuid::parse_str(created_id_str).unwrap();

        // 2. Test list issues
        let list_query = IssueListQuery {
            status: None,
            category: None,
            priority: None,
            verification_state: None,
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
