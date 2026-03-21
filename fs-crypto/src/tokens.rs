//! Join-token generation and verification for FSN node federation.
//!
//! A join token allows a new node to securely join an existing FSN cluster.
//! The token encodes the issuing node's address, an expiry timestamp, and a
//! random nonce, all concatenated and base64url-encoded.
//!
//! # Format
//! `fsn1.<base64url(node_id|address|expires_unix|nonce_hex)>`
//!
//! Fields are separated by `|` to avoid conflicts with `:` in IPv4 `host:port`
//! or IPv6 addresses.
//!
//! # Example
//! ```rust
//! # #[cfg(feature = "tokens")]
//! # {
//! use fs_crypto::tokens::{JoinToken, JoinTokenError};
//!
//! let token = JoinToken::generate("node-01", "192.168.1.1:7000", 3600);
//! assert!(token.to_string().starts_with("fsn1."));
//! assert!(token.verify("192.168.1.1:7000").is_ok());
//! # }
//! ```

use rand::RngCore;
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

// ── JoinTokenError ────────────────────────────────────────────────────────────

/// Errors that can occur during join-token parsing or verification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JoinTokenError {
    /// The token has passed its expiry time.
    Expired,
    /// The token's embedded address does not match the expected address.
    AddressMismatch,
    /// The token string is malformed and cannot be parsed.
    InvalidFormat,
}

impl fmt::Display for JoinTokenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JoinTokenError::Expired => write!(f, "join token has expired"),
            JoinTokenError::AddressMismatch => write!(f, "join token address mismatch"),
            JoinTokenError::InvalidFormat => write!(f, "join token has invalid format"),
        }
    }
}

impl std::error::Error for JoinTokenError {}

// ── JoinToken ─────────────────────────────────────────────────────────────────

/// A signed join token for FSN cluster federation.
///
/// Encodes the issuing node's identifier and address, an expiry timestamp
/// (Unix seconds), and a random nonce to prevent replay.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JoinToken {
    /// Identifier of the node that issued the token.
    pub node_id: String,
    /// Network address of the issuing node (e.g. `"192.168.1.1:7000"`).
    pub address: String,
    /// Expiry timestamp as Unix seconds (UTC).
    pub expires_at: u64,
    /// Random nonce (hex-encoded).
    pub nonce: String,
}

impl JoinToken {
    /// Generate a new join token for `node_id` at `address` valid for `ttl_secs` seconds.
    pub fn generate(node_id: &str, address: &str, ttl_secs: u64) -> Self {
        let expires_at = unix_now() + ttl_secs;
        let nonce = random_nonce();
        Self {
            node_id: node_id.to_string(),
            address: address.to_string(),
            expires_at,
            nonce,
        }
    }

    /// Parse a token string in `fsn1.<base64url(...)>` format.
    ///
    /// Returns `Err(JoinTokenError::InvalidFormat)` if the string is malformed.
    pub fn parse(token_str: &str) -> Result<Self, JoinTokenError> {
        let payload = token_str
            .strip_prefix("fsn1.")
            .ok_or(JoinTokenError::InvalidFormat)?;

        let decoded = base64url_decode(payload).map_err(|_| JoinTokenError::InvalidFormat)?;
        let text = String::from_utf8(decoded).map_err(|_| JoinTokenError::InvalidFormat)?;

        // Fields are separated by `|` to avoid conflicts with `:` in addresses.
        let parts: Vec<&str> = text.splitn(4, '|').collect();
        if parts.len() != 4 {
            return Err(JoinTokenError::InvalidFormat);
        }

        let expires_at: u64 = parts[2]
            .parse()
            .map_err(|_| JoinTokenError::InvalidFormat)?;

        Ok(Self {
            node_id: parts[0].to_string(),
            address: parts[1].to_string(),
            expires_at,
            nonce: parts[3].to_string(),
        })
    }

    /// Verify that the token is not expired and matches `expected_address`.
    ///
    /// Returns `Ok(())` when the token is valid, otherwise a [`JoinTokenError`].
    pub fn verify(&self, expected_address: &str) -> Result<(), JoinTokenError> {
        if self.is_expired() {
            return Err(JoinTokenError::Expired);
        }
        if self.address != expected_address {
            return Err(JoinTokenError::AddressMismatch);
        }
        Ok(())
    }

    /// Return `true` if the token has passed its expiry time.
    pub fn is_expired(&self) -> bool {
        unix_now() >= self.expires_at
    }
}

impl fmt::Display for JoinToken {
    /// Encode the token as `fsn1.<base64url(node_id|address|expires_unix|nonce_hex)>`.
    ///
    /// Fields are separated by `|` to avoid conflicts with `:` in IPv6 or host:port addresses.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let payload = format!(
            "{}|{}|{}|{}",
            self.node_id, self.address, self.expires_at, self.nonce
        );
        write!(f, "fsn1.{}", base64url_encode(payload.as_bytes()))
    }
}

