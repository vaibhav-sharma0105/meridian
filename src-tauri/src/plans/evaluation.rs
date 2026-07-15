use crate::ai::litellm::LiteLLMClient;
use crate::models::ai_settings::AiSettings;
use super::models::PlanEvaluationResult;

pub fn build_complexity_prompt(title: &str, description: &str) -> String {
    format!(
        r#"Evaluate the complexity of this task:

Title: {}
Description: {}

Classify as:
- SIMPLE: Single action, can be done immediately (send email, make call, quick review)
- MEDIUM: 2-5 discrete steps, can be broken into subtasks
- COMPLEX: Requires research, multiple stakeholders, or unclear scope

Respond with JSON only, no explanation:
{{"complexity": "simple|medium|complex", "reasoning": "brief explanation", "suggested_subtasks": ["subtask1", "subtask2"]}}"#,
        title,
        description
    )
}

pub async fn evaluate_task_complexity(
    client: &LiteLLMClient,
    title: &str,
    description: &str,
) -> Result<PlanEvaluationResult, String> {
    let prompt = build_complexity_prompt(title, description);

    let messages = vec![serde_json::json!({
        "role": "user",
        "content": prompt
    })];

    let response = client.chat_completion(messages, Some(500)).await?;

    let cleaned = response
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    serde_json::from_str::<PlanEvaluationResult>(cleaned)
        .map_err(|e| format!("Failed to parse plan evaluation: {}. Response: {}", e, cleaned))
}

pub fn generate_simple_action(title: &str) -> Option<String> {
    let action_keywords = ["send", "email", "message", "call", "follow up", "share", "notify", "remind"];
    let title_lower = title.to_lowercase();

    for keyword in action_keywords {
        if title_lower.contains(keyword) {
            return Some(format!("Draft a message for: {}", title));
        }
    }
    None
}

pub fn has_action_keywords(title: &str) -> bool {
    let action_keywords = ["send", "email", "message", "call", "follow up", "share", "notify", "remind", "contact"];
    let title_lower = title.to_lowercase();
    action_keywords.iter().any(|kw| title_lower.contains(kw))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_action_keywords() {
        assert!(has_action_keywords("Send email to client"));
        assert!(has_action_keywords("Follow up with team"));
        assert!(has_action_keywords("Message Bob about the meeting"));
        assert!(!has_action_keywords("Review code changes"));
        assert!(!has_action_keywords("Fix the login bug"));
    }

    #[test]
    fn test_generate_simple_action() {
        assert!(generate_simple_action("Send email to client").is_some());
        assert!(generate_simple_action("Follow up with team").is_some());
        assert!(generate_simple_action("Review code changes").is_none());
    }

    #[test]
    fn test_build_complexity_prompt() {
        let prompt = build_complexity_prompt("Fix bug", "The login doesn't work");
        assert!(prompt.contains("Fix bug"));
        assert!(prompt.contains("The login doesn't work"));
        assert!(prompt.contains("SIMPLE"));
        assert!(prompt.contains("MEDIUM"));
        assert!(prompt.contains("COMPLEX"));
    }
}
