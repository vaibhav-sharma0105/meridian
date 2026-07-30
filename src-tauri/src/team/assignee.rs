use rusqlite::Connection;
use crate::team::models::{TeamMember, AssigneeSuggestion, AssigneeFactors, AssigneeWeights};
use crate::team::repository;

pub fn get_assignee_suggestions(
    conn: &Connection,
    task_title: &str,
    task_description: Option<&str>,
    _project_id: Option<&str>,
) -> Result<Vec<AssigneeSuggestion>, String> {
    let members = repository::get_all_team_members(conn)?;
    let task_keywords = extract_keywords(task_title, task_description);

    if members.is_empty() {
        return fallback_suggestions_from_patterns(conn, &task_keywords);
    }

    let weights = get_learned_weights(conn)?;

    let mut suggestions: Vec<AssigneeSuggestion> = members
        .into_iter()
        .filter_map(|member| {
            let factors = calculate_factors(conn, &member, &task_keywords).ok()?;
            let score = calculate_combined_score(&factors, &weights);

            if score > 0.1 {
                Some(AssigneeSuggestion {
                    member,
                    score,
                    confidence: score_to_confidence(score),
                    reason: determine_primary_reason(&factors),
                    factors,
                })
            } else {
                None
            }
        })
        .collect();

    // Sort by score descending
    suggestions.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

    // Return top 5
    suggestions.truncate(5);

    Ok(suggestions)
}

/// Used when the team roster is empty: suggests assignees straight from the
/// smart_defaults keyword -> assignee patterns already learned from past
/// manual assignments, instead of returning no suggestions at all.
fn fallback_suggestions_from_patterns(
    conn: &Connection,
    task_keywords: &[String],
) -> Result<Vec<AssigneeSuggestion>, String> {
    #[derive(serde::Deserialize)]
    struct AssigneePatternRow {
        keyword: String,
        assignee: String,
        occurrence_count: i64,
    }
    #[derive(serde::Deserialize)]
    struct SmartDefaultsData {
        #[serde(default)]
        assignee_patterns: Vec<AssigneePatternRow>,
    }

    let mut stmt = conn
        .prepare("SELECT model_data FROM pattern_models WHERE pattern_type = 'smart_defaults'")
        .map_err(|e| e.to_string())?;
    let rows: Vec<String> = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    let mut totals: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
    let mut keyword_matches: std::collections::HashMap<String, i64> = std::collections::HashMap::new();

    for row in &rows {
        if let Ok(data) = serde_json::from_str::<SmartDefaultsData>(row) {
            for pattern in data.assignee_patterns {
                if pattern.assignee.is_empty() {
                    continue;
                }
                *totals.entry(pattern.assignee.clone()).or_insert(0) += pattern.occurrence_count;
                if task_keywords.iter().any(|kw| kw.eq_ignore_ascii_case(&pattern.keyword)) {
                    *keyword_matches.entry(pattern.assignee.clone()).or_insert(0) += pattern.occurrence_count;
                }
            }
        }
    }

    let mut suggestions: Vec<AssigneeSuggestion> = totals
        .into_iter()
        .map(|(assignee, total)| {
            let keyword_hits = keyword_matches.get(&assignee).copied().unwrap_or(0);
            let score = ((total as f64 / 10.0) + (keyword_hits as f64 / 5.0)).min(1.0);
            AssigneeSuggestion {
                member: TeamMember {
                    id: format!("pattern:{}", assignee),
                    name: assignee.clone(),
                    email: None,
                    avatar_url: None,
                    source: "pattern".to_string(),
                    source_id: None,
                    role: "member".to_string(),
                    expertise: None,
                    workload_score: None,
                    metadata: None,
                    last_synced_at: None,
                    created_at: String::new(),
                },
                score,
                confidence: score_to_confidence(score),
                reason: "Based on your past assignment patterns".to_string(),
                factors: AssigneeFactors {
                    pattern_score: score,
                    workload_score: 0.0,
                    expertise_score: 0.0,
                    recency_score: 0.0,
                },
            }
        })
        .filter(|s| s.score > 0.05)
        .collect();

    suggestions.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    suggestions.truncate(5);
    Ok(suggestions)
}

