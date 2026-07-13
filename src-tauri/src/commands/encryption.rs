use crate::crypto::{get_key_config, is_encryption_initialized, KeyConfig, KeyMode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct EncryptionStatus {
    pub initialized: bool,
    pub mode: Option<String>,
    pub version: Option<u32>,
}

#[tauri::command]
pub async fn get_encryption_status() -> Result<EncryptionStatus, String> {
    let initialized = is_encryption_initialized();

    if !initialized {
        return Ok(EncryptionStatus {
            initialized: false,
            mode: None,
            version: None,
        });
    }

    let config = get_key_config()?;

    Ok(EncryptionStatus {
        initialized: true,
        mode: config.as_ref().map(|c| match c.mode {
            KeyMode::Password => "password".to_string(),
            KeyMode::Device => "device".to_string(),
        }),
        version: config.map(|c| c.version),
    })
}

#[tauri::command]
pub async fn check_password_strength(password: String) -> Result<PasswordStrength, String> {
    let length = password.len();
    let has_upper = password.chars().any(|c| c.is_uppercase());
    let has_lower = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_numeric());
    let has_special = password.chars().any(|c| !c.is_alphanumeric());

    let mut score = 0;
    if length >= 8 { score += 1; }
    if length >= 12 { score += 1; }
    if length >= 16 { score += 1; }
    if has_upper { score += 1; }
    if has_lower { score += 1; }
    if has_digit { score += 1; }
    if has_special { score += 1; }

    let (strength, label) = match score {
        0..=2 => ("weak", "Weak"),
        3..=4 => ("fair", "Fair"),
        5..=6 => ("good", "Good"),
        _ => ("strong", "Strong"),
    };

    Ok(PasswordStrength {
        score,
        strength: strength.to_string(),
        label: label.to_string(),
        suggestions: get_password_suggestions(&password),
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PasswordStrength {
    pub score: u32,
    pub strength: String,
    pub label: String,
    pub suggestions: Vec<String>,
}

fn get_password_suggestions(password: &str) -> Vec<String> {
    let mut suggestions = Vec::new();

    if password.len() < 12 {
        suggestions.push("Use at least 12 characters".to_string());
    }
    if !password.chars().any(|c| c.is_uppercase()) {
        suggestions.push("Add uppercase letters".to_string());
    }
    if !password.chars().any(|c| c.is_lowercase()) {
        suggestions.push("Add lowercase letters".to_string());
    }
    if !password.chars().any(|c| c.is_numeric()) {
        suggestions.push("Add numbers".to_string());
    }
    if !password.chars().any(|c| !c.is_alphanumeric()) {
        suggestions.push("Add special characters (!@#$%...)".to_string());
    }

    suggestions
}
