use chrono::Utc;
use rusqlite::{params, Connection};
use std::collections::HashMap;

use super::models::{ProductivityInsights, ProductivityPatterns, ProductivityStatus};

pub const MINIMUM_COMPLETIONS: u32 = 50;
pub const DEFAULT_PEAK_HOURS: &[u8] = &[9, 10, 14, 15]; // 9-11am, 2-4pm
pub const DEFAULT_LOW_HOURS: &[u8] = &[12, 13, 17, 18]; // Lunch, end of day

pub fn get_productivity_patterns(conn: &Connection) -> Result<ProductivityPatterns, String> {
    let patterns_json: Option<String> = conn
        .query_row(
            "SELECT productivity_patterns FROM user_profile WHERE id = 'default'",
            [],
            |row| row.get(0),
        )
        .ok();

    match patterns_json {
        Some(json) => serde_json::from_str(&json).map_err(|e| e.to_string()),
        None => Ok(ProductivityPatterns::default()),
    }
}

pub fn get_effective_peak_hours(patterns: &ProductivityPatterns, category: &str) -> Vec<u8> {
    if patterns.total_completions < MINIMUM_COMPLETIONS {
        DEFAULT_PEAK_HOURS.to_vec()
    } else {
        patterns
            .peak_hours
            .get(category)
            .cloned()
            .unwrap_or_else(|| DEFAULT_PEAK_HOURS.to_vec())
    }
}

pub fn aggregate_patterns(conn: &Connection) -> Result<ProductivityPatterns, String> {
    // Check if tracking is enabled
    let tracking_enabled: bool = conn
        .query_row(
            "SELECT productivity_tracking_enabled FROM user_profile WHERE id = 'default'",
            [],
            |row| row.get::<_, i32>(0).map(|v| v != 0),
        )
        .unwrap_or(true);

    if !tracking_enabled {
        return Ok(ProductivityPatterns {
            tracking_enabled: false,
            ..Default::default()
        });
    }

    // Get completion observations with hour/category
    let mut stmt = conn
        .prepare(
            "SELECT completion_hour, task_category, COUNT(*) as count
             FROM pattern_observations
             WHERE completion_hour IS NOT NULL AND task_category IS NOT NULL
             GROUP BY completion_hour, task_category",
        )
        .map_err(|e| e.to_string())?;

    let mut completions_by_hour: HashMap<String, [u32; 24]> = HashMap::new();
    completions_by_hour.insert("focus_work".to_string(), [0; 24]);
    completions_by_hour.insert("meetings".to_string(), [0; 24]);
    completions_by_hour.insert("quick_tasks".to_string(), [0; 24]);

    let mut total_completions = 0u32;

    let rows = stmt
        .query_map([], |row| {
            let hour: i32 = row.get(0)?;
            let category: String = row.get(1)?;
            let count: i32 = row.get(2)?;
            Ok((hour, category, count))
        })
        .map_err(|e| e.to_string())?;

    for row in rows {
        if let Ok((hour, category, count)) = row {
            if hour >= 0 && hour < 24 {
                if let Some(hours) = completions_by_hour.get_mut(&category) {
                    hours[hour as usize] += count as u32;
                    total_completions += count as u32;
                }
            }
        }
    }

    // Calculate peak hours for each category
    let mut peak_hours: HashMap<String, Vec<u8>> = HashMap::new();
    for (category, hours) in &completions_by_hour {
        let mut indexed: Vec<(usize, u32)> = hours.iter().enumerate().map(|(i, &c)| (i, c)).collect();
        indexed.sort_by(|a, b| b.1.cmp(&a.1));
        let top_3: Vec<u8> = indexed.iter().take(3).map(|(i, _)| *i as u8).collect();
        peak_hours.insert(category.clone(), top_3);
    }

    // Calculate low productivity hours (bottom 25%)
    let all_hours: Vec<u32> = completions_by_hour
        .values()
        .flat_map(|h| h.iter().copied())
        .collect();
    let total: u32 = all_hours.iter().sum();
    let threshold = if total > 0 { total / 96 } else { 0 }; // 24 hours * 4 categories / 4 (bottom 25%)

    let mut hour_totals: Vec<(u8, u32)> = (0..24)
        .map(|h| {
            let sum: u32 = completions_by_hour.values().map(|arr| arr[h as usize]).sum();
            (h, sum)
        })
        .collect();
    hour_totals.sort_by(|a, b| a.1.cmp(&b.1));
    let low_productivity_hours: Vec<u8> = hour_totals
        .iter()
        .take(4)
        .filter(|(_, count)| *count <= threshold || *count == 0)
        .map(|(h, _)| *h)
        .collect();

    let patterns = ProductivityPatterns {
        task_completions_by_hour: completions_by_hour,
        peak_hours,
        low_productivity_hours,
        total_completions,
        last_aggregation: Some(Utc::now().to_rfc3339()),
        tracking_enabled: true,
    };

    // Save patterns
    save_patterns(conn, &patterns)?;

    Ok(patterns)
}