// ── Recovery token ────────────────────────────────────────────────────────────

/// Generate a recovery token for node disaster recovery.
///
/// Returns a random 32-byte hex string (64 hex characters) suitable for
/// one-time node recovery flows.
pub fn generate_recovery_token() -> String {
    let mut buf = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut buf);
    hex::encode(buf)
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn random_nonce() -> String {
    let mut buf = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut buf);
    hex::encode(buf)
}

/// Encode `input` as base64url without padding.
fn base64url_encode(input: &[u8]) -> String {
    const ALPHABET: &[u8] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut output = String::with_capacity((input.len() * 4 + 2) / 3);
    for chunk in input.chunks(3) {
        let b0 = chunk[0] as usize;
        let b1 = if chunk.len() > 1 { chunk[1] as usize } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as usize } else { 0 };

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

/// Decode a base64url string (no padding) into bytes.
fn base64url_decode(input: &str) -> Result<Vec<u8>, ()> {
    const DECODE: [i8; 256] = {
        let mut table = [-1i8; 256];
        let mut i = 0u8;
        loop {
            let ch = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_"
                [i as usize];
            table[ch as usize] = i as i8;
            i += 1;
            if i == 64 {
                break;
            }
        }
        table
    };

    let bytes = input.as_bytes();
    let mut output = Vec::with_capacity(bytes.len() * 3 / 4);
    let mut i = 0;

    while i < bytes.len() {
        let remaining = bytes.len() - i;
        if remaining < 2 {
            return Err(());
        }

        let v0 = DECODE[bytes[i] as usize];
        let v1 = DECODE[bytes[i + 1] as usize];
        if v0 < 0 || v1 < 0 {
            return Err(());
        }
        output.push(((v0 as u8) << 2) | ((v1 as u8) >> 4));

        if remaining >= 3 {
            let v2 = DECODE[bytes[i + 2] as usize];
            if v2 < 0 {
                return Err(());
            }
            output.push(((v1 as u8 & 0xf) << 4) | ((v2 as u8) >> 2));

            if remaining >= 4 {
                let v3 = DECODE[bytes[i + 3] as usize];
                if v3 < 0 {
                    return Err(());
                }
                output.push(((v2 as u8 & 0x3) << 6) | (v3 as u8));
                i += 4;
            } else {
                i += 3;
            }
        } else {
            i += 2;
        }
    }

    Ok(output)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_starts_with_prefix() {
        let token = JoinToken::generate("node-01", "192.168.1.1:7000", 3600);
        assert!(token.to_string().starts_with("fsn1."));
    }

    #[test]
    fn parse_round_trip() {
        let token = JoinToken::generate("node-01", "10.0.0.1:7000", 3600);
        let encoded = token.to_string();
        let parsed = JoinToken::parse(&encoded).expect("parse should succeed");
        assert_eq!(parsed.node_id, "node-01");
        assert_eq!(parsed.address, "10.0.0.1:7000");
        assert_eq!(parsed.expires_at, token.expires_at);
        assert_eq!(parsed.nonce, token.nonce);
    }

    #[test]
    fn verify_ok() {
        let token = JoinToken::generate("node-01", "192.168.1.1:7000", 3600);
        assert!(token.verify("192.168.1.1:7000").is_ok());
    }

    #[test]
    fn verify_expired() {
        // TTL = 0 → expires_at = unix_now(), which is immediately expired
        let token = JoinToken {
            node_id: "node-01".into(),
            address: "192.168.1.1:7000".into(),
            expires_at: 1, // Unix epoch + 1s — definitely expired
            nonce: "aabbccdd".into(),
        };
        assert_eq!(token.verify("192.168.1.1:7000"), Err(JoinTokenError::Expired));
    }

    #[test]
    fn verify_wrong_address() {
        let token = JoinToken::generate("node-01", "192.168.1.1:7000", 3600);
        assert_eq!(
            token.verify("10.0.0.2:7000"),
            Err(JoinTokenError::AddressMismatch)
        );
    }

    #[test]
    fn parse_invalid_format_no_prefix() {
        assert_eq!(
            JoinToken::parse("notavalidtoken"),
            Err(JoinTokenError::InvalidFormat)
        );
    }

    #[test]
    fn parse_invalid_format_bad_base64() {
        assert_eq!(
            JoinToken::parse("fsn1.!!!invalid!!!"),
            Err(JoinTokenError::InvalidFormat)
        );
    }

    #[test]
    fn recovery_token_length() {
        let t = generate_recovery_token();
        // 32 bytes → 64 hex chars
        assert_eq!(t.len(), 64);
        assert!(t.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn recovery_token_unique() {
        assert_ne!(generate_recovery_token(), generate_recovery_token());
    }
}
