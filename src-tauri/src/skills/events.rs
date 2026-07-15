use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    TaskCreated,
    TaskCompleted,
    TaskOverdue,
    MeetingImported,
    SuggestionAccepted,
    DailyStart,
    WeeklyStart,
}

impl EventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EventType::TaskCreated => "task_created",
            EventType::TaskCompleted => "task_completed",
            EventType::TaskOverdue => "task_overdue",
            EventType::MeetingImported => "meeting_imported",
            EventType::SuggestionAccepted => "suggestion_accepted",
            EventType::DailyStart => "daily_start",
            EventType::WeeklyStart => "weekly_start",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "task_created" => Some(EventType::TaskCreated),
            "task_completed" => Some(EventType::TaskCompleted),
            "task_overdue" => Some(EventType::TaskOverdue),
            "meeting_imported" => Some(EventType::MeetingImported),
            "suggestion_accepted" => Some(EventType::SuggestionAccepted),
            "daily_start" => Some(EventType::DailyStart),
            "weekly_start" => Some(EventType::WeeklyStart),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillEvent {
    pub event_type: String,
    pub entity_id: Option<String>,
    pub entity_type: Option<String>,
    pub project_id: Option<String>,
    pub payload: serde_json::Value,
    pub timestamp: String,
}

impl SkillEvent {
    pub fn new(event_type: EventType, payload: serde_json::Value) -> Self {
        Self {
            event_type: event_type.as_str().to_string(),
            entity_id: None,
            entity_type: None,
            project_id: None,
            payload,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    pub fn with_entity(mut self, entity_type: &str, entity_id: &str) -> Self {
        self.entity_type = Some(entity_type.to_string());
        self.entity_id = Some(entity_id.to_string());
        self
    }

    pub fn with_project(mut self, project_id: &str) -> Self {
        self.project_id = Some(project_id.to_string());
        self
    }
}

pub fn event_matches_filter(event: &SkillEvent, filter: &serde_json::Value) -> bool {
    if filter.is_null() {
        return true;
    }

    if let Some(obj) = filter.as_object() {
        for (key, expected) in obj {
            let actual = match key.as_str() {
                "project_id" => event.project_id.as_ref().map(|s| serde_json::Value::String(s.clone())),
                "entity_type" => event.entity_type.as_ref().map(|s| serde_json::Value::String(s.clone())),
                "entity_id" => event.entity_id.as_ref().map(|s| serde_json::Value::String(s.clone())),
                _ => event.payload.get(key).cloned(),
            };

            match actual {
                Some(val) => {
                    // Handle array filters (e.g., priority: ["high", "critical"])
                    if let Some(arr) = expected.as_array() {
                        if !arr.contains(&val) {
                            return false;
                        }
                    } else if &val != expected {
                        return false;
                    }
                }
                None => return false,
            }
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_event_type_roundtrip() {
        assert_eq!(EventType::from_str("task_created"), Some(EventType::TaskCreated));
        assert_eq!(EventType::TaskCreated.as_str(), "task_created");
    }

    #[test]
    fn test_event_creation() {
        let event = SkillEvent::new(
            EventType::TaskCreated,
            json!({"title": "Test task", "priority": "high"}),
        )
        .with_entity("task", "task-123")
        .with_project("proj-1");

        assert_eq!(event.event_type, "task_created");
        assert_eq!(event.entity_id, Some("task-123".to_string()));
        assert_eq!(event.project_id, Some("proj-1".to_string()));
    }

    #[test]
    fn test_filter_matches_null() {
        let event = SkillEvent::new(EventType::TaskCreated, json!({}));
        assert!(event_matches_filter(&event, &serde_json::Value::Null));
    }

    #[test]
    fn test_filter_matches_project() {
        let event = SkillEvent::new(EventType::TaskCreated, json!({}))
            .with_project("proj-1");

        let filter = json!({"project_id": "proj-1"});
        assert!(event_matches_filter(&event, &filter));

        let filter_other = json!({"project_id": "proj-2"});
        assert!(!event_matches_filter(&event, &filter_other));
    }

    #[test]
    fn test_filter_matches_payload() {
        let event = SkillEvent::new(
            EventType::TaskCreated,
            json!({"priority": "high", "assignee": "Alice"}),
        );

        let filter = json!({"priority": "high"});
        assert!(event_matches_filter(&event, &filter));

        let filter_wrong = json!({"priority": "low"});
        assert!(!event_matches_filter(&event, &filter_wrong));
    }

    #[test]
    fn test_filter_matches_array() {
        let event = SkillEvent::new(
            EventType::TaskCreated,
            json!({"priority": "critical"}),
        );

        let filter = json!({"priority": ["high", "critical"]});
        assert!(event_matches_filter(&event, &filter));

        let filter_no_match = json!({"priority": ["low", "medium"]});
        assert!(!event_matches_filter(&event, &filter_no_match));
    }
}
