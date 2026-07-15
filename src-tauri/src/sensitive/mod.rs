use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensitiveWarning {
    pub warning_type: String,
    pub severity: String,
    pub message: String,
    pub pattern_name: String,
    pub start_pos: usize,
    pub end_pos: usize,
}

pub fn scan_content(content: &str) -> Vec<SensitiveWarning> {
    let mut warnings = Vec::new();

    warnings.extend(detect_pii(content));
    warnings.extend(detect_credentials(content));
    warnings.extend(detect_financial(content));

    warnings
}

fn detect_pii(content: &str) -> Vec<SensitiveWarning> {
    let mut warnings = Vec::new();

    if let Ok(ssn_re) = Regex::new(r"\b\d{3}-\d{2}-\d{4}\b") {
        for m in ssn_re.find_iter(content) {
            warnings.push(SensitiveWarning {
                warning_type: "pii".to_string(),
                severity: "warning".to_string(),
                message: "Social Security Number detected".to_string(),
                pattern_name: "ssn".to_string(),
                start_pos: m.start(),
                end_pos: m.end(),
            });
        }
    }

    if let Ok(phone_re) = Regex::new(r"\b(?:\+1[-.\s]?)?\(?\d{3}\)?[-.\s]?\d{3}[-.\s]?\d{4}\b") {
        for m in phone_re.find_iter(content) {
            warnings.push(SensitiveWarning {
                warning_type: "pii".to_string(),
                severity: "info".to_string(),
                message: "Phone number detected".to_string(),
                pattern_name: "phone".to_string(),
                start_pos: m.start(),
                end_pos: m.end(),
            });
        }
    }

    if let Ok(email_re) = Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b") {
        for m in email_re.find_iter(content) {
            warnings.push(SensitiveWarning {
                warning_type: "pii".to_string(),
                severity: "info".to_string(),
                message: "Email address detected".to_string(),
                pattern_name: "email".to_string(),
                start_pos: m.start(),
                end_pos: m.end(),
            });
        }
    }

    warnings
}

fn detect_credentials(content: &str) -> Vec<SensitiveWarning> {
    let mut warnings = Vec::new();

    if let Ok(api_key_re) = Regex::new(r"\b(sk-[a-zA-Z0-9]{20,}|api[_-]?key[=:]\s*\S{10,})\b") {
        for m in api_key_re.find_iter(content) {
            warnings.push(SensitiveWarning {
                warning_type: "credentials".to_string(),
                severity: "critical".to_string(),
                message: "API key detected".to_string(),
                pattern_name: "api_key".to_string(),
                start_pos: m.start(),
                end_pos: m.end(),
            });
        }
    }

    if let Ok(password_re) = Regex::new(r"(?i)(password|passwd|pwd)[=:]\s*\S+") {
        for m in password_re.find_iter(content) {
            warnings.push(SensitiveWarning {
                warning_type: "credentials".to_string(),
                severity: "critical".to_string(),
                message: "Password detected".to_string(),
                pattern_name: "password".to_string(),
                start_pos: m.start(),
                end_pos: m.end(),
            });
        }
    }

    if let Ok(secret_re) = Regex::new(r#"(?i)(secret|token)[=:]\s*['"]?[a-zA-Z0-9_-]{10,}['"]?"#) {
        for m in secret_re.find_iter(content) {
            warnings.push(SensitiveWarning {
                warning_type: "credentials".to_string(),
                severity: "critical".to_string(),
                message: "Secret/token detected".to_string(),
                pattern_name: "secret".to_string(),
                start_pos: m.start(),
                end_pos: m.end(),
            });
        }
    }

    warnings
}

fn detect_financial(content: &str) -> Vec<SensitiveWarning> {
    let mut warnings = Vec::new();

    if let Ok(cc_re) = Regex::new(r"\b(?:\d{4}[- ]?){3}\d{4}\b") {
        for m in cc_re.find_iter(content) {
            let matched = m.as_str().replace(['-', ' '], "");
            if matched.len() == 16 && luhn_check(&matched) {
                warnings.push(SensitiveWarning {
                    warning_type: "financial".to_string(),
                    severity: "critical".to_string(),
                    message: "Credit card number detected".to_string(),
                    pattern_name: "credit_card".to_string(),
                    start_pos: m.start(),
                    end_pos: m.end(),
                });
            }
        }
    }

    if let Ok(bank_re) = Regex::new(r"(?i)(?:account|routing)[#\s]*(?:number)?[:\s]*\d{8,17}") {
        for m in bank_re.find_iter(content) {
            warnings.push(SensitiveWarning {
                warning_type: "financial".to_string(),
                severity: "warning".to_string(),
                message: "Bank account/routing number detected".to_string(),
                pattern_name: "bank_account".to_string(),
                start_pos: m.start(),
                end_pos: m.end(),
            });
        }
    }

    warnings
}

fn luhn_check(number: &str) -> bool {
    let digits: Vec<u32> = number.chars().filter_map(|c| c.to_digit(10)).collect();
    if digits.len() < 13 || digits.len() > 19 {
        return false;
    }

    let mut sum = 0;
    let mut double = false;

    for digit in digits.iter().rev() {
        let mut d = *digit;
        if double {
            d *= 2;
            if d > 9 {
                d -= 9;
            }
        }
        sum += d;
        double = !double;
    }

    sum % 10 == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_ssn() {
        let content = "My SSN is 123-45-6789";
        let warnings = scan_content(content);
        assert!(warnings.iter().any(|w| w.pattern_name == "ssn"));
    }

    #[test]
    fn test_detect_phone() {
        let content = "Call me at 555-123-4567";
        let warnings = scan_content(content);
        assert!(warnings.iter().any(|w| w.pattern_name == "phone"));
    }

    #[test]
    fn test_detect_api_key() {
        let content = "Use API key sk-abcdefghijklmnopqrstuvwxyz";
        let warnings = scan_content(content);
        assert!(warnings.iter().any(|w| w.pattern_name == "api_key"));
        assert!(warnings.iter().any(|w| w.severity == "critical"));
    }

    #[test]
    fn test_detect_password() {
        let content = "password=mysecretpass123";
        let warnings = scan_content(content);
        assert!(warnings.iter().any(|w| w.pattern_name == "password"));
    }

    #[test]
    fn test_detect_credit_card() {
        let content = "Card: 4532015112830366";
        let warnings = scan_content(content);
        assert!(warnings.iter().any(|w| w.pattern_name == "credit_card"));
    }

    #[test]
    fn test_no_false_positives_on_clean_text() {
        let content = "Hello, this is a normal message about our meeting tomorrow.";
        let warnings = scan_content(content);
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_luhn_valid() {
        assert!(luhn_check("4532015112830366"));
        assert!(luhn_check("4111111111111111"));
    }

    #[test]
    fn test_luhn_invalid() {
        assert!(!luhn_check("1234567890123456"));
    }
}
