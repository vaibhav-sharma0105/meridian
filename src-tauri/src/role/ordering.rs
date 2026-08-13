use rusqlite::Connection;
use std::collections::HashSet;

use crate::attention::models::AttentionItem;

use super::models::UserProfile;

/// Flags the role ordering rules sort on.
///
/// The Phase 10 design named these four booleans, but `attention_items` stores
/// none of them, so each is derived from data that does exist:
///
/// - `is_assigned_to_me` — a `task` item whose `tasks.assignee` matches one of
///   the identity tokens on `user_profile`.
/// - `is_team_item` — a `task` item assigned to someone other than the user,
///   plus every `approval` item (an approval is always aimed at the team).
/// - `is_review_request` — an `approval` item. Governance approvals are
///   Meridian's only review queue today; GitHub PR-review attention items are
///   not produced by any job.
/// - `is_blocker` — a critical-severity item.
///
/// Items whose assignee is unknown (unassigned tasks, integration cache rows)
/// are neither "mine" nor "team", so they sort after both.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActivityFlags {
    pub is_assigned_to_me: bool,
    pub is_team_item: bool,
    pub is_review_request: bool,
    pub is_blocker: bool,
}

fn severity_rank(severity: &str) -> u8 {
    match severity {
        "critical" => 0,
        "warning" => 1,
        _ => 2,
    }
}

/// True when any comma-separated name in `assignee` matches an identity token.
fn assignee_matches(assignee: &str, tokens: &[String]) -> bool {
    assignee
        .split(',')
        .map(|part| part.trim().to_lowercase())
        .filter(|part| !part.is_empty())
        .any(|part| tokens.iter().any(|token| *token == part))
}

/// Task IDs (from the given candidates) whose assignee matches the user.
fn my_task_ids(
    conn: &Connection,
    task_ids: &[String],
    tokens: &[String],
) -> Result<HashSet<String>, String> {
    if task_ids.is_empty() || tokens.is_empty() {
        return Ok(HashSet::new());
    }

    let placeholders = std::iter::repeat("?")
        .take(task_ids.len())
        .collect::<Vec<_>>()
        .join(",");
    let sql = format!(
        "SELECT id, assignee FROM tasks WHERE id IN ({}) AND assignee IS NOT NULL",
        placeholders
    );

    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| format!("Failed to prepare assignee lookup: {}", e))?;
    let params = rusqlite::params_from_iter(task_ids.iter());
    let rows = stmt
        .query_map(params, |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| format!("Failed to query assignees: {}", e))?;

    let mut mine = HashSet::new();
    for row in rows.flatten() {
        let (id, assignee) = row;
        if assignee_matches(&assignee, tokens) {
            mine.insert(id);
        }
    }
    Ok(mine)
}

/// Task IDs that carry any assignee at all — used to tell "assigned to a
/// teammate" apart from "unassigned".
fn assigned_task_ids(conn: &Connection, task_ids: &[String]) -> Result<HashSet<String>, String> {
    if task_ids.is_empty() {
        return Ok(HashSet::new());
    }

    let placeholders = std::iter::repeat("?")
        .take(task_ids.len())
        .collect::<Vec<_>>()
        .join(",");
    let sql = format!(
        "SELECT id FROM tasks WHERE id IN ({}) AND assignee IS NOT NULL AND TRIM(assignee) <> ''",
        placeholders
    );

    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| format!("Failed to prepare assigned lookup: {}", e))?;
    let params = rusqlite::params_from_iter(task_ids.iter());
    let rows = stmt
        .query_map(params, |row| row.get::<_, String>(0))
        .map_err(|e| format!("Failed to query assigned tasks: {}", e))?;

    Ok(rows.flatten().collect())
}

pub fn compute_flags(
    item: &AttentionItem,
    mine: &HashSet<String>,
    assigned: &HashSet<String>,
) -> ActivityFlags {
    let is_task = item.source_type == "task";
    let is_approval = item.source_type == "approval";
    let is_assigned_to_me = is_task && mine.contains(&item.source_id);
    let is_team_item =
        is_approval || (is_task && assigned.contains(&item.source_id) && !is_assigned_to_me);

    ActivityFlags {
        is_assigned_to_me,
        is_team_item,
        is_review_request: is_approval,
        is_blocker: item.severity == "critical",
    }
}

