use rusqlite::{Connection, params, OptionalExtension};
use uuid::Uuid;
use crate::team::models::{TeamMember, CreateTeamMemberInput, UpdateTeamMemberInput};

pub fn get_all_team_members(conn: &Connection) -> Result<Vec<TeamMember>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, name, email, avatar_url, source, source_id, role, expertise,
                    workload_score, metadata, last_synced_at, created_at
             FROM team_members
             ORDER BY name ASC"
        )
        .map_err(|e| e.to_string())?;

    let members = stmt
        .query_map([], |row| {
            let expertise_json: Option<String> = row.get(7)?;
            let expertise: Option<Vec<String>> = expertise_json
                .and_then(|s| serde_json::from_str(&s).ok());

            let metadata_json: Option<String> = row.get(9)?;
            let metadata: Option<serde_json::Value> = metadata_json
                .and_then(|s| serde_json::from_str(&s).ok());

            Ok(TeamMember {
                id: row.get(0)?,
                name: row.get(1)?,
                email: row.get(2)?,
                avatar_url: row.get(3)?,
                source: row.get(4)?,
                source_id: row.get(5)?,
                role: row.get(6)?,
                expertise,
                workload_score: row.get(8)?,
                metadata,
                last_synced_at: row.get(10)?,
                created_at: row.get(11)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(members)
}

pub fn get_team_member(conn: &Connection, id: &str) -> Result<Option<TeamMember>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, name, email, avatar_url, source, source_id, role, expertise,
                    workload_score, metadata, last_synced_at, created_at
             FROM team_members
             WHERE id = ?1"
        )
        .map_err(|e| e.to_string())?;

    let member = stmt
        .query_row([id], |row| {
            let expertise_json: Option<String> = row.get(7)?;
            let expertise: Option<Vec<String>> = expertise_json
                .and_then(|s| serde_json::from_str(&s).ok());

            let metadata_json: Option<String> = row.get(9)?;
            let metadata: Option<serde_json::Value> = metadata_json
                .and_then(|s| serde_json::from_str(&s).ok());

            Ok(TeamMember {
                id: row.get(0)?,
                name: row.get(1)?,
                email: row.get(2)?,
                avatar_url: row.get(3)?,
                source: row.get(4)?,
                source_id: row.get(5)?,
                role: row.get(6)?,
                expertise,
                workload_score: row.get(8)?,
                metadata,
                last_synced_at: row.get(10)?,
                created_at: row.get(11)?,
            })
        })
        .optional()
        .map_err(|e| e.to_string())?;

    Ok(member)
}

pub fn get_team_member_by_source(conn: &Connection, source: &str, source_id: &str) -> Result<Option<TeamMember>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, name, email, avatar_url, source, source_id, role, expertise,
                    workload_score, metadata, last_synced_at, created_at
             FROM team_members
             WHERE source = ?1 AND source_id = ?2"
        )
        .map_err(|e| e.to_string())?;

    let member = stmt
        .query_row(params![source, source_id], |row| {
            let expertise_json: Option<String> = row.get(7)?;
            let expertise: Option<Vec<String>> = expertise_json
                .and_then(|s| serde_json::from_str(&s).ok());

            let metadata_json: Option<String> = row.get(9)?;
            let metadata: Option<serde_json::Value> = metadata_json
                .and_then(|s| serde_json::from_str(&s).ok());

            Ok(TeamMember {
                id: row.get(0)?,
                name: row.get(1)?,
                email: row.get(2)?,
                avatar_url: row.get(3)?,
                source: row.get(4)?,
                source_id: row.get(5)?,
                role: row.get(6)?,
                expertise,
                workload_score: row.get(8)?,
                metadata,
                last_synced_at: row.get(10)?,
                created_at: row.get(11)?,
            })
        })
        .optional()
        .map_err(|e| e.to_string())?;

    Ok(member)
}

