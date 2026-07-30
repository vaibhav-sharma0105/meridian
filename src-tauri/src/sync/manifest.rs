use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const EXPORT_FORMAT_VERSION: &str = "1.0.0";
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Computes a stable SHA-256 digest over a set of named byte buffers.
/// Entries are hashed in the order given, each prefixed with its name and
/// length so callers must pass entries in a fixed, agreed-upon order.
pub fn compute_checksum(entries: &[(&str, &[u8])]) -> String {
    let mut hasher = Sha256::new();
    for (name, bytes) in entries {
        hasher.update(name.as_bytes());
        hasher.update((bytes.len() as u64).to_le_bytes());
        hasher.update(bytes);
    }
    hex::encode(hasher.finalize())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportManifest {
    pub format_version: String,
    pub app_version: String,
    pub created_at: String,
    pub created_by: Option<String>,
    pub description: Option<String>,
    pub contents: ExportContents,
    pub encryption: Option<EncryptionInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportContents {
    pub projects: bool,
    pub tasks: bool,
    pub meetings: bool,
    pub skills: bool,
    pub patterns: bool,
    pub documents: bool,
    pub team_members: bool,
    pub settings: bool,
    pub vectors: bool,
    pub project_count: i32,
    pub task_count: i32,
    pub meeting_count: i32,
    pub skill_count: i32,
    pub pattern_count: i32,
    pub document_count: i32,
    pub team_member_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionInfo {
    pub algorithm: String,
    pub kdf: String,
    pub iterations: u32,
    pub salt: String,
}

impl ExportManifest {
    pub fn new(contents: ExportContents, encryption: Option<EncryptionInfo>) -> Self {
        Self {
            format_version: EXPORT_FORMAT_VERSION.to_string(),
            app_version: APP_VERSION.to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            created_by: whoami::fallible::username().ok(),
            description: None,
            contents,
            encryption,
        }
    }

    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|e| e.to_string())
    }

    pub fn from_json(json: &str) -> Result<Self, String> {
        serde_json::from_str(json).map_err(|e| e.to_string())
    }

    pub fn is_compatible(&self) -> bool {
        self.format_version.starts_with("1.")
    }
}

impl Default for ExportContents {
    fn default() -> Self {
        Self {
            projects: true,
            tasks: true,
            meetings: true,
            skills: true,
            patterns: true,
            documents: false,
            team_members: true,
            settings: false,
            vectors: false,
            project_count: 0,
            task_count: 0,
            meeting_count: 0,
            skill_count: 0,
            pattern_count: 0,
            document_count: 0,
            team_member_count: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_creation() {
        let contents = ExportContents {
            project_count: 5,
            task_count: 20,
            ..Default::default()
        };
        let manifest = ExportManifest::new(contents, None);

        assert_eq!(manifest.format_version, EXPORT_FORMAT_VERSION);
        assert_eq!(manifest.contents.project_count, 5);
        assert_eq!(manifest.contents.task_count, 20);
        assert!(manifest.encryption.is_none());
    }

    #[test]
    fn test_manifest_with_description() {
        let contents = ExportContents::default();
        let manifest = ExportManifest::new(contents, None)
            .with_description("Test backup");

        assert_eq!(manifest.description, Some("Test backup".to_string()));
    }

    #[test]
    fn test_manifest_serialization() {
        let contents = ExportContents {
            project_count: 3,
            ..Default::default()
        };
        let manifest = ExportManifest::new(contents, None);

        let json = manifest.to_json().unwrap();
        assert!(json.contains("format_version"));
        assert!(json.contains("project_count"));

        let parsed = ExportManifest::from_json(&json).unwrap();
        assert_eq!(parsed.format_version, manifest.format_version);
        assert_eq!(parsed.contents.project_count, 3);
    }

    #[test]
    fn test_version_compatibility() {
        let contents = ExportContents::default();

        // 1.x versions are compatible
        let mut manifest = ExportManifest::new(contents.clone(), None);
        manifest.format_version = "1.0.0".to_string();
        assert!(manifest.is_compatible());

        manifest.format_version = "1.5.0".to_string();
        assert!(manifest.is_compatible());

        // 2.x versions are not compatible
        manifest.format_version = "2.0.0".to_string();
        assert!(!manifest.is_compatible());
    }

    #[test]
    fn test_compute_checksum_is_deterministic_and_order_sensitive() {
        let a = compute_checksum(&[("data/tasks.json", b"[]"), ("manifest.json", b"{}")]);
        let b = compute_checksum(&[("data/tasks.json", b"[]"), ("manifest.json", b"{}")]);
        assert_eq!(a, b);

        let different_content = compute_checksum(&[("data/tasks.json", b"[1]"), ("manifest.json", b"{}")]);
        assert_ne!(a, different_content);

        let different_order = compute_checksum(&[("manifest.json", b"{}"), ("data/tasks.json", b"[]")]);
        assert_ne!(a, different_order);
    }

    #[test]
    fn test_encryption_info() {
        let contents = ExportContents::default();
        let encryption = EncryptionInfo {
            algorithm: "AES-256-GCM".to_string(),
            kdf: "PBKDF2-SHA256".to_string(),
            iterations: 100000,
            salt: "abc123".to_string(),
        };
        let manifest = ExportManifest::new(contents, Some(encryption));

        assert!(manifest.encryption.is_some());
        let enc = manifest.encryption.unwrap();
        assert_eq!(enc.algorithm, "AES-256-GCM");
        assert_eq!(enc.iterations, 100000);
    }
}
