use serde::{Deserialize, Serialize};

/// Search filters from the frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFilters {
    pub project_ids: Option<Vec<String>>,
    pub tiers: Option<Vec<String>>,
    pub status: Option<Vec<String>>, // "missing_in_production", "empty_value", "localhost_detected"
}

/// Fuzzy match score for a query against a variable key.
/// Returns a score 0-100 where 100 is an exact match.
pub fn fuzzy_score(query: &str, key: &str) -> u32 {
    let query_upper = query.to_uppercase();
    let key_upper = key.to_uppercase();

    // Exact match
    if key_upper == query_upper {
        return 100;
    }

    // Starts with
    if key_upper.starts_with(&query_upper) {
        return 90;
    }

    // Contains
    if key_upper.contains(&query_upper) {
        return 70;
    }

    // Fuzzy: check if all chars of query appear in order in key
    let mut key_chars = key_upper.chars().peekable();
    let mut matched = 0;
    let query_chars: Vec<char> = query_upper.chars().collect();

    for qc in &query_chars {
        while let Some(&kc) = key_chars.peek() {
            key_chars.next();
            if kc == *qc {
                matched += 1;
                break;
            }
        }
    }

    if matched == query_chars.len() {
        50
    } else if matched > 0 {
        (matched as u32 * 30) / query_chars.len() as u32
    } else {
        0
    }
}