fn calculate_factors(
    conn: &Connection,
    member: &TeamMember,
    task_keywords: &[String],
) -> Result<AssigneeFactors, String> {
    let pattern_score = calculate_pattern_score(conn, member)?;
    let workload_score = calculate_workload_score(member);
    let expertise_score = calculate_expertise_score(member, task_keywords);
    let recency_score = calculate_recency_score(conn, member)?;

    Ok(AssigneeFactors {
        pattern_score,
        workload_score,
        expertise_score,
        recency_score,
    })
}

fn calculate_pattern_score(conn: &Connection, member: &TeamMember) -> Result<f64, String> {
    // Check smart_defaults patterns for this member
    let count: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM pattern_models
             WHERE pattern_type = 'smart_defaults'
             AND model_data LIKE ?1",
            [format!("%{}%", member.name)],
            |row| row.get(0),
        )
        .unwrap_or(0);

    // Normalize to 0-1 range (assume max 10 patterns)
    Ok((count as f64 / 10.0).min(1.0))
}

fn calculate_workload_score(member: &TeamMember) -> f64 {
    // Inverse of workload (low workload = high score)
    let workload = member.workload_score.unwrap_or(0.0);
    1.0 - workload
}

fn calculate_expertise_score(member: &TeamMember, task_keywords: &[String]) -> f64 {
    let expertise = match &member.expertise {
        Some(exp) => exp.clone(),
        None => return 0.0,
    };

    if expertise.is_empty() || task_keywords.is_empty() {
        return 0.0;
    }

    // Count matching keywords
    let matches = task_keywords
        .iter()
        .filter(|kw| {
            expertise.iter().any(|exp| {
                exp.to_lowercase().contains(&kw.to_lowercase())
                    || kw.to_lowercase().contains(&exp.to_lowercase())
            })
        })
        .count();

    // Normalize by number of task keywords
    (matches as f64 / task_keywords.len() as f64).min(1.0)
}

fn calculate_recency_score(conn: &Connection, member: &TeamMember) -> Result<f64, String> {
    // Check recent task completions (last 30 days)
    let count: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM tasks
             WHERE assignee = ?1
             AND status = 'done'
             AND updated_at > datetime('now', '-30 days')",
            [&member.name],
            |row| row.get(0),
        )
        .unwrap_or(0);

    // Normalize (assume active is 5+ completions/month)
    Ok((count as f64 / 5.0).min(1.0))
}

fn calculate_combined_score(factors: &AssigneeFactors, weights: &AssigneeWeights) -> f64 {
    weights.pattern * factors.pattern_score
        + weights.workload * factors.workload_score
        + weights.expertise * factors.expertise_score
        + weights.recency * factors.recency_score
}

fn score_to_confidence(score: f64) -> String {
    if score >= 0.7 {
        "high".to_string()
    } else if score >= 0.4 {
        "medium".to_string()
    } else {
        "low".to_string()
    }
}

fn determine_primary_reason(factors: &AssigneeFactors) -> String {
    let max_factor = [
        (factors.pattern_score, "Based on past assignments"),
        (factors.workload_score, "Has availability"),
        (factors.expertise_score, "Matches task expertise"),
        (factors.recency_score, "Recently active on similar tasks"),
    ]
    .into_iter()
    .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    max_factor.map(|(_, reason)| reason.to_string()).unwrap_or_else(|| "Suggested".to_string())
}

pub(crate) fn extract_keywords(title: &str, description: Option<&str>) -> Vec<String> {
    let mut text = title.to_lowercase();
    if let Some(desc) = description {
        text.push(' ');
        text.push_str(&desc.to_lowercase());
    }

    // Simple keyword extraction: words longer than 3 chars, excluding common words
    let stop_words = ["the", "and", "for", "with", "this", "that", "from", "have", "will", "should", "need", "task"];

    text.split_whitespace()
        .filter(|w| w.len() > 3 && !stop_words.contains(w))
        .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
        .filter(|w| !w.is_empty())
        .collect()
}

