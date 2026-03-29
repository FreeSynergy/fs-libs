//! `FsPort` — a validated TCP/UDP port number (1–65535).
//!
//! Wraps `u16` to document intent and prevent `0` being used as a port.
//! Serializes transparently as a plain integer.
//!
//! # Serialization
//!
//! ```toml
//! port = 8080
//! ```

use serde::{Deserialize, Serialize};

use super::FsValue;

// ── FsPort ────────────────────────────────────────────────────────────────────

/// A validated TCP/UDP port number in the range 1–65535.
///
/// Port `0` is invalid — the OS uses it internally for dynamic assignment and
/// it must never appear in configuration.
///
/// # Example
///
/// ```rust
/// use fs_types::primitives::FsPort;
/// use fs_types::primitives::FsValue;
///
/// let p = FsPort::new(8080);
/// assert!(p.validate().is_ok());
///
/// let zero = FsPort::new(0);
/// assert!(zero.validate().is_err());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FsPort(pub u16);

impl FsPort {
    /// Wrap a raw `u16` as an `FsPort`.
    ///
    /// Does not validate — call [`FsValue::validate`] to check.
    #[must_use]
    pub fn new(port: u16) -> Self {
        Self(port)
    }

    /// Return the raw port number.
    #[must_use]
    pub fn value(self) -> u16 {
        self.0
    }

    /// Returns `true` when this is a well-known port (< 1024).
    #[must_use]
    pub fn is_privileged(self) -> bool {
        self.0 < 1024
    }
}

impl FsValue for FsPort {
    fn type_label_key(&self) -> &'static str {
        "type-port"
    }

    fn placeholder_key(&self) -> &'static str {
        "placeholder-port"
    }

    fn help_key(&self) -> &'static str {
        "help-port"
    }

    fn validate(&self) -> Result<(), &'static str> {
        if self.0 == 0 {
            return Err("error-validation-port-zero");
        }
        Ok(())
    }

    fn display(&self) -> String {
        self.0.to_string()
    }
}

impl std::fmt::Display for FsPort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u16> for FsPort {
    fn from(p: u16) -> Self {
        Self(p)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_port() {
        assert!(FsPort::new(8080).validate().is_ok());
        assert!(FsPort::new(443).validate().is_ok());
        assert!(FsPort::new(65535).validate().is_ok());
        assert!(FsPort::new(1).validate().is_ok());
    }

    #[test]
    fn zero_is_invalid() {
        assert_eq!(FsPort::new(0).validate(), Err("error-validation-port-zero"));
    }

    #[test]
    fn privileged_detection() {
        assert!(FsPort::new(80).is_privileged());
        assert!(FsPort::new(443).is_privileged());
        assert!(!FsPort::new(1024).is_privileged());
        assert!(!FsPort::new(8080).is_privileged());
    }

    #[test]
    fn display() {
        assert_eq!(FsPort::new(8080).to_string(), "8080");
        assert_eq!(FsPort::new(8080).display(), "8080");
    }

    #[test]
    fn serde_transparent() {
        let p = FsPort::new(8080);
        let json = serde_json::to_string(&p).unwrap();
        assert_eq!(json, "8080");
        let back: FsPort = serde_json::from_str(&json).unwrap();
        assert_eq!(p, back);
    }

    #[test]
    fn ordering() {
        assert!(FsPort::new(80) < FsPort::new(443));
        assert!(FsPort::new(8080) > FsPort::new(3000));
    }

    #[test]
    fn fsvalue_keys() {
        let p = FsPort::new(8080);
        assert_eq!(p.type_label_key(), "type-port");
        assert_eq!(p.placeholder_key(), "placeholder-port");
        assert_eq!(p.help_key(), "help-port");
    }
}
