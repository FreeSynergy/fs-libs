#![deny(clippy::all, clippy::pedantic, warnings)]
//! Random secret generation and key derivation helpers.

use pbkdf2::pbkdf2_hmac;
use rand::Rng;
use sha2::Sha256;

use crate::base64url;

// ── KeyGen ────────────────────────────────────────────────────────────────────

/// Stateless helper for random byte and key generation.
pub struct KeyGen;

impl KeyGen {
    /// Generate `n` cryptographically random bytes.
    #[must_use]
    pub fn random_bytes(n: usize) -> Vec<u8> {
        let mut buf = vec![0u8; n];
        rand::rng().fill_bytes(&mut buf);
        buf
    }

    /// Generate a random secret string of `len` bytes, base64url-encoded (no padding).
    #[must_use]
    pub fn random_secret(len: usize) -> String {
        base64url::encode(&Self::random_bytes(len))
    }

    /// Generate a random hex string of `len` bytes (hex-encoded = 2×`len` chars).
    #[must_use]
    pub fn random_hex(len: usize) -> String {
        hex::encode(Self::random_bytes(len))
    }

    /// Derive a key from a password and salt using PBKDF2-HMAC-SHA256.
    ///
    /// Returns a 32-byte derived key.
    #[must_use]
    pub fn derive_key(password: &[u8], salt: &[u8], iterations: u32) -> [u8; 32] {
        let mut key = [0u8; 32];
        pbkdf2_hmac::<Sha256>(password, salt, iterations, &mut key);
        key
    }
}

// ── Public shims ──────────────────────────────────────────────────────────────

/// Generate `n` cryptographically random bytes.
#[must_use]
pub fn random_bytes(n: usize) -> Vec<u8> {
    KeyGen::random_bytes(n)
}

/// Generate a random secret string of `len` bytes, base64url-encoded.
#[must_use]
pub fn random_secret(len: usize) -> String {
    KeyGen::random_secret(len)
}

/// Generate a random hex string of `len` bytes.
#[must_use]
pub fn random_hex(len: usize) -> String {
    KeyGen::random_hex(len)
}

/// Derive a 32-byte key via PBKDF2-HMAC-SHA256.
#[must_use]
pub fn derive_key(password: &[u8], salt: &[u8], iterations: u32) -> [u8; 32] {
    KeyGen::derive_key(password, salt, iterations)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random_bytes_length() {
        for n in [0, 1, 16, 32, 64] {
            assert_eq!(random_bytes(n).len(), n);
        }
    }

    #[test]
    fn random_bytes_not_all_zero() {
        assert!(!random_bytes(32).iter().all(|&b| b == 0));
    }

    #[test]
    fn random_secret_is_base64url() {
        let s = random_secret(32);
        assert!(s.len() >= 40);
        assert!(s
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_'));
    }

    #[test]
    fn random_secret_is_unique() {
        assert_ne!(random_secret(32), random_secret(32));
    }

    #[test]
    fn random_hex_length() {
        assert_eq!(random_hex(16).len(), 32);
        assert_eq!(random_hex(8).len(), 16);
    }

    #[test]
    fn random_hex_charset() {
        assert!(random_hex(32).chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn derive_key_deterministic() {
        assert_eq!(
            derive_key(b"password", b"salt", 1000),
            derive_key(b"password", b"salt", 1000)
        );
    }

    #[test]
    fn derive_key_differs_on_password() {
        assert_ne!(
            derive_key(b"password1", b"salt", 1000),
            derive_key(b"password2", b"salt", 1000)
        );
    }

    #[test]
    fn derive_key_differs_on_salt() {
        assert_ne!(
            derive_key(b"password", b"salt1", 1000),
            derive_key(b"password", b"salt2", 1000)
        );
    }
}
