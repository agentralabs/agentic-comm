//! Encryption operations for AgenticComm.
//!
//! Provides:
//! - ChaCha20-Poly1305 AEAD encryption (original)
//! - AES-256-GCM AEAD encryption (new)
//! - [`EncryptionScheme`] enum to select algorithm

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce as AesNonce,
};
use chacha20poly1305::{ChaCha20Poly1305, Nonce as ChachaNonce};
use rand::RngCore;
use serde::{Deserialize, Serialize};

// Re-use the canonical EncryptionScheme from types.rs (re-exported by lib.rs).
use crate::types::EncryptionScheme;

// ---------------------------------------------------------------------------
// EncryptionKey
// ---------------------------------------------------------------------------

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
    /// Generate a new random encryption key for ChaCha20-Poly1305.
    pub fn generate() -> Self {
        let mut key_bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut key_bytes);
        Self {
            key_bytes,
            epoch: 1,
            algorithm: "ChaCha20-Poly1305".to_string(),
        }
    }

    /// Generate a new random encryption key for a specific scheme.
    pub fn generate_for(scheme: &EncryptionScheme) -> Self {
        let mut key_bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut key_bytes);
        let algorithm = match scheme {
            EncryptionScheme::None => "None".to_string(),
            EncryptionScheme::ChaCha20Poly1305 => "ChaCha20-Poly1305".to_string(),
            EncryptionScheme::Aes256Gcm => "AES-256-GCM".to_string(),
            EncryptionScheme::X25519Aes256Gcm => "X25519-AES-256-GCM".to_string(),
        };
        Self {
            key_bytes,
            epoch: 1,
            algorithm,
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

    /// Create from raw bytes with a specific algorithm tag.
    pub fn from_bytes_with_algorithm(bytes: [u8; 32], epoch: u64, algorithm: &str) -> Self {
        Self {
            key_bytes: bytes,
            epoch,
            algorithm: algorithm.to_string(),
        }
    }

    /// Rotate: generate new key with incremented epoch.
    pub fn rotate(&self) -> Self {
        let mut new = Self::generate();
        new.epoch = self.epoch + 1;
        new.algorithm = self.algorithm.clone();
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

// ---------------------------------------------------------------------------
// Encrypted payload
// ---------------------------------------------------------------------------

/// Encrypted message payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
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

// ---------------------------------------------------------------------------
// ChaCha20-Poly1305
// ---------------------------------------------------------------------------

/// Encrypt plaintext content using ChaCha20-Poly1305.
pub fn encrypt(key: &EncryptionKey, plaintext: &str) -> Result<EncryptedPayload, String> {
    let cipher = ChaCha20Poly1305::new_from_slice(&key.key_bytes)
        .map_err(|e| format!("Invalid key: {}", e))?;

    // Generate random 12-byte nonce
    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = ChachaNonce::from_slice(&nonce_bytes);

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
    let nonce = ChachaNonce::from_slice(&nonce_bytes);

    let ciphertext =
        hex::decode(&payload.ciphertext).map_err(|e| format!("Invalid ciphertext: {}", e))?;

    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|e| format!("Decryption failed: {}", e))?;

    String::from_utf8(plaintext).map_err(|e| format!("Invalid UTF-8: {}", e))
}

// ---------------------------------------------------------------------------
// AES-256-GCM
// ---------------------------------------------------------------------------

/// AES-256-GCM encryption context.
///
/// Wraps an [`Aes256Gcm`] cipher for convenient encrypt/decrypt operations.
/// The encrypted output is `nonce (12 bytes) || ciphertext`.
pub struct Aes256GcmCipher {
    cipher: Aes256Gcm,
}

impl Aes256GcmCipher {
    /// Create from a 32-byte key.
    pub fn new(key: &[u8; 32]) -> Self {
        let cipher = Aes256Gcm::new_from_slice(key).expect("valid key length");
        Self { cipher }
    }

    /// Generate a random 32-byte key.
    pub fn generate_key() -> [u8; 32] {
        let mut key = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut key);
        key
    }

    /// Encrypt plaintext with a random 12-byte nonce.
    ///
    /// Returns `nonce (12 bytes) || ciphertext`.
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, String> {
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = AesNonce::from_slice(&nonce_bytes);

        let ciphertext = self
            .cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| format!("AES-256-GCM encryption failed: {}", e))?;

        let mut result = nonce_bytes.to_vec();
        result.extend(ciphertext);
        Ok(result)
    }

    /// Decrypt ciphertext (expects `nonce (12 bytes) || ciphertext`).
    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, String> {
        if data.len() < 12 {
            return Err("Data too short for AES-256-GCM".to_string());
        }
        let (nonce_bytes, ciphertext) = data.split_at(12);
        let nonce = AesNonce::from_slice(nonce_bytes);
        self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| format!("AES-256-GCM decryption failed: {}", e))
    }
}

