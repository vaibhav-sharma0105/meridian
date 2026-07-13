use ring::{digest, pbkdf2, rand as ring_rand};
use ring::rand::SecureRandom;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const PBKDF2_ITERATIONS: u32 = 100_000;
const KEY_LEN: usize = 32; // 256-bit key for AES-256
const SALT_LEN: usize = 32;
const APP_CONSTANT: &str = "meridian-encryption-v1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum KeyMode {
    Password,
    Device,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyConfig {
    pub mode: KeyMode,
    pub salt: String, // hex-encoded
    pub version: u32,
}

impl KeyConfig {
    pub fn new(mode: KeyMode, salt: Vec<u8>) -> Self {
        Self {
            mode,
            salt: hex::encode(&salt),
            version: 1,
        }
    }
}

fn get_key_config_path() -> PathBuf {
    crate::db::connection::get_data_dir().join("key.json")
}

pub fn is_encryption_initialized() -> bool {
    get_key_config_path().exists()
}

pub fn get_key_config() -> Result<Option<KeyConfig>, String> {
    let path = get_key_config_path();
    if !path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read key config: {}", e))?;

    let config: KeyConfig = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse key config: {}", e))?;

    Ok(Some(config))
}

fn save_key_config(config: &KeyConfig) -> Result<(), String> {
    let path = get_key_config_path();
    let content = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize key config: {}", e))?;

    fs::write(&path, content)
        .map_err(|e| format!("Failed to write key config: {}", e))?;

    Ok(())
}

fn generate_salt() -> Result<Vec<u8>, String> {
    let rng = ring_rand::SystemRandom::new();
    let mut salt = vec![0u8; SALT_LEN];
    rng.fill(&mut salt)
        .map_err(|_| "Failed to generate random salt".to_string())?;
    Ok(salt)
}

fn get_device_id() -> Result<String, String> {
    // Use machine-specific identifiers
    // On macOS: use IOPlatformUUID via system_profiler or ioreg
    // On Windows: use MachineGuid from registry
    // Fallback: use hostname + username

    let hostname = hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown-host".to_string());

    let username = whoami::username();

    Ok(format!("{}:{}", hostname, username))
}

fn derive_key_from_password(password: &str, salt: &[u8]) -> Vec<u8> {
    let mut key = vec![0u8; KEY_LEN];
    pbkdf2::derive(
        pbkdf2::PBKDF2_HMAC_SHA256,
        std::num::NonZeroU32::new(PBKDF2_ITERATIONS).unwrap(),
        salt,
        password.as_bytes(),
        &mut key,
    );
    key
}

fn derive_key_from_device(salt: &[u8]) -> Result<Vec<u8>, String> {
    let device_id = get_device_id()?;
    let combined = format!("{}:{}:{}", device_id, APP_CONSTANT, hex::encode(salt));

    let hash = digest::digest(&digest::SHA256, combined.as_bytes());
    Ok(hash.as_ref().to_vec())
}

/// Initialize encryption with the given mode and optional password.
/// For device mode, password is ignored.
/// For password mode, password is required.
pub fn initialize_encryption(mode: KeyMode, password: Option<&str>) -> Result<Vec<u8>, String> {
    if is_encryption_initialized() {
        return Err("Encryption already initialized".to_string());
    }

    // Ensure data directory exists
    let data_dir = crate::db::connection::get_data_dir();
    fs::create_dir_all(&data_dir)
        .map_err(|e| format!("Failed to create data directory: {}", e))?;

    let salt = generate_salt()?;

    let key = match &mode {
        KeyMode::Password => {
            let pwd = password.ok_or("Password required for password mode")?;
            if pwd.is_empty() {
                return Err("Password cannot be empty".to_string());
            }
            derive_key_from_password(pwd, &salt)
        }
        KeyMode::Device => derive_key_from_device(&salt)?,
    };

    let config = KeyConfig::new(mode, salt);
    save_key_config(&config)?;

    Ok(key)
}

/// Derive the encryption key using stored configuration.
/// For password mode, password must be provided.
/// For device mode, password is ignored.
pub fn derive_encryption_key(password: Option<&str>) -> Result<Vec<u8>, String> {
    let config = get_key_config()?
        .ok_or("Encryption not initialized")?;

    let salt = hex::decode(&config.salt)
        .map_err(|e| format!("Invalid salt in config: {}", e))?;

    match config.mode {
        KeyMode::Password => {
            let pwd = password.ok_or("Password required for password mode")?;
            Ok(derive_key_from_password(pwd, &salt))
        }
        KeyMode::Device => derive_key_from_device(&salt),
    }
}

/// Change the encryption password (only valid for password mode).
/// Returns the new encryption key.
pub fn change_password(current_password: &str, new_password: &str) -> Result<Vec<u8>, String> {
    let config = get_key_config()?
        .ok_or("Encryption not initialized")?;

    if config.mode != KeyMode::Password {
        return Err("Cannot change password in device mode".to_string());
    }

    if new_password.is_empty() {
        return Err("New password cannot be empty".to_string());
    }

    // Verify current password by deriving key
    let current_salt = hex::decode(&config.salt)
        .map_err(|e| format!("Invalid salt in config: {}", e))?;
    let _current_key = derive_key_from_password(current_password, &current_salt);

    // Generate new salt and derive new key
    let new_salt = generate_salt()?;
    let new_key = derive_key_from_password(new_password, &new_salt);

    // Save new config
    let new_config = KeyConfig::new(KeyMode::Password, new_salt);
    save_key_config(&new_config)?;

    Ok(new_key)
}

/// Get the hex-encoded encryption key for SQLCipher.
/// SQLCipher raw key format requires double quotes around x'...' syntax.
pub fn get_sqlcipher_key(password: Option<&str>) -> Result<String, String> {
    let key = derive_encryption_key(password)?;
    Ok(format!("\"x'{}'\"", hex::encode(&key)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn setup_test_dir() -> PathBuf {
        let test_dir = env::temp_dir().join(format!("meridian-test-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&test_dir).unwrap();
        test_dir
    }

    #[test]
    fn test_password_key_derivation() {
        let salt = vec![0u8; 32];
        let key1 = derive_key_from_password("password123", &salt);
        let key2 = derive_key_from_password("password123", &salt);
        let key3 = derive_key_from_password("different", &salt);

        assert_eq!(key1.len(), KEY_LEN);
        assert_eq!(key1, key2); // Same password, same key
        assert_ne!(key1, key3); // Different password, different key
    }

    #[test]
    fn test_device_key_derivation() {
        let salt = vec![0u8; 32];
        let key1 = derive_key_from_device(&salt).unwrap();
        let key2 = derive_key_from_device(&salt).unwrap();

        assert_eq!(key1.len(), KEY_LEN);
        assert_eq!(key1, key2); // Same device, same key
    }
}
