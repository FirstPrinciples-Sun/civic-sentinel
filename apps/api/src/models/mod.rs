use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub category: IssueCategory,
    pub priority: Priority,
    pub status: IssueStatus,
    pub location: GeoLocation,
    pub reporter_id: Option<Uuid>,
    pub assigned_to: Option<Uuid>,
    pub media_urls: Vec<String>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoLocation {
    pub latitude: f64,
    pub longitude: f64,
    pub address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum IssueCategory {
    Infrastructure,
    Safety,
    Environment,
    Sanitation,
    Transportation,
    PublicUtility,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum IssueStatus {
    Reported,
    UnderReview,
    InProgress,
    Resolved,
    Closed,
    Escalated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateIssueRequest {
    pub title: String,
    pub description: String,
    pub category: IssueCategory,
    pub location: GeoLocation,
    pub media_urls: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsSummary {
    pub total_issues: i64,
    pub open_issues: i64,
    pub resolved_issues: i64,
    pub avg_resolution_hours: f64,
    pub issues_by_category: Vec<CategoryCount>,
    pub issues_by_priority: Vec<PriorityCount>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryCount {
    pub category: IssueCategory,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorityCount {
    pub priority: Priority,
    pub count: i64,
}
