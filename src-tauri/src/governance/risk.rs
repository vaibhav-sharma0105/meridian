use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use super::models::RiskLevel;
use super::repository::get_risk_adjustment;
use crate::sensitive;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    Read,
    Create,
    Update,
    ExternalSend,
    Delete,
}

impl ActionType {
    pub fn weight(&self) -> u8 {
        match self {
            ActionType::Read => 1,
            ActionType::Create => 2,
            ActionType::Update => 3,
            ActionType::ExternalSend => 4,
            ActionType::Delete => 5,
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "read" | "get" | "list" | "fetch" | "query" => Some(ActionType::Read),
            "create" | "insert" | "add" | "new" => Some(ActionType::Create),
            "update" | "edit" | "modify" | "patch" => Some(ActionType::Update),
            "send" | "post" | "publish" | "notify" | "message" | "external_send" => {
                Some(ActionType::ExternalSend)
            }
            "delete" | "remove" | "archive" | "destroy" => Some(ActionType::Delete),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DestinationType {
    Internal,
    Team,
    External,
    Executive,
}

impl DestinationType {
    pub fn score(&self) -> u8 {
        match self {
            DestinationType::Internal => 1,
            DestinationType::Team => 2,
            DestinationType::External => 3,
            DestinationType::Executive => 4,
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "internal" | "self" | "local" => Some(DestinationType::Internal),
            "team" | "group" | "channel" => Some(DestinationType::Team),
            "external" | "public" | "customer" | "client" => Some(DestinationType::External),
            "executive" | "ceo" | "cto" | "vp" | "director" | "leadership" => {
                Some(DestinationType::Executive)
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentRisk {
    Normal,
    Sensitive,
    Pii,
    Financial,
}

impl ContentRisk {
    pub fn score(&self) -> u8 {
        match self {
            ContentRisk::Normal => 1,
            ContentRisk::Sensitive => 2,
            ContentRisk::Pii => 3,
            ContentRisk::Financial => 4,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskScore {
    pub action_type_weight: u8,
    pub destination_score: u8,
    pub content_score: u8,
    pub adjustment: i32,
    pub total: i32,
    pub risk_level: RiskLevel,
}

impl RiskScore {
    pub fn new(action_type_weight: u8, destination_score: u8, content_score: u8) -> Self {
        let base_total = action_type_weight as i32 + destination_score as i32 + content_score as i32;
        let risk_level = Self::calculate_level(action_type_weight, destination_score, content_score, 0);

        RiskScore {
            action_type_weight,
            destination_score,
            content_score,
            adjustment: 0,
            total: base_total,
            risk_level,
        }
    }

    pub fn with_adjustment(mut self, adjustment: i32) -> Self {
        self.adjustment = adjustment;
        self.total = (self.action_type_weight as i32 + self.destination_score as i32 + self.content_score as i32) + adjustment;
        self.risk_level = Self::calculate_level(
            self.action_type_weight,
            self.destination_score,
            self.content_score,
            adjustment,
        );
        self
    }

    fn calculate_level(action: u8, destination: u8, content: u8, adjustment: i32) -> RiskLevel {
        if action == 5 || destination == 4 || content == 4 {
            return RiskLevel::Critical;
        }

        let total = (action as i32 + destination as i32 + content as i32) + adjustment;

        match total {
            t if t <= 4 => RiskLevel::Low,
            t if t <= 7 => RiskLevel::Medium,
            t if t <= 10 => RiskLevel::High,
            _ => RiskLevel::Critical,
        }
    }
}

pub fn classify_action(action_str: &str) -> ActionType {
    ActionType::from_str(action_str).unwrap_or(ActionType::Update)
}

pub fn classify_destination(destination_str: &str) -> DestinationType {
    DestinationType::from_str(destination_str).unwrap_or(DestinationType::Internal)
}

pub fn classify_content(content: &str) -> ContentRisk {
    let warnings = sensitive::scan_content(content);

    if warnings.is_empty() {
        return ContentRisk::Normal;
    }

    let has_financial = warnings.iter().any(|w| w.warning_type == "financial");
    let has_pii = warnings.iter().any(|w| w.warning_type == "pii" && w.severity == "warning");
    let has_credentials = warnings.iter().any(|w| w.warning_type == "credentials");

    if has_financial || has_credentials {
        ContentRisk::Financial
    } else if has_pii {
        ContentRisk::Pii
    } else {
        ContentRisk::Sensitive
    }
}

pub fn calculate_risk(
    action_type: ActionType,
    destination: DestinationType,
    content_risk: ContentRisk,
) -> RiskScore {
    RiskScore::new(action_type.weight(), destination.score(), content_risk.score())
}

pub fn calculate_risk_with_adjustment(
    conn: &Connection,
    action_type: ActionType,
    destination: DestinationType,
    content_risk: ContentRisk,
    target_type: &str,
    target_id: &str,
) -> RiskScore {
    let base_score = RiskScore::new(action_type.weight(), destination.score(), content_risk.score());

    if let Ok(Some(adjustment)) = get_risk_adjustment(conn, target_type, target_id) {
        base_score.with_adjustment(adjustment.risk_delta)
    } else {
        base_score
    }
}

pub fn calculate_risk_level(
    action_str: &str,
    destination_str: &str,
    content: Option<&str>,
) -> RiskLevel {
    let action = classify_action(action_str);
    let destination = classify_destination(destination_str);
    let content_risk = content.map(classify_content).unwrap_or(ContentRisk::Normal);

    calculate_risk(action, destination, content_risk).risk_level
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_type_weights() {
        assert_eq!(ActionType::Read.weight(), 1);
        assert_eq!(ActionType::Create.weight(), 2);
        assert_eq!(ActionType::Update.weight(), 3);
        assert_eq!(ActionType::ExternalSend.weight(), 4);
        assert_eq!(ActionType::Delete.weight(), 5);
    }

    #[test]
    fn test_destination_scores() {
        assert_eq!(DestinationType::Internal.score(), 1);
        assert_eq!(DestinationType::Team.score(), 2);
        assert_eq!(DestinationType::External.score(), 3);
        assert_eq!(DestinationType::Executive.score(), 4);
    }

    #[test]
    fn test_content_scores() {
        assert_eq!(ContentRisk::Normal.score(), 1);
        assert_eq!(ContentRisk::Sensitive.score(), 2);
        assert_eq!(ContentRisk::Pii.score(), 3);
        assert_eq!(ContentRisk::Financial.score(), 4);
    }

    #[test]
    fn test_low_risk_calculation() {
        let score = calculate_risk(ActionType::Read, DestinationType::Internal, ContentRisk::Normal);
        assert_eq!(score.risk_level, RiskLevel::Low);
        assert_eq!(score.total, 3);
    }

    #[test]
    fn test_medium_risk_calculation() {
        let score = calculate_risk(ActionType::Create, DestinationType::Team, ContentRisk::Normal);
        assert_eq!(score.risk_level, RiskLevel::Medium);
        assert_eq!(score.total, 5);
    }

    #[test]
    fn test_high_risk_calculation() {
        let score = calculate_risk(ActionType::Update, DestinationType::External, ContentRisk::Pii);
        assert_eq!(score.risk_level, RiskLevel::High);
        assert_eq!(score.total, 9);
    }

    #[test]
    fn test_critical_override_delete() {
        let score = calculate_risk(ActionType::Delete, DestinationType::Internal, ContentRisk::Normal);
        assert_eq!(score.risk_level, RiskLevel::Critical);
    }

    #[test]
    fn test_critical_override_executive() {
        let score = calculate_risk(ActionType::Read, DestinationType::Executive, ContentRisk::Normal);
        assert_eq!(score.risk_level, RiskLevel::Critical);
    }

    #[test]
    fn test_critical_override_financial() {
        let score = calculate_risk(ActionType::Read, DestinationType::Internal, ContentRisk::Financial);
        assert_eq!(score.risk_level, RiskLevel::Critical);
    }

    #[test]
    fn test_adjustment_lowers_risk() {
        let score = RiskScore::new(3, 2, 2).with_adjustment(-3);
        assert_eq!(score.risk_level, RiskLevel::Low);
        assert_eq!(score.total, 4);
    }

    #[test]
    fn test_adjustment_raises_risk() {
        let score = RiskScore::new(1, 1, 1).with_adjustment(5);
        assert_eq!(score.risk_level, RiskLevel::High);
        assert_eq!(score.total, 8);
    }

    #[test]
    fn test_classify_action() {
        assert_eq!(classify_action("get"), ActionType::Read);
        assert_eq!(classify_action("create"), ActionType::Create);
        assert_eq!(classify_action("update"), ActionType::Update);
        assert_eq!(classify_action("send"), ActionType::ExternalSend);
        assert_eq!(classify_action("delete"), ActionType::Delete);
        assert_eq!(classify_action("unknown"), ActionType::Update);
    }

    #[test]
    fn test_classify_destination() {
        assert_eq!(classify_destination("internal"), DestinationType::Internal);
        assert_eq!(classify_destination("team"), DestinationType::Team);
        assert_eq!(classify_destination("external"), DestinationType::External);
        assert_eq!(classify_destination("ceo"), DestinationType::Executive);
        assert_eq!(classify_destination("unknown"), DestinationType::Internal);
    }

    #[test]
    fn test_classify_content_normal() {
        let risk = classify_content("Hello, this is a normal message");
        assert_eq!(risk, ContentRisk::Normal);
    }

    #[test]
    fn test_classify_content_pii() {
        let risk = classify_content("SSN: 123-45-6789");
        assert_eq!(risk, ContentRisk::Pii);
    }

    #[test]
    fn test_classify_content_financial() {
        let risk = classify_content("Card: 4532015112830366");
        assert_eq!(risk, ContentRisk::Financial);
    }

    #[test]
    fn test_classify_content_credentials() {
        let risk = classify_content("password=secretvalue123");
        assert_eq!(risk, ContentRisk::Financial);
    }
}