/// Reorder attention items according to the user's role.
///
/// Severity stays the primary key — the dashboard groups by severity, and a
/// critical item must never sort below a warning. The role rule breaks ties
/// within a severity band, and recency breaks ties within that.
///
/// Returns the input unchanged when the role is unrecognised, or when the role
/// rule needs an identity the profile does not carry.
pub fn order_activity_items(
    conn: &Connection,
    items: Vec<AttentionItem>,
    role: &str,
    profile: &UserProfile,
) -> Result<Vec<AttentionItem>, String> {
    let needs_identity = matches!(role, "manager" | "ic");
    if needs_identity && !profile.has_identity() {
        return Ok(items);
    }
    if !matches!(role, "manager" | "ic" | "tech_lead") {
        return Ok(items);
    }

    let tokens = profile.identity_tokens();
    let task_ids: Vec<String> = items
        .iter()
        .filter(|i| i.source_type == "task")
        .map(|i| i.source_id.clone())
        .collect();

    let mine = my_task_ids(conn, &task_ids, &tokens)?;
    let assigned = assigned_task_ids(conn, &task_ids)?;

    let mut items = items;
    items.sort_by(|a, b| {
        let fa = compute_flags(a, &mine, &assigned);
        let fb = compute_flags(b, &mine, &assigned);

        let key_a = role_key(role, &fa);
        let key_b = role_key(role, &fb);

        severity_rank(&a.severity)
            .cmp(&severity_rank(&b.severity))
            .then(key_a.cmp(&key_b))
            // computed_at DESC — newest first, matching list_attention_items.
            .then(b.computed_at.cmp(&a.computed_at))
    });

    Ok(items)
}