fn get_learned_weights(conn: &Connection) -> Result<AssigneeWeights, String> {
    // Try to load learned weights from app_settings
    let weights_json: Option<String> = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key = 'assignee_weights'",
            [],
            |row| row.get(0),
        )
        .ok();

    if let Some(json) = weights_json {
        if let Ok(weights) = serde_json::from_str::<AssigneeWeights>(&json) {
            return Ok(weights);
        }
    }

    Ok(AssigneeWeights::default())
}

pub fn record_assignee_selection(
    conn: &Connection,
    selected_name: &str,
    suggestions: &[AssigneeSuggestion],
    was_override: bool,
) -> Result<(), String> {
    // Record observation for pattern learning
    let observation = serde_json::json!({
        "selected": selected_name,
        "was_override": was_override,
        "top_suggestion": suggestions.first().map(|s| &s.member.name),
    });

    conn.execute(
        "INSERT INTO pattern_observations (id, pattern_type, observation, created_at)
         VALUES (?1, 'assignee_selection', ?2, datetime('now'))",
        rusqlite::params![
            uuid::Uuid::new_v4().to_string(),
            observation.to_string()
        ],
    )
    .map_err(|e| e.to_string())?;

    // If override, adjust weights
    if was_override {
        adjust_weights_for_override(conn)?;
    }

    Ok(())
}

