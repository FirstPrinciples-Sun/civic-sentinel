use axum::{extract::State, http::StatusCode, response::Json};
use serde_json::{json, Value};

use crate::error::AppError;
use crate::models::{IssueCategory, Priority};
use crate::AppState;

fn parse_enum<T: serde::de::DeserializeOwned>(s: &str) -> Result<T, AppError> {
    serde_json::from_value(Value::String(s.to_string()))
        .map_err(|_| AppError::Internal(format!("Invalid enum value: {}", s)))
}

/// Get analytics summary with real database counts
pub async fn get_analytics(
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    let conn = state.db.conn().await;

    // Total issues
    let mut total_rows = conn
        .query("SELECT COUNT(*) as total FROM issues", ())
        .await?;
    let total_issues: i64 = if let Some(row) = total_rows.next().await? {
        row.get("total")?
    } else {
        0
    };

    // Resolved issues
    let mut resolved_rows = conn
        .query(
            "SELECT COUNT(*) as count FROM issues WHERE status = 'resolved'",
            (),
        )
        .await?;
    let resolved_issues: i64 = if let Some(row) = resolved_rows.next().await? {
        row.get("count")?
    } else {
        0
    };

    // Closed issues (also considered done)
    let mut closed_rows = conn
        .query(
            "SELECT COUNT(*) as count FROM issues WHERE status = 'closed'",
            (),
        )
        .await?;
    let closed_issues: i64 = if let Some(row) = closed_rows.next().await? {
        row.get("count")?
    } else {
        0
    };

    // Open issues = total - resolved - closed
    let open_issues = total_issues - resolved_issues - closed_issues;

    // Average resolution time in hours
    let mut avg_rows = conn
        .query(
            "SELECT AVG(
                CAST((strftime('%s', resolved_at) - strftime('%s', created_at)) AS REAL) / 3600.0
            ) as avg_hours FROM issues WHERE resolved_at IS NOT NULL",
            (),
        )
        .await?;
    let avg_resolution_hours: f64 = if let Some(row) = avg_rows.next().await? {
        row.get::<&str, Option<f64>>("avg_hours")?.unwrap_or(0.0)
    } else {
        0.0
    };

    // Issues by category
    let mut cat_rows = conn
        .query(
            "SELECT category, COUNT(*) as count FROM issues GROUP BY category",
            (),
        )
        .await?;

    let mut issues_by_category = Vec::new();
    while let Some(row) = cat_rows.next().await? {
        let category_str: String = row.get("category")?;
        let category = parse_enum::<IssueCategory>(&category_str)?;
        let count: i64 = row.get("count")?;
        issues_by_category.push(json!({
            "category": category,
            "count": count
        }));
    }

    // Issues by priority
    let mut pri_rows = conn
        .query(
            "SELECT priority, COUNT(*) as count FROM issues GROUP BY priority",
            (),
        )
        .await?;

    let mut issues_by_priority = Vec::new();
    while let Some(row) = pri_rows.next().await? {
        let priority_str: String = row.get("priority")?;
        let priority = parse_enum::<Priority>(&priority_str)?;
        let count: i64 = row.get("count")?;
        issues_by_priority.push(json!({
            "priority": priority,
            "count": count
        }));
    }

    // Simple impact score: percentage of resolved + closed issues
    let impact_score = if total_issues > 0 {
        ((resolved_issues + closed_issues) as f64 / total_issues as f64) * 100.0
    } else {
        0.0
    };

    Ok((
        StatusCode::OK,
        Json(json!({
            "success": true,
            "data": {
                "total_issues": total_issues,
                "open_issues": open_issues,
                "resolved_issues": resolved_issues,
                "avg_resolution_hours": avg_resolution_hours,
                "issues_by_category": issues_by_category,
                "issues_by_priority": issues_by_priority,
                "impact_score": impact_score
            }
        })),
    ))
}