pub fn create_team_member(conn: &Connection, input: &CreateTeamMemberInput) -> Result<TeamMember, String> {
    let id = Uuid::new_v4().to_string();
    let role = input.role.clone().unwrap_or_else(|| "member".to_string());
    let expertise_json = input.expertise.as_ref().map(|e| serde_json::to_string(e).unwrap_or_default());
    let metadata_json = input.metadata.as_ref().map(|m| serde_json::to_string(m).unwrap_or_default());

    conn.execute(
        "INSERT INTO team_members (id, name, email, avatar_url, source, source_id, role, expertise, metadata)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            id,
            input.name,
            input.email,
            input.avatar_url,
            input.source,
            input.source_id,
            role,
            expertise_json,
            metadata_json
        ],
    )
    .map_err(|e| e.to_string())?;

    get_team_member(conn, &id)?.ok_or_else(|| "Failed to retrieve created team member".to_string())
}

pub fn upsert_team_member(conn: &Connection, input: &CreateTeamMemberInput) -> Result<TeamMember, String> {
    // Check if exists by source + source_id
    if let Some(source_id) = &input.source_id {
        if let Some(existing) = get_team_member_by_source(conn, &input.source, source_id)? {
            // Update existing
            let update = UpdateTeamMemberInput {
                id: existing.id.clone(),
                name: Some(input.name.clone()),
                email: input.email.clone(),
                avatar_url: input.avatar_url.clone(),
                role: input.role.clone(),
                expertise: input.expertise.clone(),
                metadata: input.metadata.clone(),
            };
            return update_team_member(conn, &update);
        }
    }
    // Create new
    create_team_member(conn, input)
}

pub fn update_team_member(conn: &Connection, input: &UpdateTeamMemberInput) -> Result<TeamMember, String> {
    let existing = get_team_member(conn, &input.id)?
        .ok_or_else(|| "Team member not found".to_string())?;

    let name = input.name.clone().unwrap_or(existing.name);
    let email = input.email.clone().or(existing.email);
    let avatar_url = input.avatar_url.clone().or(existing.avatar_url);
    let role = input.role.clone().unwrap_or(existing.role);
    let expertise = input.expertise.clone().or(existing.expertise);
    let metadata = input.metadata.clone().or(existing.metadata);

    let expertise_json = expertise.as_ref().map(|e| serde_json::to_string(e).unwrap_or_default());
    let metadata_json = metadata.as_ref().map(|m| serde_json::to_string(m).unwrap_or_default());

    conn.execute(
        "UPDATE team_members
         SET name = ?1, email = ?2, avatar_url = ?3, role = ?4, expertise = ?5, metadata = ?6,
             last_synced_at = datetime('now')
         WHERE id = ?7",
        params![name, email, avatar_url, role, expertise_json, metadata_json, input.id],
    )
    .map_err(|e| e.to_string())?;

    get_team_member(conn, &input.id)?.ok_or_else(|| "Failed to retrieve updated team member".to_string())
}