fn save_patterns(conn: &Connection, patterns: &ProductivityPatterns) -> Result<(), String> {
    let patterns_json = serde_json::to_string(patterns).map_err(|e| e.to_string())?;
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "UPDATE user_profile SET productivity_patterns = ?1, updated_at = ?2 WHERE id = 'default'",
        params![patterns_json, now],
    )
    .map_err(|e| format!("Failed to save patterns: {}", e))?;

    Ok(())
}

pub fn get_productivity_insights(conn: &Connection) -> Result<ProductivityInsights, String> {
    let patterns = get_productivity_patterns(conn)?;

    let status = if !patterns.tracking_enabled {
        ProductivityStatus::Disabled
    } else if patterns.total_completions < MINIMUM_COMPLETIONS {
        ProductivityStatus::Learning {
            completions_needed: MINIMUM_COMPLETIONS - patterns.total_completions,
        }
    } else {
        ProductivityStatus::Ready
    };

    Ok(ProductivityInsights {
        patterns,
        status,
        storage_warning: None,
    })
}

pub fn record_completion_with_time(
    conn: &Connection,
    task_id: &str,
    category: &str,
) -> Result<(), String> {
    // Check if tracking is enabled
    let tracking_enabled: bool = conn
        .query_row(
            "SELECT productivity_tracking_enabled FROM user_profile WHERE id = 'default'",
            [],
            |row| row.get::<_, i32>(0).map(|v| v != 0),
        )
        .unwrap_or(true);

    if !tracking_enabled {
        return Ok(());
    }

    let now = Utc::now();
    let hour = now.format("%H").to_string().parse::<i32>().unwrap_or(0);
    let day_of_week = now.format("%w").to_string().parse::<i32>().unwrap_or(0);

    conn.execute(
        "UPDATE pattern_observations
         SET completion_hour = ?1, completion_day_of_week = ?2, task_category = ?3
         WHERE entity_id = ?4 AND observation_type = 'task_completion'
         ORDER BY created_at DESC LIMIT 1",
        params![hour, day_of_week, category, task_id],
    )
    .map_err(|e| format!("Failed to record completion time: {}", e))?;

    Ok(())
}

pub fn clear_productivity_data(conn: &Connection) -> Result<(), String> {
    let now = Utc::now().to_rfc3339();

    // Clear pattern observations productivity data
    conn.execute(
        "UPDATE pattern_observations
         SET completion_hour = NULL, completion_day_of_week = NULL, task_category = NULL",
        [],
    )
    .map_err(|e| e.to_string())?;

    // Reset user profile patterns
    conn.execute(
        "UPDATE user_profile SET productivity_patterns = '{}', updated_at = ?1 WHERE id = 'default'",
        params![now],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

pub fn export_productivity_data(conn: &Connection) -> Result<super::models::ProductivityExport, String> {
    let patterns = get_productivity_patterns(conn)?;

    let created_at: String = conn
        .query_row(
            "SELECT created_at FROM user_profile WHERE id = 'default'",
            [],
            |row| row.get(0),
        )
        .unwrap_or_else(|_| Utc::now().to_rfc3339());

    Ok(super::models::ProductivityExport {
        peak_hours: patterns.peak_hours,
        total_data_points: patterns.total_completions,
        tracking_since: created_at,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_peak_hours() {
        assert_eq!(DEFAULT_PEAK_HOURS, &[9, 10, 14, 15]);
    }

    #[test]
    fn test_default_low_hours() {
        assert_eq!(DEFAULT_LOW_HOURS, &[12, 13, 17, 18]);
    }

    #[test]
    fn test_minimum_completions_threshold() {
        assert_eq!(MINIMUM_COMPLETIONS, 50);
    }

    #[test]
    fn test_get_effective_peak_hours_below_threshold() {
        let patterns = ProductivityPatterns {
            total_completions: 10,
            ..Default::default()
        };
        let hours = get_effective_peak_hours(&patterns, "focus_work");
        assert_eq!(hours, DEFAULT_PEAK_HOURS.to_vec());
    }

    #[test]
    fn test_get_effective_peak_hours_above_threshold() {
        let mut peak_hours = HashMap::new();
        peak_hours.insert("focus_work".to_string(), vec![8, 9, 10]);

        let patterns = ProductivityPatterns {
            total_completions: 100,
            peak_hours,
            ..Default::default()
        };
        let hours = get_effective_peak_hours(&patterns, "focus_work");
        assert_eq!(hours, vec![8, 9, 10]);
    }

    #[test]
    fn test_get_effective_peak_hours_missing_category() {
        let patterns = ProductivityPatterns {
            total_completions: 100,
            ..Default::default()
        };
        let hours = get_effective_peak_hours(&patterns, "unknown_category");
        assert_eq!(hours, DEFAULT_PEAK_HOURS.to_vec());
    }
}
