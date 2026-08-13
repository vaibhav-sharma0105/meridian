use chrono::Utc;
use rusqlite::{params, Connection};

use super::models::{InferenceStatus, RoleObservation, RoleScores, UserProfile};
use super::inference::{classify_role, compute_role_scores, get_inference_progress, has_minimum_activity};

pub fn get_or_create_user_profile(conn: &Connection) -> Result<UserProfile, String> {
    let exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM user_profile WHERE id = 'default')",
            [],
            |row| row.get(0),
        )
        .unwrap_or(false);

    if !exists {
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO user_profile (id, created_at, updated_at) VALUES ('default', ?1, ?1)",
            params![now],
        )
        .map_err(|e| format!("Failed to create user profile: {}", e))?;
    }

    get_user_profile(conn)
}

pub fn get_user_profile(conn: &Connection) -> Result<UserProfile, String> {
    conn.query_row(
        "SELECT id, inferred_role, secondary_role, custom_role_description,
                role_confirmed, role_confirmed_at, role_scores, last_inference_at,
                productivity_patterns, productivity_tracking_enabled, ai_context_days,
                message_retention, created_at, updated_at,
                display_name, user_email, user_aliases,
                archive_old_files, archive_after_days
         FROM user_profile WHERE id = 'default'",
        [],
        |row| {
            let role_scores_json: Option<String> = row.get(6)?;
            let role_scores = role_scores_json
                .and_then(|s| serde_json::from_str::<RoleScores>(&s).ok());

            let productivity_json: Option<String> = row.get(8)?;
            let productivity_patterns = productivity_json
                .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok());

            Ok(UserProfile {
                id: row.get(0)?,
                inferred_role: row.get(1)?,
                secondary_role: row.get(2)?,
                custom_role_description: row.get(3)?,
                role_confirmed: row.get::<_, i32>(4)? != 0,
                role_confirmed_at: row.get(5)?,
                role_scores,
                last_inference_at: row.get(7)?,
                productivity_patterns,
                productivity_tracking_enabled: row.get::<_, i32>(9).unwrap_or(1) != 0,
                ai_context_days: row.get(10).unwrap_or(30),
                message_retention: row.get(11).unwrap_or_else(|_| "forever".to_string()),
                created_at: row.get(12)?,
                updated_at: row.get(13)?,
                archive_old_files: row.get::<_, i64>(17).unwrap_or(0) != 0,
                archive_after_days: row.get(18).unwrap_or(90),
                display_name: row.get(14).unwrap_or(None),
                user_email: row.get(15).unwrap_or(None),
                user_aliases: row
                    .get::<_, Option<String>>(16)
                    .unwrap_or(None)
                    .and_then(|s| serde_json::from_str::<Vec<String>>(&s).ok())
                    .unwrap_or_default(),
            })
        },
    )
    .map_err(|e| format!("User profile not found: {}", e))
}

pub fn update_role_scores(conn: &Connection, scores: &RoleScores) -> Result<(), String> {
    let now = Utc::now().to_rfc3339();
    let scores_json = serde_json::to_string(scores).map_err(|e| e.to_string())?;
    let classification = classify_role(scores);

    conn.execute(
        "UPDATE user_profile SET
            role_scores = ?1,
            inferred_role = ?2,
            secondary_role = ?3,
            last_inference_at = ?4,
            updated_at = ?4
         WHERE id = 'default'",
        params![
            scores_json,
            classification.primary,
            classification.secondary,
            now
        ],
    )
    .map_err(|e| format!("Failed to update role scores: {}", e))?;

    Ok(())
}

pub fn confirm_role(
    conn: &Connection,
    role: &str,
    custom_description: Option<&str>,
) -> Result<UserProfile, String> {
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "UPDATE user_profile SET
            inferred_role = ?1,
            custom_role_description = ?2,
            role_confirmed = 1,
            role_confirmed_at = ?3,
            updated_at = ?3
         WHERE id = 'default'",
        params![role, custom_description, now],
    )
    .map_err(|e| format!("Failed to confirm role: {}", e))?;

    get_user_profile(conn)
}

pub fn change_role(
    conn: &Connection,
    role: &str,
    custom_description: Option<&str>,
) -> Result<UserProfile, String> {
    confirm_role(conn, role, custom_description)
}

pub fn update_user_identity(
    conn: &Connection,
    display_name: Option<&str>,
    user_email: Option<&str>,
    user_aliases: Option<&[String]>,
) -> Result<UserProfile, String> {
    let now = Utc::now().to_rfc3339();
    let aliases_json = match user_aliases {
        Some(aliases) => Some(
            serde_json::to_string(aliases)
                .map_err(|e| format!("Failed to serialize aliases: {}", e))?,
        ),
        None => None,
    };

    // COALESCE so a caller updating only one field doesn't blank the others.
    conn.execute(
        "UPDATE user_profile SET
            display_name = COALESCE(?1, display_name),
            user_email = COALESCE(?2, user_email),
            user_aliases = COALESCE(?3, user_aliases),
            updated_at = ?4
         WHERE id = 'default'",
        params![display_name, user_email, aliases_json, now],
    )
    .map_err(|e| format!("Failed to update user identity: {}", e))?;

    get_user_profile(conn)
}

pub fn dismiss_drift_alert(conn: &Connection) -> Result<(), String> {
    // For now, just update the last_inference_at to reset drift detection window
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE user_profile SET last_inference_at = ?1, updated_at = ?1 WHERE id = 'default'",
        params![now],
    )
    .map_err(|e| format!("Failed to dismiss drift alert: {}", e))?;
    Ok(())
}