pub fn delete_team_member(conn: &Connection, id: &str) -> Result<(), String> {
    conn.execute("DELETE FROM team_members WHERE id = ?1", [id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn update_workload_score(conn: &Connection, member_id: &str, score: f64) -> Result<(), String> {
    conn.execute(
        "UPDATE team_members SET workload_score = ?1 WHERE id = ?2",
        params![score, member_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn get_members_by_source(conn: &Connection, source: &str) -> Result<Vec<TeamMember>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, name, email, avatar_url, source, source_id, role, expertise,
                    workload_score, metadata, last_synced_at, created_at
             FROM team_members
             WHERE source = ?1
             ORDER BY name ASC"
        )
        .map_err(|e| e.to_string())?;

    let members = stmt
        .query_map([source], |row| {
            let expertise_json: Option<String> = row.get(7)?;
            let expertise: Option<Vec<String>> = expertise_json
                .and_then(|s| serde_json::from_str(&s).ok());

            let metadata_json: Option<String> = row.get(9)?;
            let metadata: Option<serde_json::Value> = metadata_json
                .and_then(|s| serde_json::from_str(&s).ok());

            Ok(TeamMember {
                id: row.get(0)?,
                name: row.get(1)?,
                email: row.get(2)?,
                avatar_url: row.get(3)?,
                source: row.get(4)?,
                source_id: row.get(5)?,
                role: row.get(6)?,
                expertise,
                workload_score: row.get(8)?,
                metadata,
                last_synced_at: row.get(10)?,
                created_at: row.get(11)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(members)
}

pub fn count_open_tasks_for_assignee(conn: &Connection, assignee_name: &str) -> Result<i32, String> {
    let count: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM tasks
             WHERE assignee = ?1 AND status IN ('open', 'in_progress') AND archived_at IS NULL",
            [assignee_name],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    Ok(count)
}

pub fn compute_all_workload_scores(conn: &Connection) -> Result<Vec<(String, f64)>, String> {
    let members = get_all_team_members(conn)?;
    let mut results = Vec::new();

    // Get max task count for normalization
    let mut max_tasks = 1;
    for member in &members {
        let count = count_open_tasks_for_assignee(conn, &member.name)?;
        if count > max_tasks {
            max_tasks = count;
        }
    }

    for member in members {
        let task_count = count_open_tasks_for_assignee(conn, &member.name)?;
        // Workload score: 0 = no tasks, 1 = max tasks (overloaded)
        let score = task_count as f64 / max_tasks as f64;

        // Update the member's workload_score
        update_workload_score(conn, &member.id, score)?;

        results.push((member.id, score));
    }

    Ok(results)
}

/// Number of distinct completed tasks whose keywords must match a member
/// before that keyword is promoted from "pending" into their visible
/// expertise tags. Matches the spec's "confidence increases with
/// repetition" — a single completed task never mutates expertise on its own.
pub const EXPERTISE_PROMOTION_THRESHOLD: i64 = 3;

/// Records that `member_id` completed a task with these keywords, bumping
/// each keyword's pending occurrence count. Once a keyword crosses
/// EXPERTISE_PROMOTION_THRESHOLD it's promoted into `expertise` and its
/// pending count is cleared. Pending counts live in a separate column from
/// `metadata` so they survive Slack/Google roster resyncs (which overwrite
/// `metadata` wholesale but never touch `expertise_pending`).
pub fn record_expertise_observation(
    conn: &Connection,
    member_id: &str,
    keywords: &[String],
) -> Result<(), String> {
    if keywords.is_empty() {
        return Ok(());
    }

    let member = match get_team_member(conn, member_id)? {
        Some(m) => m,
        None => return Ok(()),
    };

    let pending_json: Option<String> = conn
        .query_row(
            "SELECT expertise_pending FROM team_members WHERE id = ?1",
            [member_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?
        .flatten();

    let mut pending: std::collections::HashMap<String, i64> = pending_json
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();

    let mut expertise = member.expertise.unwrap_or_default();
    let mut expertise_changed = false;

    for keyword in keywords {
        let keyword = keyword.to_lowercase();
        if keyword.is_empty() || expertise.iter().any(|e| e.to_lowercase() == keyword) {
            continue; // already an established expertise tag, nothing to track
        }
        let count = pending.entry(keyword.clone()).or_insert(0);
        *count += 1;
        if *count >= EXPERTISE_PROMOTION_THRESHOLD {
            expertise.push(keyword.clone());
            expertise_changed = true;
            pending.remove(&keyword);
        }
    }

    let pending_json = serde_json::to_string(&pending).map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE team_members SET expertise_pending = ?1 WHERE id = ?2",
        params![pending_json, member_id],
    )
    .map_err(|e| e.to_string())?;

    if expertise_changed {
        let expertise_json = serde_json::to_string(&expertise).map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE team_members SET expertise = ?1 WHERE id = ?2",
            params![expertise_json, member_id],
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}

pub fn get_team_member_by_name(conn: &Connection, name: &str) -> Result<Option<TeamMember>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, name, email, avatar_url, source, source_id, role, expertise,
                    workload_score, metadata, last_synced_at, created_at
             FROM team_members
             WHERE LOWER(name) = LOWER(?1)"
        )
        .map_err(|e| e.to_string())?;

    let member = stmt
        .query_row([name], |row| {
            let expertise_json: Option<String> = row.get(7)?;
            let expertise: Option<Vec<String>> = expertise_json
                .and_then(|s| serde_json::from_str(&s).ok());

            let metadata_json: Option<String> = row.get(9)?;
            let metadata: Option<serde_json::Value> = metadata_json
                .and_then(|s| serde_json::from_str(&s).ok());

            Ok(TeamMember {
                id: row.get(0)?,
                name: row.get(1)?,
                email: row.get(2)?,
                avatar_url: row.get(3)?,
                source: row.get(4)?,
                source_id: row.get(5)?,
                role: row.get(6)?,
                expertise,
                workload_score: row.get(8)?,
                metadata,
                last_synced_at: row.get(10)?,
                created_at: row.get(11)?,
            })
        })
        .optional()
        .map_err(|e| e.to_string())?;

    Ok(member)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

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
                expertise_pending TEXT,
                last_synced_at TEXT,
                created_at TEXT DEFAULT (datetime('now'))
            );
            CREATE TABLE tasks (
                id TEXT PRIMARY KEY,
                assignee TEXT,
                status TEXT,
                archived_at TEXT
            );"
        ).unwrap();
        conn
    }

    #[test]
    fn test_create_team_member() {
        let conn = setup_test_db();
        let input = CreateTeamMemberInput {
            name: "Alice".to_string(),
            email: Some("alice@example.com".to_string()),
            avatar_url: None,
            source: "manual".to_string(),
            source_id: None,
            role: Some("lead".to_string()),
            expertise: Some(vec!["rust".to_string(), "python".to_string()]),
            metadata: None,
        };

        let member = create_team_member(&conn, &input).unwrap();
        assert_eq!(member.name, "Alice");
        assert_eq!(member.email, Some("alice@example.com".to_string()));
        assert_eq!(member.role, "lead");
        assert_eq!(member.expertise, Some(vec!["rust".to_string(), "python".to_string()]));
    }

    #[test]
    fn test_get_team_member_by_source() {
        let conn = setup_test_db();
        let input = CreateTeamMemberInput {
            name: "Bob".to_string(),
            email: None,
            avatar_url: None,
            source: "slack".to_string(),
            source_id: Some("U123ABC".to_string()),
            role: None,
            expertise: None,
            metadata: None,
        };

        create_team_member(&conn, &input).unwrap();

        let found = get_team_member_by_source(&conn, "slack", "U123ABC").unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Bob");

        let not_found = get_team_member_by_source(&conn, "slack", "NONEXISTENT").unwrap();
        assert!(not_found.is_none());
    }

    #[test]
    fn test_upsert_team_member() {
        let conn = setup_test_db();
        let input = CreateTeamMemberInput {
            name: "Charlie".to_string(),
            email: Some("charlie@old.com".to_string()),
            avatar_url: None,
            source: "slack".to_string(),
            source_id: Some("U456".to_string()),
            role: None,
            expertise: None,
            metadata: None,
        };

        // First upsert creates
        let created = upsert_team_member(&conn, &input).unwrap();
        assert_eq!(created.name, "Charlie");
        assert_eq!(created.email, Some("charlie@old.com".to_string()));

        // Second upsert updates
        let input2 = CreateTeamMemberInput {
            name: "Charlie Updated".to_string(),
            email: Some("charlie@new.com".to_string()),
            avatar_url: None,
            source: "slack".to_string(),
            source_id: Some("U456".to_string()),
            role: None,
            expertise: None,
            metadata: None,
        };
        let updated = upsert_team_member(&conn, &input2).unwrap();
        assert_eq!(updated.name, "Charlie Updated");
        assert_eq!(updated.email, Some("charlie@new.com".to_string()));

        // Should still be only one member
        let all = get_all_team_members(&conn).unwrap();
        assert_eq!(all.len(), 1);
    }

    #[test]
    fn test_count_open_tasks_for_assignee() {
        let conn = setup_test_db();

        conn.execute(
            "INSERT INTO tasks (id, assignee, status, archived_at) VALUES ('t1', 'Alice', 'open', NULL)",
            [],
        ).unwrap();
        conn.execute(
            "INSERT INTO tasks (id, assignee, status, archived_at) VALUES ('t2', 'Alice', 'in_progress', NULL)",
            [],
        ).unwrap();
        conn.execute(
            "INSERT INTO tasks (id, assignee, status, archived_at) VALUES ('t3', 'Alice', 'done', NULL)",
            [],
        ).unwrap();
        conn.execute(
            "INSERT INTO tasks (id, assignee, status, archived_at) VALUES ('t4', 'Bob', 'open', NULL)",
            [],
        ).unwrap();

        let alice_count = count_open_tasks_for_assignee(&conn, "Alice").unwrap();
        assert_eq!(alice_count, 2); // open + in_progress

        let bob_count = count_open_tasks_for_assignee(&conn, "Bob").unwrap();
        assert_eq!(bob_count, 1);

        let nobody_count = count_open_tasks_for_assignee(&conn, "Nobody").unwrap();
        assert_eq!(nobody_count, 0);
    }

    #[test]
    fn test_delete_team_member() {
        let conn = setup_test_db();
        let input = CreateTeamMemberInput {
            name: "ToDelete".to_string(),
            email: None,
            avatar_url: None,
            source: "manual".to_string(),
            source_id: None,
            role: None,
            expertise: None,
            metadata: None,
        };

        let member = create_team_member(&conn, &input).unwrap();
        assert!(get_team_member(&conn, &member.id).unwrap().is_some());

        delete_team_member(&conn, &member.id).unwrap();
        assert!(get_team_member(&conn, &member.id).unwrap().is_none());
    }

    fn make_member(conn: &Connection, name: &str) -> TeamMember {
        create_team_member(
            conn,
            &CreateTeamMemberInput {
                name: name.to_string(),
                email: None,
                avatar_url: None,
                source: "manual".to_string(),
                source_id: None,
                role: None,
                expertise: None,
                metadata: None,
            },
        )
        .unwrap()
    }

    #[test]
    fn test_expertise_not_promoted_below_threshold() {
        let conn = setup_test_db();
        let member = make_member(&conn, "Priya");

        for _ in 0..(EXPERTISE_PROMOTION_THRESHOLD - 1) {
            record_expertise_observation(&conn, &member.id, &["billing".to_string()]).unwrap();
        }

        let updated = get_team_member(&conn, &member.id).unwrap().unwrap();
        assert!(updated.expertise.unwrap_or_default().is_empty());
    }

    #[test]
    fn test_expertise_promoted_at_threshold() {
        let conn = setup_test_db();
        let member = make_member(&conn, "Priya");

        for _ in 0..EXPERTISE_PROMOTION_THRESHOLD {
            record_expertise_observation(&conn, &member.id, &["billing".to_string()]).unwrap();
        }

        let updated = get_team_member(&conn, &member.id).unwrap().unwrap();
        assert_eq!(updated.expertise, Some(vec!["billing".to_string()]));

        // Pending count for a promoted keyword should be cleared, not just capped.
        let pending: Option<String> = conn
            .query_row(
                "SELECT expertise_pending FROM team_members WHERE id = ?1",
                [&member.id],
                |row| row.get(0),
            )
            .unwrap();
        let pending_map: std::collections::HashMap<String, i64> =
            serde_json::from_str(&pending.unwrap()).unwrap();
        assert!(!pending_map.contains_key("billing"));
    }

    #[test]
    fn test_expertise_already_tagged_keyword_is_not_recounted() {
        let conn = setup_test_db();
        let member = make_member(&conn, "Priya");
        update_team_member(
            &conn,
            &UpdateTeamMemberInput {
                id: member.id.clone(),
                name: None,
                email: None,
                avatar_url: None,
                role: None,
                expertise: Some(vec!["billing".to_string()]),
                metadata: None,
            },
        )
        .unwrap();

        record_expertise_observation(&conn, &member.id, &["billing".to_string()]).unwrap();

        let updated = get_team_member(&conn, &member.id).unwrap().unwrap();
        assert_eq!(updated.expertise, Some(vec!["billing".to_string()]));
    }
}
