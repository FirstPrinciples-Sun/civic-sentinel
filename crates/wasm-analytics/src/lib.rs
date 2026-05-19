use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};

/// WebAssembly module for privacy-preserving analytics
/// Runs ML inference directly in the browser — no data leaves the device

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[derive(Serialize, Deserialize)]
pub struct IssueAnalysis {
    pub priority_score: u8,
    pub category: String,
    pub urgency_keywords: Vec<String>,
    pub sentiment: String,
}

/// Analyze issue text locally in the browser
/// Returns priority score (0-100) and detected category
#[wasm_bindgen]
pub fn analyze_issue(title: &str, description: &str) -> String {
    let text = format!("{} {}", title, description).to_lowercase();
    
    // Priority scoring
    let critical_keywords = ["emergency", "flood", "fire", "dangerous", "life-threatening"];
    let high_keywords = ["broken", "leaking", "unsafe", "blocked", "outage", "accident"];
    let medium_keywords = ["damaged", "missing", "noisy", "slow", "dirty"];
    
    let mut score = 20; // Base score
    let mut urgency_keywords = Vec::new();
    
    for kw in &critical_keywords {
        if text.contains(kw) {
            score += 40;
            urgency_keywords.push(kw.to_string());
        }
    }
    
    for kw in &high_keywords {
        if text.contains(kw) {
            score += 25;
            urgency_keywords.push(kw.to_string());
        }
    }
    
    for kw in &medium_keywords {
        if text.contains(kw) {
            score += 10;
            urgency_keywords.push(kw.to_string());
        }
    }
    
    score = score.min(100);
    
    // Category detection
    let category = if text.contains("road") || text.contains("bridge") || text.contains("sidewalk") {
        "infrastructure"
    } else if text.contains("flood") || text.contains("trash") || text.contains("pollution") || text.contains("tree") {
        "environment"
    } else if text.contains("light") || text.contains("water") || text.contains("electric") {
        "public_utility"
    } else if text.contains("accident") || text.contains("dangerous") || text.contains("crime") {
        "safety"
    } else if text.contains("bus") || text.contains("traffic") || text.contains("sign") {
        "transportation"
    } else {
        "other"
    };
    
    // Sentiment analysis (simplified)
    let sentiment = if score > 70 {
        "urgent"
    } else if score > 40 {
        "concerned"
    } else {
        "neutral"
    };
    
    let analysis = IssueAnalysis {
        priority_score: score,
        category: category.to_string(),
        urgency_keywords,
        sentiment: sentiment.to_string(),
    };
    
    serde_json::to_string(&analysis).unwrap_or_default()
}

/// Calculate text similarity between two issues (Jaccard index)
#[wasm_bindgen]
pub fn text_similarity(text_a: &str, text_b: &str) -> f64 {
    let a_words: std::collections::HashSet<String> = text_a.to_lowercase()
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();
    
    let b_words: std::collections::HashSet<String> = text_b.to_lowercase()
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();
    
    let intersection: std::collections::HashSet<_> = a_words.intersection(&b_words).collect();
    let union: std::collections::HashSet<_> = a_words.union(&b_words).collect();
    
    if union.is_empty() {
        0.0
    } else {
        intersection.len() as f64 / union.len() as f64
    }
}

/// Batch analyze multiple issues (returns JSON array)
#[wasm_bindgen]
pub fn batch_analyze(issues_json: &str) -> String {
    #[derive(Deserialize)]
    struct IssueInput {
        id: String,
        title: String,
        description: String,
    }
    
    let issues: Vec<IssueInput> = match serde_json::from_str(issues_json) {
        Ok(v) => v,
        Err(_) => return "[]".to_string(),
    };
    
    let results: Vec<IssueAnalysis> = issues.into_iter().map(|issue| {
        let text = format!("{} {}", issue.title, issue.description).to_lowercase();
        let mut score = 20u8;
        let mut urgency_keywords = Vec::new();
        
        for kw in ["emergency", "flood", "fire", "dangerous"] {
            if text.contains(kw) {
                score += 40;
                urgency_keywords.push(kw.to_string());
            }
        }
        
        score = score.min(100);
        
        IssueAnalysis {
            priority_score: score,
            category: "unknown".to_string(),
            urgency_keywords,
            sentiment: if score > 60 { "urgent" } else { "neutral" }.to_string(),
        }
    }).collect();
    
    serde_json::to_string(&results).unwrap_or_default()
}