/// Lower sorts earlier. Booleans are negated so "true" wins.
fn role_key(role: &str, flags: &ActivityFlags) -> (bool, bool) {
    match role {
        "manager" => (!flags.is_team_item, false),
        "ic" => (!flags.is_assigned_to_me, false),
        "tech_lead" => (!flags.is_review_request, !flags.is_blocker),
        _ => (false, false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile_with_identity() -> UserProfile {
        UserProfile {
            id: "default".to_string(),
            inferred_role: None,
            secondary_role: None,
            custom_role_description: None,
            role_confirmed: false,
            role_confirmed_at: None,
            role_scores: None,
            last_inference_at: None,
            productivity_patterns: None,
            productivity_tracking_enabled: true,
            ai_context_days: 30,
            message_retention: "forever".to_string(),
            archive_old_files: false,
            archive_after_days: 90,
            display_name: Some("Ada Lovelace".to_string()),
            user_email: Some("ada@example.com".to_string()),
            user_aliases: vec!["ada".to_string()],
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
        }
    }

    fn bare_profile() -> UserProfile {
        let mut p = profile_with_identity();
        p.display_name = None;
        p.user_email = None;
        p.user_aliases = vec![];
        p
    }

    fn item(id: &str, source_type: &str, source_id: &str, severity: &str) -> AttentionItem {
        AttentionItem {
            id: id.to_string(),
            source_type: source_type.to_string(),
            source_id: source_id.to_string(),
            severity: severity.to_string(),
            category: "overdue".to_string(),
            reason_text: None,
            matched_skill_id: None,
            computed_at: "2026-01-01T00:00:00Z".to_string(),
            dismissed_at: None,
        }
    }

    fn setup_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE tasks (id TEXT PRIMARY KEY, assignee TEXT);
             INSERT INTO tasks (id, assignee) VALUES
                ('t_mine', 'Ada Lovelace'),
                ('t_alias', 'ada'),
                ('t_multi', 'Grace Hopper, ada@example.com'),
                ('t_theirs', 'Grace Hopper'),
                ('t_unassigned', NULL);",
        )
        .unwrap();
        conn
    }

    #[test]
    fn test_identity_tokens_normalize_and_dedupe() {
        let p = profile_with_identity();
        let tokens = p.identity_tokens();
        assert!(tokens.contains(&"ada lovelace".to_string()));
        assert!(tokens.contains(&"ada@example.com".to_string()));
        assert!(tokens.contains(&"ada".to_string()));
        assert!(p.has_identity());
        assert!(!bare_profile().has_identity());
    }

    #[test]
    fn test_assignee_matches_handles_multi_assignee() {
        let tokens = profile_with_identity().identity_tokens();
        assert!(assignee_matches("Grace Hopper, ada@example.com", &tokens));
        assert!(assignee_matches("Ada Lovelace", &tokens));
        assert!(!assignee_matches("Grace Hopper", &tokens));
        // Substring must not count: "adam" is a different person.
        assert!(!assignee_matches("Adam Smith", &tokens));
    }

    #[test]
    fn test_ic_puts_own_assignments_first() {
        let conn = setup_conn();
        let items = vec![
            item("a", "task", "t_theirs", "warning"),
            item("b", "task", "t_mine", "warning"),
        ];
        let ordered =
            order_activity_items(&conn, items, "ic", &profile_with_identity()).unwrap();
        assert_eq!(ordered[0].source_id, "t_mine");
    }

    #[test]
    fn test_manager_puts_team_items_first() {
        let conn = setup_conn();
        let items = vec![
            item("a", "task", "t_mine", "warning"),
            item("b", "task", "t_theirs", "warning"),
        ];
        let ordered =
            order_activity_items(&conn, items, "manager", &profile_with_identity()).unwrap();
        assert_eq!(ordered[0].source_id, "t_theirs");
    }

    #[test]
    fn test_tech_lead_puts_reviews_then_blockers_first() {
        let conn = setup_conn();
        let items = vec![
            item("a", "task", "t_mine", "warning"),
            item("b", "approval", "ap_1", "warning"),
        ];
        let ordered =
            order_activity_items(&conn, items, "tech_lead", &profile_with_identity()).unwrap();
        assert_eq!(ordered[0].source_type, "approval");
    }

    #[test]
    fn test_severity_outranks_role_rule() {
        let conn = setup_conn();
        // For an IC, "mine" normally wins — but a critical team item must still
        // sort above a warning-level personal item.
        let items = vec![
            item("a", "task", "t_mine", "warning"),
            item("b", "task", "t_theirs", "critical"),
        ];
        let ordered =
            order_activity_items(&conn, items, "ic", &profile_with_identity()).unwrap();
        assert_eq!(ordered[0].severity, "critical");
    }

    #[test]
    fn test_no_identity_leaves_order_untouched() {
        let conn = setup_conn();
        let items = vec![
            item("a", "task", "t_theirs", "warning"),
            item("b", "task", "t_mine", "warning"),
        ];
        let ordered = order_activity_items(&conn, items, "ic", &bare_profile()).unwrap();
        assert_eq!(ordered[0].source_id, "t_theirs");
    }

    #[test]
    fn test_unknown_role_leaves_order_untouched() {
        let conn = setup_conn();
        let items = vec![
            item("a", "task", "t_theirs", "warning"),
            item("b", "task", "t_mine", "warning"),
        ];
        let ordered =
            order_activity_items(&conn, items, "pm", &profile_with_identity()).unwrap();
        assert_eq!(ordered[0].source_id, "t_theirs");
    }

    #[test]
    fn test_unassigned_task_is_neither_mine_nor_team() {
        let conn = setup_conn();
        let items = vec![item("a", "task", "t_unassigned", "warning")];
        let mine = my_task_ids(
            &conn,
            &["t_unassigned".to_string()],
            &profile_with_identity().identity_tokens(),
        )
        .unwrap();
        let assigned = assigned_task_ids(&conn, &["t_unassigned".to_string()]).unwrap();
        let flags = compute_flags(&items[0], &mine, &assigned);
        assert!(!flags.is_assigned_to_me);
        assert!(!flags.is_team_item);
    }
}
