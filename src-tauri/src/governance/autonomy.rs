use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use super::models::{ApprovalDecision, AutonomyMode, AutonomySource, RiskLevel};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutonomyContext {
    pub integration_id: Option<String>,
    pub skill_id: Option<String>,
}

pub struct AutonomyController;

impl AutonomyController {
    pub fn resolve_effective_autonomy(
        conn: &Connection,
        context: &AutonomyContext,
    ) -> Result<(AutonomyMode, AutonomySource), String> {
        if let Some(skill_id) = &context.skill_id {
            if let Some(mode) = Self::get_skill_autonomy(conn, skill_id)? {
                return Ok((mode, AutonomySource::Skill));
            }
        }

        if let Some(integration_id) = &context.integration_id {
            if let Some(mode) = Self::get_integration_autonomy(conn, integration_id)? {
                return Ok((mode, AutonomySource::Integration));
            }
        }

        let global = Self::get_global_autonomy(conn)?;
        Ok((global, AutonomySource::Global))
    }

    pub fn should_require_approval(risk_level: RiskLevel, autonomy_mode: AutonomyMode) -> bool {
        match autonomy_mode {
            AutonomyMode::Manual => true,
            AutonomyMode::Supervised => {
                matches!(risk_level, RiskLevel::High | RiskLevel::Critical)
            }
            AutonomyMode::Autonomous => {
                matches!(risk_level, RiskLevel::Critical)
            }
        }
    }

    pub fn evaluate_action(
        conn: &Connection,
        context: &AutonomyContext,
        risk_level: RiskLevel,
    ) -> Result<ApprovalDecision, String> {
        let (autonomy_mode, autonomy_source) = Self::resolve_effective_autonomy(conn, context)?;
        let requires_approval = Self::should_require_approval(risk_level, autonomy_mode);

        let reason = if requires_approval {
            Self::format_approval_reason(risk_level, autonomy_mode, autonomy_source)
        } else {
            Self::format_auto_approve_reason(risk_level, autonomy_mode, autonomy_source)
        };

        Ok(ApprovalDecision {
            requires_approval,
            risk_level,
            autonomy_mode,
            autonomy_source,
            reason,
        })
    }

    fn get_global_autonomy(conn: &Connection) -> Result<AutonomyMode, String> {
        let result: Result<String, _> = conn.query_row(
            "SELECT value FROM app_settings WHERE key = 'autonomy_mode'",
            [],
            |row| row.get(0),
        );

        match result {
            Ok(value) => Ok(AutonomyMode::from_str(&value).unwrap_or(AutonomyMode::Supervised)),
            Err(_) => Ok(AutonomyMode::Supervised),
        }
    }

    fn get_integration_autonomy(
        conn: &Connection,
        integration_id: &str,
    ) -> Result<Option<AutonomyMode>, String> {
        let result: Result<Option<String>, _> = conn.query_row(
            "SELECT autonomy_mode FROM integrations WHERE id = ?1",
            [integration_id],
            |row| row.get(0),
        );

        match result {
            Ok(Some(value)) => Ok(AutonomyMode::from_str(&value)),
            Ok(None) => Ok(None),
            Err(_) => Ok(None),
        }
    }

    fn get_skill_autonomy(conn: &Connection, skill_id: &str) -> Result<Option<AutonomyMode>, String> {
        let result: Result<Option<String>, _> = conn.query_row(
            "SELECT autonomy_mode FROM skills WHERE id = ?1",
            [skill_id],
            |row| row.get(0),
        );

        match result {
            Ok(Some(value)) => Ok(AutonomyMode::from_str(&value)),
            Ok(None) => Ok(None),
            Err(_) => Ok(None),
        }
    }

    fn format_approval_reason(
        risk: RiskLevel,
        mode: AutonomyMode,
        source: AutonomySource,
    ) -> String {
        match mode {
            AutonomyMode::Manual => format!(
                "Manual mode ({}) requires approval for all actions",
                source.as_str()
            ),
            AutonomyMode::Supervised => format!(
                "{} risk action requires approval in Supervised mode ({})",
                risk.as_str(),
                source.as_str()
            ),
            AutonomyMode::Autonomous => format!(
                "Critical risk action requires approval even in Autonomous mode ({})",
                source.as_str()
            ),
        }
    }

    fn format_auto_approve_reason(
        risk: RiskLevel,
        mode: AutonomyMode,
        source: AutonomySource,
    ) -> String {
        format!(
            "{} risk action auto-approved in {} mode ({})",
            risk.as_str(),
            mode.as_str(),
            source.as_str()
        )
    }
}

