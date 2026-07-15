use rusqlite::Connection;
use serde_json::json;

use super::events::{event_matches_filter, EventType, SkillEvent};
use super::repository::{create_skill_run, get_skills_for_event};
use super::CreateSkillRunInput;

pub struct EventDispatcher;

impl EventDispatcher {
    pub fn fire_event(conn: &Connection, event: SkillEvent) -> Result<Vec<String>, String> {
        let matching_skills = get_skills_for_event(conn, &event.event_type)?;
        let mut queued_run_ids = Vec::new();

        for skill in matching_skills {
            // Check if skill's filter matches the event
            let filter = skill
                .get_trigger_config()
                .and_then(|c| c.filter)
                .unwrap_or(serde_json::Value::Null);

            if !event_matches_filter(&event, &filter) {
                continue;
            }

            // Create a skill run for this event
            let run = create_skill_run(
                conn,
                &CreateSkillRunInput {
                    skill_id: skill.id.clone(),
                    trigger_type: "event".to_string(),
                    trigger_context: Some(json!({
                        "event_type": event.event_type,
                        "entity_id": event.entity_id,
                        "entity_type": event.entity_type,
                        "project_id": event.project_id,
                        "payload": event.payload,
                    })),
                },
            )?;

            queued_run_ids.push(run.id);
        }

        Ok(queued_run_ids)
    }

    pub fn fire_task_created(
        conn: &Connection,
        task_id: &str,
        project_id: &str,
        title: &str,
        priority: &str,
        assignee: Option<&str>,
    ) -> Result<Vec<String>, String> {
        let event = SkillEvent::new(
            EventType::TaskCreated,
            json!({
                "title": title,
                "priority": priority,
                "assignee": assignee,
            }),
        )
        .with_entity("task", task_id)
        .with_project(project_id);

        Self::fire_event(conn, event)
    }

    pub fn fire_task_completed(
        conn: &Connection,
        task_id: &str,
        project_id: &str,
        title: &str,
    ) -> Result<Vec<String>, String> {
        let event = SkillEvent::new(
            EventType::TaskCompleted,
            json!({
                "title": title,
            }),
        )
        .with_entity("task", task_id)
        .with_project(project_id);

        Self::fire_event(conn, event)
    }

    pub fn fire_meeting_imported(
        conn: &Connection,
        meeting_id: &str,
        project_id: &str,
        title: &str,
        platform: &str,
        attendees: Option<&str>,
    ) -> Result<Vec<String>, String> {
        let event = SkillEvent::new(
            EventType::MeetingImported,
            json!({
                "title": title,
                "platform": platform,
                "attendees": attendees,
            }),
        )
        .with_entity("meeting", meeting_id)
        .with_project(project_id);

        Self::fire_event(conn, event)
    }

    pub fn fire_suggestion_accepted(
        conn: &Connection,
        suggestion_id: &str,
        suggestion_type: &str,
        project_id: Option<&str>,
    ) -> Result<Vec<String>, String> {
        let mut event = SkillEvent::new(
            EventType::SuggestionAccepted,
            json!({
                "suggestion_type": suggestion_type,
            }),
        )
        .with_entity("suggestion", suggestion_id);

        if let Some(pid) = project_id {
            event = event.with_project(pid);
        }

        Self::fire_event(conn, event)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_creation() {
        let event = SkillEvent::new(EventType::TaskCreated, json!({"title": "Test"}))
            .with_entity("task", "task-1")
            .with_project("proj-1");

        assert_eq!(event.event_type, "task_created");
        assert_eq!(event.entity_id, Some("task-1".to_string()));
    }
}
