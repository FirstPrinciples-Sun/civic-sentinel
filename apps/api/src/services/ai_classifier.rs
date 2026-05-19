/// Issue classification service
/// Uses simple keyword matching for categorization
/// WebAssembly module provides browser-side analysis

use crate::models::{Issue, IssueCategory, Priority};

pub struct AIClassifier;

impl AIClassifier {
    /// Classify issue category from title and description
    pub fn classify_category(title: &str, description: &str) -> IssueCategory {
        let text = format!("{} {}", title, description).to_lowercase();
        
        if text.contains("road") || text.contains("bridge") || text.contains("sidewalk") || text.contains("pothole") {
            IssueCategory::Infrastructure
        } else if text.contains("flood") || text.contains("trash") || text.contains("pollution") || text.contains("tree") {
            IssueCategory::Environment
        } else if text.contains("light") || text.contains("water") || text.contains("electric") || text.contains("pipe") {
            IssueCategory::PublicUtility
        } else if text.contains("accident") || text.contains("dangerous") || text.contains("unsafe") || text.contains("crime") {
            IssueCategory::Safety
        } else if text.contains("bus") || text.contains("traffic") || text.contains("sign") {
            IssueCategory::Transportation
        } else {
            IssueCategory::Other
        }
    }

    /// Score urgency based on keywords
    pub fn score_priority(title: &str, description: &str) -> Priority {
        let text = format!("{} {}", title, description).to_lowercase();
        let critical_keywords = ["emergency", "urgent", "dangerous", "life-threatening", "fire", "flood"];
        let high_keywords = ["broken", "leaking", "unsafe", "blocked", "outage"];
        
        if critical_keywords.iter().any(|kw| text.contains(kw)) {
            Priority::Critical
        } else if high_keywords.iter().any(|kw| text.contains(kw)) {
            Priority::High
        } else if text.len() > 200 {
            Priority::Medium
        } else {
            Priority::Low
        }
    }

    /// Detect duplicate issues based on location proximity and similarity
    pub fn detect_duplicate(new_issue: &Issue, existing_issues: &[Issue]) -> Option<uuid::Uuid> {
        for existing in existing_issues {
            let distance = Self::haversine_distance(
                new_issue.location.latitude,
                new_issue.location.longitude,
                existing.location.latitude,
                existing.location.longitude,
            );
            
            if distance < 100.0 { // Within 100 meters
                let similarity = Self::text_similarity(&new_issue.title, &existing.title);
                if similarity > 0.8 {
                    return Some(existing.id);
                }
            }
        }
        None
    }

    fn haversine_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
        let r = 6371.0; // Earth's radius in km
        let dlat = (lat2 - lat1).to_radians();
        let dlon = (lon2 - lon1).to_radians();
        let a = (dlat / 2.0).sin().powi(2)
            + lat1.to_radians().cos() * lat2.to_radians().cos() * (dlon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
        r * c * 1000.0 // Return meters
    }

    fn text_similarity(a: &str, b: &str) -> f64 {
        let a_words: std::collections::HashSet<String> = a.to_lowercase()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();
        let b_words: std::collections::HashSet<String> = b.to_lowercase()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();
        
        let intersection: std::collections::HashSet<_> = a_words.intersection(&b_words).collect();
        let union: std::collections::HashSet<_> = a_words.union(&b_words).collect();
        
        intersection.len() as f64 / union.len() as f64
    }
}