pub fn get_role_observations(conn: &Connection, days: i64) -> Result<Vec<RoleObservation>, String> {
    let cutoff = (Utc::now() - chrono::Duration::days(days)).to_rfc3339();

    let mut stmt = conn
        .prepare(
            "SELECT role_signal, COUNT(*) as count
             FROM pattern_observations
             WHERE role_signal IS NOT NULL AND created_at > ?1
             GROUP BY role_signal",
        )
        .map_err(|e| e.to_string())?;

    let results: Vec<RoleObservation> = stmt
        .query_map(params![cutoff], |row| {
            Ok(RoleObservation {
                signal: row.get(0)?,
                count: row.get(1)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(results)
}

/// Role observations inside a half-open window `[from_days_ago, to_days_ago)`,
/// where 0 means "now". Used by drift detection to compare a recent behavioral
/// window against the prior one.
pub fn get_role_observations_in_window(
    conn: &Connection,
    from_days_ago: i64,
    to_days_ago: i64,
) -> Result<Vec<RoleObservation>, String> {
    let now = Utc::now();
    let older_bound = (now - chrono::Duration::days(from_days_ago)).to_rfc3339();
    let newer_bound = (now - chrono::Duration::days(to_days_ago)).to_rfc3339();

    let mut stmt = conn
        .prepare(
            "SELECT role_signal, COUNT(*) as count
             FROM pattern_observations
             WHERE role_signal IS NOT NULL
               AND created_at >= ?1
               AND created_at < ?2
             GROUP BY role_signal",
        )
        .map_err(|e| e.to_string())?;

    let results: Vec<RoleObservation> = stmt
        .query_map(params![older_bound, newer_bound], |row| {
            Ok(RoleObservation {
                signal: row.get(0)?,
                count: row.get(1)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(results)
}

/// Compares the last 14 days of behaviour against the preceding 28 days.
///
/// Deliberately *not* a comparison against the previously-stored `role_scores`:
/// inference runs daily, so consecutive snapshots are always ~1 day apart and
/// would never satisfy the spec's 14-day drift window. Recomputing both sides
/// from the observation log gives a true recent-vs-prior comparison and needs
/// no stored baseline.
pub fn check_role_drift(conn: &Connection) -> Result<Option<super::models::RoleDriftAlert>, String> {
    let profile = get_or_create_user_profile(conn)?;

    // Drift only means something once the user has committed to a role.
    if !profile.role_confirmed {
        return Ok(None);
    }

    let recent = get_role_observations_in_window(conn, DRIFT_WINDOW_DAYS, 0)?;
    let prior = get_role_observations_in_window(conn, DRIFT_WINDOW_DAYS * 3, DRIFT_WINDOW_DAYS)?;

    // Both windows need evidence, otherwise a quiet fortnight reads as a role change.
    if recent.is_empty() || prior.is_empty() {
        return Ok(None);
    }

    let current = compute_role_scores(&recent);
    let historical = compute_role_scores(&prior);

    Ok(super::drift::detect_role_drift(
        &current,
        &historical,
        DRIFT_WINDOW_DAYS,
    ))
}

const DRIFT_WINDOW_DAYS: i64 = 14;

pub fn get_activity_counts(conn: &Connection) -> Result<(i32, i32), String> {
    let task_count: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM pattern_observations WHERE observation_type = 'task_completion'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let meeting_count: i32 = conn
        .query_row(
            "SELECT COUNT(DISTINCT entity_id) FROM pattern_observations
             WHERE observation_type = 'task_completion'
               AND context_data LIKE '%meeting%'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    // Fallback: count meetings directly
    let meeting_count = if meeting_count == 0 {
        conn.query_row("SELECT COUNT(*) FROM meetings", [], |row| row.get(0))
            .unwrap_or(0)
    } else {
        meeting_count
    };

    Ok((task_count, meeting_count))
}

pub fn get_inference_status(conn: &Connection) -> Result<InferenceStatus, String> {
    let profile = get_or_create_user_profile(conn)?;
    let (task_count, meeting_count) = get_activity_counts(conn)?;

    if !has_minimum_activity(task_count, meeting_count) {
        return Ok(InferenceStatus::Learning {
            message: "Getting to know your role...".to_string(),
            progress: get_inference_progress(task_count, meeting_count),
        });
    }

    if !profile.role_confirmed {
        return Ok(InferenceStatus::PendingConfirmation {
            inferred: profile.inferred_role.unwrap_or_else(|| "ic".to_string()),
            confidence: profile
                .role_scores
                .map(|s| s.highest().1)
                .unwrap_or(0.0),
        });
    }

    Ok(InferenceStatus::Confirmed {
        role: profile.inferred_role.unwrap_or_else(|| "ic".to_string()),
        secondary: profile.secondary_role,
    })
}

pub fn record_role_observation(
    conn: &Connection,
    signal: &str,
    entity_id: &str,
) -> Result<(), String> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO pattern_observations (id, observation_type, entity_type, entity_id, role_signal, created_at)
         VALUES (?1, 'role_signal', 'task', ?2, ?3, ?4)",
        params![id, entity_id, signal, now],
    )
    .map_err(|e| format!("Failed to record role observation: {}", e))?;

    Ok(())
}

pub fn run_role_inference(conn: &Connection) -> Result<(), String> {
    let observations = get_role_observations(conn, 30)?;
    if observations.is_empty() {
        return Ok(());
    }

    let scores = compute_role_scores(&observations);
    update_role_scores(conn, &scores)?;

    Ok(())
}