pub fn get_autonomy_setting(conn: &Connection, key: &str) -> Result<Option<String>, String> {
    match key {
        "global" => {
            let result: Result<String, _> = conn.query_row(
                "SELECT value FROM app_settings WHERE key = 'autonomy_mode'",
                [],
                |row| row.get(0),
            );
            Ok(result.ok())
        }
        _ if key.starts_with("integration:") => {
            let integration_id = &key[12..];
            let result: Result<Option<String>, _> = conn.query_row(
                "SELECT autonomy_mode FROM integrations WHERE id = ?1",
                [integration_id],
                |row| row.get(0),
            );
            Ok(result.unwrap_or(None))
        }
        _ if key.starts_with("skill:") => {
            let skill_id = &key[6..];
            let result: Result<Option<String>, _> = conn.query_row(
                "SELECT autonomy_mode FROM skills WHERE id = ?1",
                [skill_id],
                |row| row.get(0),
            );
            Ok(result.unwrap_or(None))
        }
        _ => Err(format!("Unknown autonomy setting key: {}", key)),
    }
}

pub fn set_autonomy_setting(conn: &Connection, key: &str, value: Option<&str>) -> Result<(), String> {
    match key {
        "global" => {
            let mode = value.unwrap_or("supervised");
            conn.execute(
                "INSERT INTO app_settings (key, value) VALUES ('autonomy_mode', ?1)
                 ON CONFLICT(key) DO UPDATE SET value = ?1",
                [mode],
            )
            .map_err(|e| format!("Failed to set global autonomy: {}", e))?;
            Ok(())
        }
        _ if key.starts_with("integration:") => {
            let integration_id = &key[12..];
            conn.execute(
                "UPDATE integrations SET autonomy_mode = ?2 WHERE id = ?1",
                rusqlite::params![integration_id, value],
            )
            .map_err(|e| format!("Failed to set integration autonomy: {}", e))?;
            Ok(())
        }
        _ if key.starts_with("skill:") => {
            let skill_id = &key[6..];
            conn.execute(
                "UPDATE skills SET autonomy_mode = ?2 WHERE id = ?1",
                rusqlite::params![skill_id, value],
            )
            .map_err(|e| format!("Failed to set skill autonomy: {}", e))?;
            Ok(())
        }
        _ => Err(format!("Unknown autonomy setting key: {}", key)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manual_always_requires_approval() {
        assert!(AutonomyController::should_require_approval(
            RiskLevel::Low,
            AutonomyMode::Manual
        ));
        assert!(AutonomyController::should_require_approval(
            RiskLevel::Medium,
            AutonomyMode::Manual
        ));
        assert!(AutonomyController::should_require_approval(
            RiskLevel::High,
            AutonomyMode::Manual
        ));
        assert!(AutonomyController::should_require_approval(
            RiskLevel::Critical,
            AutonomyMode::Manual
        ));
    }

    #[test]
    fn test_supervised_requires_approval_for_high_risk() {
        assert!(!AutonomyController::should_require_approval(
            RiskLevel::Low,
            AutonomyMode::Supervised
        ));
        assert!(!AutonomyController::should_require_approval(
            RiskLevel::Medium,
            AutonomyMode::Supervised
        ));
        assert!(AutonomyController::should_require_approval(
            RiskLevel::High,
            AutonomyMode::Supervised
        ));
        assert!(AutonomyController::should_require_approval(
            RiskLevel::Critical,
            AutonomyMode::Supervised
        ));
    }

    #[test]
    fn test_autonomous_only_requires_critical() {
        assert!(!AutonomyController::should_require_approval(
            RiskLevel::Low,
            AutonomyMode::Autonomous
        ));
        assert!(!AutonomyController::should_require_approval(
            RiskLevel::Medium,
            AutonomyMode::Autonomous
        ));
        assert!(!AutonomyController::should_require_approval(
            RiskLevel::High,
            AutonomyMode::Autonomous
        ));
        assert!(AutonomyController::should_require_approval(
            RiskLevel::Critical,
            AutonomyMode::Autonomous
        ));
    }

    #[test]
    fn test_autonomy_mode_default() {
        assert_eq!(AutonomyMode::default(), AutonomyMode::Supervised);
    }

    #[test]
    fn test_autonomy_mode_from_str() {
        assert_eq!(AutonomyMode::from_str("manual"), Some(AutonomyMode::Manual));
        assert_eq!(
            AutonomyMode::from_str("supervised"),
            Some(AutonomyMode::Supervised)
        );
        assert_eq!(
            AutonomyMode::from_str("autonomous"),
            Some(AutonomyMode::Autonomous)
        );
        assert_eq!(AutonomyMode::from_str("invalid"), None);
    }
}
