pub mod user;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
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
    pub deleted_at: Option<DateTime<Utc>>,
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

impl IssueCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            IssueCategory::Infrastructure => "infrastructure",
            IssueCategory::Safety => "safety",
            IssueCategory::Environment => "environment",
            IssueCategory::Sanitation => "sanitation",
            IssueCategory::Transportation => "transportation",
            IssueCategory::PublicUtility => "publicutility",
            IssueCategory::Other => "other",
        }
    }
}

impl FromStr for IssueCategory {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "infrastructure" => Ok(IssueCategory::Infrastructure),
            "safety" => Ok(IssueCategory::Safety),
            "environment" => Ok(IssueCategory::Environment),
            "sanitation" => Ok(IssueCategory::Sanitation),
            "transportation" => Ok(IssueCategory::Transportation),
            "publicutility" => Ok(IssueCategory::PublicUtility),
            "other" => Ok(IssueCategory::Other),
            _ => Err(format!("Unknown issue category: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd, Ord, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

impl Priority {
    pub fn as_str(&self) -> &'static str {
        match self {
            Priority::Low => "low",
            Priority::Medium => "medium",
            Priority::High => "high",
            Priority::Critical => "critical",
        }
    }
}

impl FromStr for Priority {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "low" => Ok(Priority::Low),
            "medium" => Ok(Priority::Medium),
            "high" => Ok(Priority::High),
            "critical" => Ok(Priority::Critical),
            _ => Err(format!("Unknown priority: {}", s)),
        }
    }
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

impl IssueStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            IssueStatus::Reported => "reported",
            IssueStatus::UnderReview => "underreview",
            IssueStatus::InProgress => "inprogress",
            IssueStatus::Resolved => "resolved",
            IssueStatus::Closed => "closed",
            IssueStatus::Escalated => "escalated",
        }
    }
}

impl FromStr for IssueStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "reported" => Ok(IssueStatus::Reported),
            "underreview" => Ok(IssueStatus::UnderReview),
            "inprogress" => Ok(IssueStatus::InProgress),
            "resolved" => Ok(IssueStatus::Resolved),
            "closed" => Ok(IssueStatus::Closed),
            "escalated" => Ok(IssueStatus::Escalated),
            _ => Err(format!("Unknown issue status: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateIssueRequest {
    pub title: String,
    pub description: String,
    pub category: Option<IssueCategory>,
    pub location: GeoLocation,
    pub media_urls: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateIssueRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<IssueStatus>,
    pub priority: Option<Priority>,
    pub assigned_to: Option<Uuid>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IssueListQuery {
    pub status: Option<IssueStatus>,
    pub category: Option<IssueCategory>,
    pub priority: Option<Priority>,
    pub sort: Option<String>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub id: Uuid,
    pub issue_id: Uuid,
    pub author_id: Uuid,
    pub content: String,
    pub is_internal: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCommentRequest {
    pub issue_id: Uuid,
    pub author_id: Uuid,
    pub content: String,
    pub is_internal: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusHistoryEntry {
    pub id: Uuid,
    pub issue_id: Uuid,
    pub old_status: IssueStatus,
    pub new_status: IssueStatus,
    pub changed_by: Uuid,
    pub reason: Option<String>,
    pub created_at: DateTime<Utc>,
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