/// Encrypt plaintext using AES-256-GCM, returning an [`EncryptedPayload`].
pub fn encrypt_aes(key: &EncryptionKey, plaintext: &str) -> Result<EncryptedPayload, String> {
    let cipher = Aes256Gcm::new_from_slice(&key.key_bytes)
        .map_err(|e| format!("Invalid AES key: {}", e))?;

    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = AesNonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| format!("AES-256-GCM encryption failed: {}", e))?;

    Ok(EncryptedPayload {
        ciphertext: hex::encode(&ciphertext),
        nonce: hex::encode(nonce_bytes),
        epoch: key.epoch,
        algorithm: "AES-256-GCM".to_string(),
    })
}

/// Decrypt an [`EncryptedPayload`] using AES-256-GCM.
pub fn decrypt_aes(key: &EncryptionKey, payload: &EncryptedPayload) -> Result<String, String> {
    let cipher = Aes256Gcm::new_from_slice(&key.key_bytes)
        .map_err(|e| format!("Invalid AES key: {}", e))?;

    let nonce_bytes =
        hex::decode(&payload.nonce).map_err(|e| format!("Invalid nonce: {}", e))?;
    if nonce_bytes.len() != 12 {
        return Err("Nonce must be 12 bytes".to_string());
    }
    let nonce = AesNonce::from_slice(&nonce_bytes);

    let ciphertext =
        hex::decode(&payload.ciphertext).map_err(|e| format!("Invalid ciphertext: {}", e))?;

    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|e| format!("AES-256-GCM decryption failed: {}", e))?;

    String::from_utf8(plaintext).map_err(|e| format!("Invalid UTF-8: {}", e))
}

// ---------------------------------------------------------------------------
// ChaCha20-Poly1305 cipher wrapper
// ---------------------------------------------------------------------------

/// ChaCha20-Poly1305 encryption context.
///
/// Wraps a [`ChaCha20Poly1305`] cipher for convenient encrypt/decrypt operations.
/// The encrypted output is `nonce (12 bytes) || ciphertext`.
pub struct ChaCha20Cipher {
    cipher: ChaCha20Poly1305,
}

impl ChaCha20Cipher {
    /// Create from a 32-byte key.
    pub fn new(key: &[u8; 32]) -> Self {
        let cipher = ChaCha20Poly1305::new_from_slice(key).expect("valid key length");
        Self { cipher }
    }

