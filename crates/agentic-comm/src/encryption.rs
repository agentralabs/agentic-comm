//! Encryption operations for AgenticComm.
//!
//! Provides ChaCha20-Poly1305 AEAD encryption for message content.

use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Nonce,
};
use rand::RngCore;

/// An encryption key for channel communication.
#[derive(Debug, Clone)]
pub struct EncryptionKey {
    /// Raw 32-byte key
    key_bytes: [u8; 32],
    /// Key epoch (incremented on rotation)
    pub epoch: u64,
    /// Algorithm name
    pub algorithm: String,
}

impl EncryptionKey {
    /// Generate a new random encryption key.
    pub fn generate() -> Self {
        let mut key_bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut key_bytes);
        Self {
            key_bytes,
            epoch: 1,
            algorithm: "ChaCha20-Poly1305".to_string(),
        }
    }

    /// Create from raw bytes.
    pub fn from_bytes(bytes: [u8; 32], epoch: u64) -> Self {
        Self {
            key_bytes: bytes,
            epoch,
            algorithm: "ChaCha20-Poly1305".to_string(),
        }
    }

    /// Rotate: generate new key with incremented epoch.
    pub fn rotate(&self) -> Self {
        let mut new = Self::generate();
        new.epoch = self.epoch + 1;
        new
    }

    /// Get the raw key bytes.
    pub fn key_bytes(&self) -> &[u8; 32] {
        &self.key_bytes
    }

    /// Get hex-encoded fingerprint (first 8 bytes of key hash).
    pub fn fingerprint(&self) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(&self.key_bytes);
        let hash = hasher.finalize();
        hex::encode(&hash[..8])
    }
}

/// Encrypted message payload.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EncryptedPayload {
    /// Hex-encoded ciphertext
    pub ciphertext: String,
    /// Hex-encoded 12-byte nonce
    pub nonce: String,
    /// Key epoch used for encryption
    pub epoch: u64,
    /// Algorithm identifier
    pub algorithm: String,
}

/// Encrypt plaintext content using ChaCha20-Poly1305.
pub fn encrypt(key: &EncryptionKey, plaintext: &str) -> Result<EncryptedPayload, String> {
    let cipher = ChaCha20Poly1305::new_from_slice(&key.key_bytes)
        .map_err(|e| format!("Invalid key: {}", e))?;

    // Generate random 12-byte nonce
    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    Ok(EncryptedPayload {
        ciphertext: hex::encode(&ciphertext),
        nonce: hex::encode(nonce_bytes),
        epoch: key.epoch,
        algorithm: key.algorithm.clone(),
    })
}

/// Decrypt an encrypted payload using ChaCha20-Poly1305.
pub fn decrypt(key: &EncryptionKey, payload: &EncryptedPayload) -> Result<String, String> {
    let cipher = ChaCha20Poly1305::new_from_slice(&key.key_bytes)
        .map_err(|e| format!("Invalid key: {}", e))?;

    let nonce_bytes =
        hex::decode(&payload.nonce).map_err(|e| format!("Invalid nonce: {}", e))?;
    if nonce_bytes.len() != 12 {
        return Err("Nonce must be 12 bytes".to_string());
    }
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext =
        hex::decode(&payload.ciphertext).map_err(|e| format!("Invalid ciphertext: {}", e))?;

    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|e| format!("Decryption failed: {}", e))?;

    String::from_utf8(plaintext).map_err(|e| format!("Invalid UTF-8: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_key() {
        let key = EncryptionKey::generate();
        assert_eq!(key.epoch, 1);
        assert_eq!(key.algorithm, "ChaCha20-Poly1305");
        // Key bytes should not be all zeros (astronomically unlikely)
        assert!(key.key_bytes().iter().any(|&b| b != 0));
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = EncryptionKey::generate();
        let plaintext = "Hello, AgenticComm! This is a secret message.";
        let payload = encrypt(&key, plaintext).expect("encryption should succeed");
        let decrypted = decrypt(&key, &payload).expect("decryption should succeed");
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_decrypt_wrong_key_fails() {
        let key1 = EncryptionKey::generate();
        let key2 = EncryptionKey::generate();
        let plaintext = "Secret message";
        let payload = encrypt(&key1, plaintext).expect("encryption should succeed");
        let result = decrypt(&key2, &payload);
        assert!(result.is_err(), "decryption with wrong key must fail");
    }

    #[test]
    fn test_key_rotation() {
        let key1 = EncryptionKey::generate();
        assert_eq!(key1.epoch, 1);

        let key2 = key1.rotate();
        assert_eq!(key2.epoch, 2);
        assert_eq!(key2.algorithm, "ChaCha20-Poly1305");
        // Rotated key should have different fingerprint
        assert_ne!(key1.fingerprint(), key2.fingerprint());
        // Rotated key should have different bytes
        assert_ne!(key1.key_bytes(), key2.key_bytes());
    }

    #[test]
    fn test_encrypt_empty_string() {
        let key = EncryptionKey::generate();
        let payload = encrypt(&key, "").expect("encrypting empty string should succeed");
        let decrypted = decrypt(&key, &payload).expect("decrypting empty string should succeed");
        assert_eq!(decrypted, "");
    }

    #[test]
    fn test_encrypt_large_content() {
        let key = EncryptionKey::generate();
        // 10 KB of content
        let plaintext = "A".repeat(10 * 1024);
        let payload = encrypt(&key, &plaintext).expect("encrypting 10KB should succeed");
        let decrypted = decrypt(&key, &payload).expect("decrypting 10KB should succeed");
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_fingerprint_deterministic() {
        let bytes = [42u8; 32];
        let key1 = EncryptionKey::from_bytes(bytes, 1);
        let key2 = EncryptionKey::from_bytes(bytes, 1);
        assert_eq!(key1.fingerprint(), key2.fingerprint());
    }

    #[test]
    fn test_from_bytes_roundtrip() {
        let original = EncryptionKey::generate();
        let bytes = *original.key_bytes();
        let restored = EncryptionKey::from_bytes(bytes, original.epoch);
        assert_eq!(restored.key_bytes(), original.key_bytes());
        assert_eq!(restored.epoch, original.epoch);
        assert_eq!(restored.fingerprint(), original.fingerprint());
    }
}
