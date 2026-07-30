use ring::aead;
use ring::rand::{SecureRandom, SystemRandom};
use ring::pbkdf2;
use std::num::NonZeroU32;

const PBKDF2_ITERATIONS: u32 = 100_000;
const KEY_LEN: usize = 32; // 256-bit key for AES-256-GCM
const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;
const MAGIC: &[u8; 4] = b"MRX1";

fn derive_key(password: &str, salt: &[u8]) -> [u8; KEY_LEN] {
    let mut key = [0u8; KEY_LEN];
    pbkdf2::derive(
        pbkdf2::PBKDF2_HMAC_SHA256,
        NonZeroU32::new(PBKDF2_ITERATIONS).unwrap(),
        salt,
        password.as_bytes(),
        &mut key,
    );
    key
}

/// Encrypts `plaintext` with AES-256-GCM using a key derived from `password`.
/// Output layout: MAGIC(4) | salt(16) | nonce(12) | ciphertext+tag.
pub fn encrypt(plaintext: &[u8], password: &str) -> Result<Vec<u8>, String> {
    let rng = SystemRandom::new();

    let mut salt = [0u8; SALT_LEN];
    rng.fill(&mut salt)
        .map_err(|_| "Failed to generate encryption salt".to_string())?;

    let mut nonce_bytes = [0u8; NONCE_LEN];
    rng.fill(&mut nonce_bytes)
        .map_err(|_| "Failed to generate encryption nonce".to_string())?;

    let key_bytes = derive_key(password, &salt);
    let unbound_key = aead::UnboundKey::new(&aead::AES_256_GCM, &key_bytes)
        .map_err(|_| "Failed to initialize encryption key".to_string())?;
    let key = aead::LessSafeKey::new(unbound_key);
    let nonce = aead::Nonce::assume_unique_for_key(nonce_bytes);

    let mut in_out = plaintext.to_vec();
    key.seal_in_place_append_tag(nonce, aead::Aad::empty(), &mut in_out)
        .map_err(|_| "Encryption failed".to_string())?;

    let mut output = Vec::with_capacity(MAGIC.len() + SALT_LEN + NONCE_LEN + in_out.len());
    output.extend_from_slice(MAGIC);
    output.extend_from_slice(&salt);
    output.extend_from_slice(&nonce_bytes);
    output.extend_from_slice(&in_out);
    Ok(output)
}

/// Returns true if `data` starts with the Meridian encrypted-archive header.
pub fn is_encrypted(data: &[u8]) -> bool {
    data.len() >= MAGIC.len() && &data[..MAGIC.len()] == MAGIC
}

/// Decrypts data produced by [`encrypt`]. Fails with a generic error on wrong
/// password or corruption (AEAD does not distinguish the two).
pub fn decrypt(data: &[u8], password: &str) -> Result<Vec<u8>, String> {
    if !is_encrypted(data) {
        return Err("Not a Meridian encrypted archive".to_string());
    }

    let rest = &data[MAGIC.len()..];
    if rest.len() < SALT_LEN + NONCE_LEN {
        return Err("Archive header is corrupted".to_string());
    }

    let salt = &rest[..SALT_LEN];
    let nonce_bytes: [u8; NONCE_LEN] = rest[SALT_LEN..SALT_LEN + NONCE_LEN]
        .try_into()
        .map_err(|_| "Archive header is corrupted".to_string())?;
    let ciphertext = &rest[SALT_LEN + NONCE_LEN..];

    let key_bytes = derive_key(password, salt);
    let unbound_key = aead::UnboundKey::new(&aead::AES_256_GCM, &key_bytes)
        .map_err(|_| "Failed to initialize decryption key".to_string())?;
    let key = aead::LessSafeKey::new(unbound_key);
    let nonce = aead::Nonce::assume_unique_for_key(nonce_bytes);

    let mut in_out = ciphertext.to_vec();
    let plaintext = key
        .open_in_place(nonce, aead::Aad::empty(), &mut in_out)
        .map_err(|_| "Incorrect password, or the archive is corrupted".to_string())?;

    Ok(plaintext.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip() {
        let plaintext = b"hello meridian export data";
        let encrypted = encrypt(plaintext, "correct horse battery staple").unwrap();
        assert!(is_encrypted(&encrypted));
        let decrypted = decrypt(&encrypted, "correct horse battery staple").unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_wrong_password_fails() {
        let plaintext = b"secret data";
        let encrypted = encrypt(plaintext, "right-password").unwrap();
        let result = decrypt(&encrypted, "wrong-password");
        assert!(result.is_err());
    }

    #[test]
    fn test_unencrypted_data_not_detected_as_encrypted() {
        let plain_zip_like = b"PK\x03\x04some zip bytes";
        assert!(!is_encrypted(plain_zip_like));
    }

    #[test]
    fn test_tampered_ciphertext_fails() {
        let plaintext = b"important data";
        let mut encrypted = encrypt(plaintext, "password123").unwrap();
        let last = encrypted.len() - 1;
        encrypted[last] ^= 0xFF;
        assert!(decrypt(&encrypted, "password123").is_err());
    }
}
