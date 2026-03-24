//! Random secret generation and key derivation helpers.

use pbkdf2::pbkdf2_hmac;
use rand::RngCore;
use sha2::Sha256;

// ── KeyGen ────────────────────────────────────────────────────────────────────

pub struct KeyGen;

impl KeyGen {
    /// Generate `n` cryptographically random bytes.
    pub fn random_bytes(n: usize) -> Vec<u8> {
        let mut buf = vec![0u8; n];
        rand::rng().fill_bytes(&mut buf);
        buf
    }

    /// Generate a random secret string of `len` bytes, base64url-encoded (no padding).
    pub fn random_secret(len: usize) -> String {
        Self::base64url_encode(&Self::random_bytes(len))
    }

    /// Generate a random hex string of `len` bytes (hex-encoded = 2×`len` chars).
    pub fn random_hex(len: usize) -> String {
        hex::encode(Self::random_bytes(len))
    }

    /// Derive a key from a password and salt using PBKDF2-HMAC-SHA256.
    ///
    /// Returns a 32-byte derived key.
    pub fn derive_key(password: &[u8], salt: &[u8], iterations: u32) -> [u8; 32] {
        let mut key = [0u8; 32];
        pbkdf2_hmac::<Sha256>(password, salt, iterations, &mut key);
        key
    }

    /// Encode `input` as base64url without padding.
    fn base64url_encode(input: &[u8]) -> String {
        const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
        let mut output = String::with_capacity((input.len() * 4 + 2) / 3);
        for chunk in input.chunks(3) {
            let b0 = chunk[0] as usize;
            let b1 = if chunk.len() > 1 {
                chunk[1] as usize
            } else {
                0
            };
            let b2 = if chunk.len() > 2 {
                chunk[2] as usize
            } else {
                0
            };

            output.push(ALPHABET[b0 >> 2] as char);
            output.push(ALPHABET[((b0 & 0x3) << 4) | (b1 >> 4)] as char);
            if chunk.len() > 1 {
                output.push(ALPHABET[((b1 & 0xf) << 2) | (b2 >> 6)] as char);
            }
            if chunk.len() > 2 {
                output.push(ALPHABET[b2 & 0x3f] as char);
            }
        }
        output
    }
}

// ── Public shims ──────────────────────────────────────────────────────────────

pub fn random_bytes(n: usize) -> Vec<u8> {
    KeyGen::random_bytes(n)
}
pub fn random_secret(len: usize) -> String {
    KeyGen::random_secret(len)
}
pub fn random_hex(len: usize) -> String {
    KeyGen::random_hex(len)
}

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
        // P(all zeros for 32 bytes) ≈ 0
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

    #[test]
    fn base64url_known_vector() {
        // RFC 4648: "Man" → "TWFu"
        assert_eq!(base64url_encode(b"Man"), "TWFu");
    }
}
