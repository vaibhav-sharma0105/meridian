use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::ai::litellm::LiteLLMClient;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedSkill {
    pub name: String,
    pub description: String,
    pub trigger_type: String,
    pub trigger_config: Option<Value>,
    pub action_type: String,
    pub system_prompt: Option<String>,
    pub approval_mode: String,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternDetection {
    pub detected: bool,
    pub pattern_type: Option<String>,
    pub description: Option<String>,
    pub confidence: f64,
    pub suggested_skill: Option<ExtractedSkill>,
}

pub async fn detect_pattern(
    client: &LiteLLMClient,
    conversation_history: &[Value],
) -> Result<PatternDetection, String> {
    if conversation_history.len() < 3 {
        return Ok(PatternDetection {
            detected: false,
            pattern_type: None,
            description: None,
            confidence: 0.0,
            suggested_skill: None,
        });
    }

    let messages = vec![
        json!({
            "role": "system",
            "content": r#"You are a pattern detection assistant. Analyze conversation history to detect repeatable patterns that could become automated skills.

Look for:
1. Repeated similar requests (same type of question/task asked multiple times)
2. Workflow patterns (sequences of related actions)
3. Scheduled needs (weekly summaries, daily reports, etc.)
4. Data gathering patterns (fetching similar info regularly)

Return JSON:
{
  "detected": true/false,
  "pattern_type": "repeated_request" | "workflow" | "scheduled" | "data_gathering" | null,
  "description": "Brief description of the pattern",
  "confidence": 0.0-1.0,
  "suggested_skill": null | {
    "name": "skill-name",
    "description": "What this skill does",
    "trigger_type": "manual" | "schedule" | "event",
    "trigger_config": { "cron": "..." } | { "event_type": "..." } | {},
    "action_type": "summarize" | "draft_message" | "create_tasks" | "analyze" | "custom",
    "system_prompt": "Instructions for the AI",
    "approval_mode": "auto" | "notify" | "approve_first",
    "confidence": 0.0-1.0
  }
}

Only suggest a skill if confidence > 0.6. Output ONLY valid JSON."#
        }),
        json!({
            "role": "user",
            "content": format!("Conversation history:\n{}", serde_json::to_string_pretty(conversation_history).unwrap_or_default())
        }),
    ];

    let response = client.chat_completion(messages, Some(800)).await?;

    let parsed: PatternDetection = serde_json::from_str(&response)
        .or_else(|_| {
            let trimmed = response
                .trim()
                .trim_start_matches("```json")
                .trim_start_matches("```")
                .trim_end_matches("```")
                .trim();
            serde_json::from_str(trimmed)
        })
        .map_err(|e| format!("Failed to parse pattern detection: {}", e))?;

    Ok(parsed)
}

pub async fn generate_skill_from_chat(
    client: &LiteLLMClient,
    description: &str,
    conversation_context: Option<&[Value]>,
) -> Result<ExtractedSkill, String> {
    let context_str = conversation_context
        .map(|ctx| format!("\n\nConversation context:\n{}", serde_json::to_string_pretty(ctx).unwrap_or_default()))
        .unwrap_or_default();

    let messages = vec![
        json!({
            "role": "system",
            "content": format!(r#"You are a skill extraction assistant. Given a description of an automation the user wants, extract a structured skill definition.

Return ONLY valid JSON with these fields:
{{
  "name": "short-kebab-case-name",
  "description": "One-line description of what this skill does",
  "trigger_type": "schedule" | "event" | "manual",
  "trigger_config": {{ "cron": "0 9 * * 1" }} | {{ "event_type": "task_completed" }} | {{}},
  "action_type": "summarize" | "draft_message" | "create_tasks" | "analyze" | "custom",
  "system_prompt": "Detailed instructions for the AI when executing this skill",
  "approval_mode": "auto" | "notify" | "approve_first",
  "confidence": 0.0-1.0
}}

Guidelines:
- For schedules, produce valid cron expressions (minutes hours day month weekday)
- Infer trigger type from context (mentions of "every Monday" → schedule, "when X happens" → event)
- approval_mode should be "notify" for most skills, "auto" only for low-risk read operations
- system_prompt should be specific and actionable

Output ONLY the JSON object, no markdown or explanation.{}"#, context_str)
        }),
        json!({
            "role": "user",
            "content": description
        }),
    ];

    let response = client.chat_completion(messages, Some(600)).await?;

    let parsed: ExtractedSkill = serde_json::from_str(&response)
        .or_else(|_| {
            let trimmed = response
                .trim()
                .trim_start_matches("```json")
                .trim_start_matches("```")
                .trim_end_matches("```")
                .trim();
            serde_json::from_str(trimmed)
        })
        .map_err(|e| format!("Failed to parse skill extraction: {}", e))?;

    Ok(parsed)
}

pub fn convert_to_skill_input(
    extracted: &ExtractedSkill,
    source_conversation_id: Option<&str>,
) -> super::CreateSkillInput {
    use super::models::{ActionConfig, ContextConfig, TriggerConfig};

    let trigger_config = match extracted.trigger_type.as_str() {
        "schedule" => extracted.trigger_config.as_ref().and_then(|tc| {
            tc.get("cron").and_then(|c| c.as_str()).map(|cron| TriggerConfig {
                cron: Some(cron.to_string()),
                timezone: None,
                event_type: None,
                filter: None,
            })
        }),
        "event" => extracted.trigger_config.as_ref().and_then(|tc| {
            tc.get("event_type").and_then(|e| e.as_str()).map(|event| TriggerConfig {
                cron: None,
                timezone: None,
                event_type: Some(event.to_string()),
                filter: tc.get("filter").cloned(),
            })
        }),
        _ => None,
    };

    let action_config = Some(ActionConfig {
        action_type: Some(extracted.action_type.clone()),
        format: Some("markdown".to_string()),
        template: extracted.system_prompt.clone(),
        max_length: None,
        channel: None,
        recipient: None,
        has_side_effects: Some(false),
        filter_source: None,
        filter_item_type: None,
        filter_patterns: None,
        filter_keywords: None,
        filter_exclude: None,
        filter_min_score: None,
    });

    let context_config = Some(ContextConfig {
        scope: Some("global".to_string()),
        project_id: None,
        include_documents: Some(false),
        document_filter: None,
        max_documents: None,
        include_archived: Some(false),
        system_prompt: extracted.system_prompt.clone(),
        output_instructions: None,
        persona: None,
        max_tokens: None,
        priority_order: None,
    });

    let mut tags = vec!["chat-extracted".to_string()];
    if let Some(conv_id) = source_conversation_id {
        tags.push(format!("source:{}", conv_id));
    }

    super::CreateSkillInput {
        name: extracted.name.clone(),
        description: Some(extracted.description.clone()),
        trigger_type: extracted.trigger_type.clone(),
        trigger_config,
        context_config,
        action_config,
        approval_mode: Some(extracted.approval_mode.clone()),
        category: Some("extracted".to_string()),
        icon: Some("💬".to_string()),
        tags: Some(tags),
        is_builtin: false,
        shared: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_to_skill_input() {
        let extracted = ExtractedSkill {
            name: "weekly-summary".to_string(),
            description: "Generate a weekly summary of tasks".to_string(),
            trigger_type: "schedule".to_string(),
            trigger_config: Some(json!({ "cron": "0 9 * * 1" })),
            action_type: "summarize".to_string(),
            system_prompt: Some("Summarize all completed tasks from the past week".to_string()),
            approval_mode: "notify".to_string(),
            confidence: 0.9,
        };

        let input = convert_to_skill_input(&extracted, Some("conv-123"));

        assert_eq!(input.name, "weekly-summary");
        assert_eq!(input.trigger_type, "schedule");
        assert!(input.trigger_config.is_some());
        assert!(input.tags.as_ref().unwrap().contains(&"chat-extracted".to_string()));
    }
}
