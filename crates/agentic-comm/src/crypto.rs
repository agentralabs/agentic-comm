//! Cryptographic operations for AgenticComm.
//!
//! Provides Ed25519 signing/verification and key pair management.

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;

/// An Ed25519 key pair for signing messages.
#[derive(Debug)]
pub struct CommKeyPair {
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
}

impl CommKeyPair {
    /// Generate a new random key pair.
    pub fn generate() -> Self {
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();
        Self {
            signing_key,
            verifying_key,
        }
    }

    /// Create from existing signing key bytes (32 bytes).
    pub fn from_bytes(bytes: &[u8; 32]) -> Self {
        let signing_key = SigningKey::from_bytes(bytes);
        let verifying_key = signing_key.verifying_key();
        Self {
            signing_key,
            verifying_key,
        }
    }

    /// Sign content and return hex-encoded signature.
    pub fn sign(&self, content: &str) -> String {
        let signature = self.signing_key.sign(content.as_bytes());
        hex::encode(signature.to_bytes())
    }

    /// Get the public key as hex string.
    pub fn public_key_hex(&self) -> String {
        hex::encode(self.verifying_key.to_bytes())
    }

    /// Get the raw signing key bytes.
    pub fn signing_key_bytes(&self) -> [u8; 32] {
        self.signing_key.to_bytes()
    }

    /// Get the raw verifying key bytes.
    pub fn verifying_key_bytes(&self) -> [u8; 32] {
        self.verifying_key.to_bytes()
    }
}

impl Clone for CommKeyPair {
    fn clone(&self) -> Self {
        Self::from_bytes(&self.signing_key.to_bytes())
    }
}

/// Verify an Ed25519 signature against a public key.
///
/// `public_key_hex`: hex-encoded 32-byte public key
/// `content`: the original content that was signed
/// `signature_hex`: hex-encoded 64-byte signature
pub fn verify_signature(public_key_hex: &str, content: &str, signature_hex: &str) -> bool {
    let pk_bytes = match hex::decode(public_key_hex) {
        Ok(b) if b.len() == 32 => {
            let mut arr = [0u8; 32];
            arr.copy_from_slice(&b);
            arr
        }
        _ => return false,
    };

    let sig_bytes = match hex::decode(signature_hex) {
        Ok(b) if b.len() == 64 => {
            let mut arr = [0u8; 64];
            arr.copy_from_slice(&b);
            arr
        }
        _ => return false,
    };

    let verifying_key = match VerifyingKey::from_bytes(&pk_bytes) {
        Ok(k) => k,
        Err(_) => return false,
    };

    let signature = Signature::from_bytes(&sig_bytes);
    verifying_key.verify(content.as_bytes(), &signature).is_ok()
}

/// Compute SHA-256 hash of content (for backward compatibility).
pub fn sha256_hash(content: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    hex::encode(hasher.finalize())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_key_pair() {
        let kp = CommKeyPair::generate();
        let pk_hex = kp.public_key_hex();
        // Ed25519 public key is 32 bytes = 64 hex chars
        assert_eq!(pk_hex.len(), 64);
        // All hex characters
        assert!(pk_hex.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_sign_and_verify() {
        let kp = CommKeyPair::generate();
        let content = "Hello, agent world!";
        let sig = kp.sign(content);
        let pk = kp.public_key_hex();

        assert!(
            verify_signature(&pk, content, &sig),
            "Signature should verify against correct content and key"
        );
    }

    #[test]
    fn test_verify_wrong_content_fails() {
        let kp = CommKeyPair::generate();
        let sig = kp.sign("original content");
        let pk = kp.public_key_hex();

        assert!(
            !verify_signature(&pk, "tampered content", &sig),
            "Signature should NOT verify against different content"
        );
    }

    #[test]
    fn test_verify_wrong_key_fails() {
        let kp1 = CommKeyPair::generate();
        let kp2 = CommKeyPair::generate();
        let content = "signed by kp1";
        let sig = kp1.sign(content);

        assert!(
            !verify_signature(&kp2.public_key_hex(), content, &sig),
            "Signature should NOT verify against a different public key"
        );
    }

    #[test]
    fn test_from_bytes_roundtrip() {
        let kp = CommKeyPair::generate();
        let bytes = kp.signing_key_bytes();
        let kp2 = CommKeyPair::from_bytes(&bytes);

        assert_eq!(kp.public_key_hex(), kp2.public_key_hex());
        assert_eq!(kp.signing_key_bytes(), kp2.signing_key_bytes());
        assert_eq!(kp.verifying_key_bytes(), kp2.verifying_key_bytes());

        // Same signature for same content
        let content = "roundtrip test";
        assert_eq!(kp.sign(content), kp2.sign(content));
    }

    #[test]
    fn test_sha256_hash() {
        let hash = sha256_hash("hello");
        // SHA-256 produces 32 bytes = 64 hex chars
        assert_eq!(hash.len(), 64);
        // Known SHA-256 of "hello"
        assert_eq!(
            hash,
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn test_clone_key_pair() {
        let kp = CommKeyPair::generate();
        let kp_clone = kp.clone();

        assert_eq!(kp.public_key_hex(), kp_clone.public_key_hex());
        assert_eq!(kp.signing_key_bytes(), kp_clone.signing_key_bytes());

        // Cloned key produces the same signature
        let content = "clone test content";
        assert_eq!(kp.sign(content), kp_clone.sign(content));
    }

    #[test]
    fn test_signature_is_128_hex_chars() {
        let kp = CommKeyPair::generate();
        let sig = kp.sign("any content");
        // Ed25519 signature is 64 bytes = 128 hex chars
        assert_eq!(sig.len(), 128);
        assert!(sig.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_verify_invalid_hex_returns_false() {
        assert!(!verify_signature("not-hex", "content", "also-not-hex"));
        assert!(!verify_signature("abcd", "content", "abcd"));
    }
}