fn adjust_weights_for_override(conn: &Connection) -> Result<(), String> {
    let mut weights = get_learned_weights(conn)?;

    // Reduce pattern weight slightly when overridden
    weights.pattern *= 0.95;

    // Increase other weights slightly
    weights.workload *= 1.02;
    weights.expertise *= 1.02;
    weights.recency *= 1.01;

    // Normalize
    let total = weights.pattern + weights.workload + weights.expertise + weights.recency;
    weights.pattern /= total;
    weights.workload /= total;
    weights.expertise /= total;
    weights.recency /= total;

    // Save
    let weights_json = serde_json::to_string(&weights).map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT OR REPLACE INTO app_settings (key, value) VALUES ('assignee_weights', ?1)",
        [weights_json],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE team_members (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                email TEXT,
                avatar_url TEXT,
                source TEXT NOT NULL,
                source_id TEXT,
                role TEXT DEFAULT 'member',
                expertise TEXT,
                workload_score REAL,
                metadata TEXT,
                last_synced_at TEXT,
                created_at TEXT DEFAULT (datetime('now'))
            );
            CREATE TABLE pattern_models (
                id TEXT PRIMARY KEY,
                pattern_type TEXT NOT NULL,
                model_data TEXT NOT NULL,
                confidence REAL
            );
            CREATE TABLE app_settings (key TEXT PRIMARY KEY, value TEXT);"
        ).unwrap();
        conn
    }

    #[test]
    fn test_empty_roster_falls_back_to_smart_defaults_patterns() {
        let conn = setup_test_db();
        let model_data = serde_json::json!({
            "priority_patterns": [],
            "assignee_patterns": [
                { "keyword": "billing", "assignee": "Priya", "occurrence_count": 6 },
                { "keyword": "billing", "assignee": "Sam", "occurrence_count": 1 }
            ],
            "project_defaults": {}
        });
        conn.execute(
            "INSERT INTO pattern_models (id, pattern_type, model_data, confidence)
             VALUES ('pm1', 'smart_defaults', ?1, 0.8)",
            [model_data.to_string()],
        ).unwrap();

        let suggestions = get_assignee_suggestions(&conn, "Fix billing issue", None, None).unwrap();

        assert!(!suggestions.is_empty(), "should suggest from patterns instead of returning nothing");
        assert_eq!(suggestions[0].member.name, "Priya");
        assert_eq!(suggestions[0].member.source, "pattern");
    }

    #[test]
    fn test_empty_roster_and_no_patterns_returns_empty_not_error() {
        let conn = setup_test_db();
        let suggestions = get_assignee_suggestions(&conn, "Some task", None, None).unwrap();
        assert!(suggestions.is_empty());
    }

    #[test]
    fn test_extract_keywords() {
        let keywords = extract_keywords("Fix the API authentication bug", Some("Users cannot log in"));
        assert!(keywords.contains(&"authentication".to_string()));
        assert!(keywords.contains(&"users".to_string()));
        // Stop words should be excluded
        assert!(!keywords.contains(&"the".to_string()));
        assert!(!keywords.contains(&"fix".to_string())); // "fix" is only 3 chars
    }

    #[test]
    fn test_calculate_workload_score() {
        // Low workload = high score (available)
        let low_workload = TeamMember {
            id: "1".to_string(),
            name: "Alice".to_string(),
            email: None,
            avatar_url: None,
            source: "manual".to_string(),
            source_id: None,
            role: "member".to_string(),
            expertise: None,
            workload_score: Some(0.2),
            metadata: None,
            last_synced_at: None,
            created_at: "2024-01-01".to_string(),
        };
        assert!((calculate_workload_score(&low_workload) - 0.8).abs() < 0.01);

        // High workload = low score (busy)
        let high_workload = TeamMember {
            workload_score: Some(0.9),
            ..low_workload.clone()
        };
        assert!((calculate_workload_score(&high_workload) - 0.1).abs() < 0.01);

        // No workload data = full availability
        let no_workload = TeamMember {
            workload_score: None,
            ..low_workload.clone()
        };
        assert!((calculate_workload_score(&no_workload) - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_calculate_expertise_score() {
        let member = TeamMember {
            id: "1".to_string(),
            name: "Bob".to_string(),
            email: None,
            avatar_url: None,
            source: "manual".to_string(),
            source_id: None,
            role: "member".to_string(),
            expertise: Some(vec!["rust".to_string(), "python".to_string(), "backend".to_string()]),
            workload_score: None,
            metadata: None,
            last_synced_at: None,
            created_at: "2024-01-01".to_string(),
        };

        // Full match
        let keywords = vec!["rust".to_string(), "backend".to_string()];
        let score = calculate_expertise_score(&member, &keywords);
        assert!(score > 0.9); // Both keywords match

        // Partial match
        let keywords = vec!["rust".to_string(), "frontend".to_string()];
        let score = calculate_expertise_score(&member, &keywords);
        assert!((score - 0.5).abs() < 0.1); // 1 of 2 matches

        // No match
        let keywords = vec!["frontend".to_string(), "react".to_string()];
        let score = calculate_expertise_score(&member, &keywords);
        assert!(score < 0.1);

        // No expertise = 0
        let no_expertise = TeamMember {
            expertise: None,
            ..member.clone()
        };
        let score = calculate_expertise_score(&no_expertise, &keywords);
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_score_to_confidence() {
        assert_eq!(score_to_confidence(0.85), "high");
        assert_eq!(score_to_confidence(0.70), "high");
        assert_eq!(score_to_confidence(0.55), "medium");
        assert_eq!(score_to_confidence(0.40), "medium");
        assert_eq!(score_to_confidence(0.30), "low");
        assert_eq!(score_to_confidence(0.10), "low");
    }

    #[test]
    fn test_determine_primary_reason() {
        let factors = AssigneeFactors {
            pattern_score: 0.9,
            workload_score: 0.3,
            expertise_score: 0.5,
            recency_score: 0.2,
        };
        assert_eq!(determine_primary_reason(&factors), "Based on past assignments");

        let factors = AssigneeFactors {
            pattern_score: 0.2,
            workload_score: 0.8,
            expertise_score: 0.3,
            recency_score: 0.1,
        };
        assert_eq!(determine_primary_reason(&factors), "Has availability");

        let factors = AssigneeFactors {
            pattern_score: 0.1,
            workload_score: 0.2,
            expertise_score: 0.9,
            recency_score: 0.3,
        };
        assert_eq!(determine_primary_reason(&factors), "Matches task expertise");
    }

    #[test]
    fn test_default_weights() {
        let weights = AssigneeWeights::default();
        // Weights should sum to ~1.0
        let sum = weights.pattern + weights.workload + weights.expertise + weights.recency;
        assert!((sum - 1.0).abs() < 0.01);
    }
}
