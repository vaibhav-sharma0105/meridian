use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RoutingDecision {
    NotificationOnly,
    MessageCenterWithNotification,
    AutoPin { reason: String },
    SuggestPin,
    None,
}

#[derive(Debug, Clone)]
pub struct SkillResultContent {
    pub has_output: bool,
    pub important: bool,
    pub skill_name: String,
}

#[derive(Debug, Clone)]
pub struct AiChatContent {
    pub has_files: bool,
    pub word_count: usize,
    pub message_preview: String,
}

#[derive(Debug, Clone)]
pub struct DigestContent {
    pub digest_type: String,
    pub summary: String,
}

#[derive(Debug, Clone)]
pub struct IntegrationSyncContent {
    pub new_items: usize,
    pub integration_name: String,
}

#[derive(Debug, Clone)]
pub enum Content {
    SkillResult(SkillResultContent),
    AiChat(AiChatContent),
    Digest(DigestContent),
    IntegrationSync(IntegrationSyncContent),
    BriefStatus { message: String },
    ApprovalRequest { action: String },
}

pub fn route_content(content: &Content) -> RoutingDecision {
    match content {
        Content::SkillResult(skill) => {
            if skill.has_output || skill.important {
                RoutingDecision::MessageCenterWithNotification
            } else {
                RoutingDecision::NotificationOnly
            }
        }
        Content::AiChat(chat) => {
            if chat.has_files {
                RoutingDecision::AutoPin {
                    reason: "file_attachment".to_string(),
                }
            } else if chat.word_count > 500 {
                RoutingDecision::SuggestPin
            } else {
                RoutingDecision::None
            }
        }
        Content::Digest(_) => RoutingDecision::MessageCenterWithNotification,
        Content::IntegrationSync(sync) => {
            if sync.new_items > 0 {
                RoutingDecision::MessageCenterWithNotification
            } else {
                RoutingDecision::NotificationOnly
            }
        }
        Content::BriefStatus { .. } => RoutingDecision::NotificationOnly,
        Content::ApprovalRequest { .. } => RoutingDecision::NotificationOnly,
    }
}

pub fn should_auto_pin(content: &Content) -> Option<String> {
    match route_content(content) {
        RoutingDecision::AutoPin { reason } => Some(reason),
        _ => None,
    }
}

pub fn should_suggest_pin(content: &Content) -> bool {
    matches!(route_content(content), RoutingDecision::SuggestPin)
}

pub fn should_create_message(content: &Content) -> bool {
    matches!(
        route_content(content),
        RoutingDecision::MessageCenterWithNotification | RoutingDecision::AutoPin { .. }
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_result_with_output_routes_to_message_center() {
        let content = Content::SkillResult(SkillResultContent {
            has_output: true,
            important: false,
            skill_name: "test".to_string(),
        });
        assert_eq!(
            route_content(&content),
            RoutingDecision::MessageCenterWithNotification
        );
    }

    #[test]
    fn test_skill_result_important_routes_to_message_center() {
        let content = Content::SkillResult(SkillResultContent {
            has_output: false,
            important: true,
            skill_name: "test".to_string(),
        });
        assert_eq!(
            route_content(&content),
            RoutingDecision::MessageCenterWithNotification
        );
    }

    #[test]
    fn test_skill_result_no_output_routes_to_notification() {
        let content = Content::SkillResult(SkillResultContent {
            has_output: false,
            important: false,
            skill_name: "test".to_string(),
        });
        assert_eq!(route_content(&content), RoutingDecision::NotificationOnly);
    }

    #[test]
    fn test_ai_chat_with_files_auto_pins() {
        let content = Content::AiChat(AiChatContent {
            has_files: true,
            word_count: 100,
            message_preview: "test".to_string(),
        });
        assert_eq!(
            route_content(&content),
            RoutingDecision::AutoPin {
                reason: "file_attachment".to_string()
            }
        );
    }

    #[test]
    fn test_ai_chat_long_response_suggests_pin() {
        let content = Content::AiChat(AiChatContent {
            has_files: false,
            word_count: 600,
            message_preview: "test".to_string(),
        });
        assert_eq!(route_content(&content), RoutingDecision::SuggestPin);
    }

    #[test]
    fn test_ai_chat_short_no_files_none() {
        let content = Content::AiChat(AiChatContent {
            has_files: false,
            word_count: 100,
            message_preview: "test".to_string(),
        });
        assert_eq!(route_content(&content), RoutingDecision::None);
    }

    #[test]
    fn test_digest_routes_to_message_center() {
        let content = Content::Digest(DigestContent {
            digest_type: "daily".to_string(),
            summary: "test".to_string(),
        });
        assert_eq!(
            route_content(&content),
            RoutingDecision::MessageCenterWithNotification
        );
    }

    #[test]
    fn test_integration_sync_with_items_routes_to_message_center() {
        let content = Content::IntegrationSync(IntegrationSyncContent {
            new_items: 5,
            integration_name: "github".to_string(),
        });
        assert_eq!(
            route_content(&content),
            RoutingDecision::MessageCenterWithNotification
        );
    }

    #[test]
    fn test_integration_sync_no_items_notification_only() {
        let content = Content::IntegrationSync(IntegrationSyncContent {
            new_items: 0,
            integration_name: "github".to_string(),
        });
        assert_eq!(route_content(&content), RoutingDecision::NotificationOnly);
    }

    #[test]
    fn test_brief_status_notification_only() {
        let content = Content::BriefStatus {
            message: "Done".to_string(),
        };
        assert_eq!(route_content(&content), RoutingDecision::NotificationOnly);
    }
}