    /// Generate a random 32-byte key.
    pub fn generate_key() -> [u8; 32] {
        let mut key = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut key);
        key
    }

    /// Encrypt plaintext with a random 12-byte nonce.
    ///
    /// Returns `nonce (12 bytes) || ciphertext`.
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, String> {
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = ChachaNonce::from_slice(&nonce_bytes);

        let ciphertext = self
            .cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| format!("ChaCha20-Poly1305 encryption failed: {}", e))?;

        let mut result = nonce_bytes.to_vec();
        result.extend(ciphertext);
        Ok(result)
    }

    /// Decrypt ciphertext (expects `nonce (12 bytes) || ciphertext`).
    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, String> {
        if data.len() < 12 {
            return Err("Data too short for ChaCha20-Poly1305".to_string());
        }
        let (nonce_bytes, ciphertext) = data.split_at(12);
        let nonce = ChachaNonce::from_slice(nonce_bytes);
        self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| format!("ChaCha20-Poly1305 decryption failed: {}", e))
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ---- EncryptionScheme tests ----

    #[test]
    fn encryption_scheme_default_is_none() {
        // The canonical EncryptionScheme in types.rs defaults to None
        assert_eq!(EncryptionScheme::default(), EncryptionScheme::None);
    }

    #[test]
    fn encryption_scheme_serde_roundtrip() {
        let schemes = vec![
            EncryptionScheme::None,
            EncryptionScheme::ChaCha20Poly1305,
            EncryptionScheme::Aes256Gcm,
        ];
        for s in &schemes {
            let json = serde_json::to_string(s).unwrap();
            let parsed: EncryptionScheme = serde_json::from_str(&json).unwrap();
            assert_eq!(&parsed, s);
        }
    }

    // ---- EncryptionKey tests ----

    #[test]
    fn test_generate_key() {
        let key = EncryptionKey::generate();
        assert_eq!(key.epoch, 1);
        assert_eq!(key.algorithm, "ChaCha20-Poly1305");
        assert!(key.key_bytes().iter().any(|&b| b != 0));
    }

    #[test]
    fn test_generate_key_for_aes() {
        let key = EncryptionKey::generate_for(&EncryptionScheme::Aes256Gcm);
        assert_eq!(key.epoch, 1);
        assert_eq!(key.algorithm, "AES-256-GCM");
    }

    // ---- ChaCha20-Poly1305 (original) tests ----

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
        assert_ne!(key1.fingerprint(), key2.fingerprint());
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

    // ---- AES-256-GCM tests ----

    #[test]
    fn test_aes_encrypt_decrypt_roundtrip() {
        let key = EncryptionKey::generate_for(&EncryptionScheme::Aes256Gcm);
        let plaintext = "AES-256-GCM roundtrip test";
        let payload = encrypt_aes(&key, plaintext).expect("AES encryption should succeed");
        assert_eq!(payload.algorithm, "AES-256-GCM");
        let decrypted = decrypt_aes(&key, &payload).expect("AES decryption should succeed");
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_aes_wrong_key_fails() {
        let key1 = EncryptionKey::generate_for(&EncryptionScheme::Aes256Gcm);
        let key2 = EncryptionKey::generate_for(&EncryptionScheme::Aes256Gcm);
        let payload = encrypt_aes(&key1, "secret").unwrap();
        let result = decrypt_aes(&key2, &payload);
        assert!(result.is_err(), "AES decryption with wrong key must fail");
    }

    #[test]
    fn test_aes_empty_data() {
        let key = EncryptionKey::generate_for(&EncryptionScheme::Aes256Gcm);
        let payload = encrypt_aes(&key, "").unwrap();
        let decrypted = decrypt_aes(&key, &payload).unwrap();
        assert_eq!(decrypted, "");
    }

    #[test]
    fn test_aes_large_content() {
        let key = EncryptionKey::generate_for(&EncryptionScheme::Aes256Gcm);
        let plaintext = "B".repeat(100 * 1024);
        let payload = encrypt_aes(&key, &plaintext).unwrap();
        let decrypted = decrypt_aes(&key, &payload).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    // ---- Aes256GcmCipher wrapper tests ----

    #[test]
    fn test_aes_cipher_roundtrip() {
        let key = Aes256GcmCipher::generate_key();
        let cipher = Aes256GcmCipher::new(&key);
        let plaintext = b"hello from the cipher wrapper";
        let encrypted = cipher.encrypt(plaintext).unwrap();
        let decrypted = cipher.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_aes_cipher_wrong_key_fails() {
        let key1 = Aes256GcmCipher::generate_key();
        let key2 = Aes256GcmCipher::generate_key();
        let cipher1 = Aes256GcmCipher::new(&key1);
        let cipher2 = Aes256GcmCipher::new(&key2);

        let encrypted = cipher1.encrypt(b"secret data").unwrap();
        let result = cipher2.decrypt(&encrypted);
        assert!(result.is_err());
    }

    #[test]
    fn test_aes_cipher_empty_data() {
        let key = Aes256GcmCipher::generate_key();
        let cipher = Aes256GcmCipher::new(&key);
        let encrypted = cipher.encrypt(b"").unwrap();
        let decrypted = cipher.decrypt(&encrypted).unwrap();
        assert!(decrypted.is_empty());
    }

    #[test]
    fn test_aes_cipher_too_short_data() {
        let key = Aes256GcmCipher::generate_key();
        let cipher = Aes256GcmCipher::new(&key);
        let result = cipher.decrypt(&[0u8; 5]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("too short"));
    }

    // ---- ChaCha20Cipher wrapper tests ----

    #[test]
    fn test_chacha_cipher_roundtrip() {
        let key = ChaCha20Cipher::generate_key();
        let cipher = ChaCha20Cipher::new(&key);
        let plaintext = b"hello from chacha cipher wrapper";
        let encrypted = cipher.encrypt(plaintext).unwrap();
        let decrypted = cipher.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_chacha_cipher_wrong_key_fails() {
        let key1 = ChaCha20Cipher::generate_key();
        let key2 = ChaCha20Cipher::generate_key();
        let cipher1 = ChaCha20Cipher::new(&key1);
        let cipher2 = ChaCha20Cipher::new(&key2);

        let encrypted = cipher1.encrypt(b"chacha secret").unwrap();
        let result = cipher2.decrypt(&encrypted);
        assert!(result.is_err());
    }

    #[test]
    fn test_chacha_cipher_empty_data() {
        let key = ChaCha20Cipher::generate_key();
        let cipher = ChaCha20Cipher::new(&key);
        let encrypted = cipher.encrypt(b"").unwrap();
        let decrypted = cipher.decrypt(&encrypted).unwrap();
        assert!(decrypted.is_empty());
    }

    #[test]
    fn test_chacha_cipher_too_short_data() {
        let key = ChaCha20Cipher::generate_key();
        let cipher = ChaCha20Cipher::new(&key);
        let result = cipher.decrypt(&[0u8; 3]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("too short"));
    }

    // ---- Cross-algorithm: same key should not decrypt across algorithms ----

    #[test]
    fn test_cross_algorithm_fails() {
        let key = EncryptionKey::generate();
        // Encrypt with ChaCha20
        let chacha_payload = encrypt(&key, "cross-test").unwrap();
        // Attempt to decrypt with AES
        let result = decrypt_aes(&key, &chacha_payload);
        assert!(result.is_err(), "AES should not decrypt ChaCha20 ciphertext");
    }
}
