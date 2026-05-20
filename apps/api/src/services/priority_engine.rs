/// Priority scoring engine
/// Combines AI classification with time-based escalation
use crate::models::{Issue, Priority};
use chrono::Utc;

pub struct PriorityEngine;

impl PriorityEngine {
    /// Calculate dynamic priority score (0-100)
    pub fn calculate_score(issue: &Issue) -> u8 {
        let base_score = match issue.priority {
            Priority::Critical => 80,
            Priority::High => 60,
            Priority::Medium => 40,
            Priority::Low => 20,
        };

        // Time decay: older issues get slight boost
        let hours_old = (Utc::now() - issue.created_at).num_hours();
        let time_boost = (hours_old * 2).min(20) as u8;

        // Category boost
        let category_boost = match issue.category {
            crate::models::IssueCategory::Safety => 10,
            crate::models::IssueCategory::PublicUtility => 5,
            _ => 0,
        };

        (base_score + time_boost + category_boost).min(100)
    }

    /// Determine if issue should be escalated
    pub fn should_escalate(issue: &Issue) -> bool {
        let hours_open = (Utc::now() - issue.created_at).num_hours();

        match issue.priority {
            Priority::Critical => hours_open > 2,
            Priority::High => hours_open > 24,
            Priority::Medium => hours_open > 72,
            Priority::Low => hours_open > 168, // 1 week
        }
    }
}
