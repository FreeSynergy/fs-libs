#![deny(clippy::all, clippy::pedantic, warnings)]
//! HMAC-SHA256 provider for invite tokens and message authentication.
//!
//! Used for `InviteToken` authentication in `fs-node` where a shared key
//! is available on both issuer and verifier sides.

use fs_error::FsError;
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};

use crate::provider::CryptoProvider;

type HmacSha256 = Hmac<Sha256>;

// ── HmacProvider ─────────────────────────────────────────────────────────────

/// `HMAC-SHA256` provider for message authentication and invite-token signing.
///
/// Both issuer and verifier must share the same secret key.
/// Tags are 32 bytes (256 bits).
pub struct HmacProvider {
    key: Vec<u8>,
}

impl HmacProvider {
    /// Create a new provider with the given secret key.
    ///
    /// HMAC accepts any key length.
    pub fn new(key: impl AsRef<[u8]>) -> Self {
        Self {
            key: key.as_ref().to_vec(),
        }
    }

    /// Compute an HMAC-SHA256 tag over `data`. Returns 32 bytes.
    ///
    /// # Panics
    /// Never panics — HMAC accepts any key length.
    #[must_use]
    pub fn sign_bytes(&self, data: &[u8]) -> Vec<u8> {
        let mut mac = HmacSha256::new_from_slice(&self.key).expect("HMAC accepts any key length");
        mac.update(data);
        mac.finalize().into_bytes().to_vec()
    }

    /// Verify an HMAC-SHA256 `tag` over `data`.
    ///
    /// Uses constant-time comparison to prevent timing attacks.
    ///
    /// # Errors
    /// Returns `FsError::Auth` when the tag is invalid.
    pub fn verify_bytes(&self, data: &[u8], tag: &[u8]) -> Result<(), FsError> {
        let mut mac = HmacSha256::new_from_slice(&self.key)
            .map_err(|e| FsError::internal(format!("HMAC key invalid: {e}")))?;
        mac.update(data);
        mac.verify_slice(tag)
            .map_err(|_| FsError::auth("HMAC-SHA256 tag verification failed"))
    }
}

// ── CryptoProvider impl ───────────────────────────────────────────────────────

impl CryptoProvider for HmacProvider {
    fn sign(&self, data: &[u8]) -> Result<Vec<u8>, FsError> {
        Ok(self.sign_bytes(data))
    }

    fn verify(&self, data: &[u8], signature: &[u8]) -> Result<(), FsError> {
        self.verify_bytes(data, signature)
    }

    fn hash(&self, data: &[u8]) -> Result<Vec<u8>, FsError> {
        Ok(Sha256::digest(data).to_vec())
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_verify_roundtrip() {
        let provider = HmacProvider::new(b"super-secret-key");
        let data = b"invite:node-01:1234567890";
        let tag = provider.sign_bytes(data);
        assert!(provider.verify_bytes(data, &tag).is_ok());
    }

    #[test]
    fn wrong_key_fails_verification() {
        let signer = HmacProvider::new(b"correct-key");
        let verifier = HmacProvider::new(b"wrong-key");
        let tag = signer.sign_bytes(b"data");
        assert!(verifier.verify_bytes(b"data", &tag).is_err());
    }

    #[test]
    fn tampered_data_fails_verification() {
        let provider = HmacProvider::new(b"key");
        let tag = provider.sign_bytes(b"original");
        assert!(provider.verify_bytes(b"tampered", &tag).is_err());
    }

    #[test]
    fn sign_is_deterministic() {
        let provider = HmacProvider::new(b"key");
        let t1 = provider.sign_bytes(b"hello");
        let t2 = provider.sign_bytes(b"hello");
        assert_eq!(t1, t2);
    }

    #[test]
    fn different_data_different_tag() {
        let provider = HmacProvider::new(b"key");
        let t1 = provider.sign_bytes(b"message-a");
        let t2 = provider.sign_bytes(b"message-b");
        assert_ne!(t1, t2);
    }

    #[test]
    fn tag_is_32_bytes() {
        let provider = HmacProvider::new(b"key");
        let tag = provider.sign_bytes(b"data");
        assert_eq!(tag.len(), 32);
    }

    #[test]
    fn crypto_provider_sign_verify() {
        let p: &dyn CryptoProvider = &HmacProvider::new(b"key");
        let tag = p.sign(b"data").unwrap();
        assert!(p.verify(b"data", &tag).is_ok());
        assert!(p.verify(b"wrong", &tag).is_err());
    }

    #[test]
    fn crypto_provider_hash() {
        let p = HmacProvider::new(b"key");
        let h = p.hash(b"hello").unwrap();
        assert_eq!(h.len(), 32);
        // SHA-256 is deterministic
        assert_eq!(h, p.hash(b"hello").unwrap());
        assert_ne!(h, p.hash(b"world").unwrap());
    }
}
