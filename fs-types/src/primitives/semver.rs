//! `SemVer` — a semantic version number.
//!
//! Replaces the plain `String` used for `version` in `ResourceMeta` and
//! `InstalledRecord`.  Ensures every version is structurally valid and
//! allows correct ordering (`0.9.0 < 1.0.0-beta.1 < 1.0.0`).
//!
//! # Format
//!
//! `MAJOR.MINOR.PATCH[-PRE]`
//!
//! - `MAJOR`, `MINOR`, `PATCH` — non-negative integers
//! - `PRE` — optional pre-release label, e.g. `alpha.1`, `beta.2`, `rc.1`
//!
//! # Serialization
//!
//! Serializes transparently as a plain string:
//! ```toml
//! version = "1.2.3"
//! version = "0.5.0-beta.1"
//! ```

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use super::FsValue;

// ── SemVer ────────────────────────────────────────────────────────────────────

/// A semantic version number (`MAJOR.MINOR.PATCH[-PRE]`).
///
/// Correct ordering: pre-release versions are less than the release:
/// `1.0.0-alpha.1 < 1.0.0-beta.1 < 1.0.0-rc.1 < 1.0.0`
///
/// # Example
///
/// ```rust
/// use fs_types::primitives::SemVer;
///
/// let v: SemVer = "1.2.3".parse().unwrap();
/// assert_eq!(v.major, 1);
/// assert_eq!(v.minor, 2);
/// assert_eq!(v.patch, 3);
/// assert!(v.pre.is_none());
///
/// let pre: SemVer = "0.5.0-beta.1".parse().unwrap();
/// assert!(pre < "0.5.0".parse::<SemVer>().unwrap());
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemVer {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
    /// Optional pre-release label, e.g. `"alpha.1"`, `"beta.2"`, `"rc.1"`.
    pub pre: Option<String>,
}

impl SemVer {
    /// Create a release version (no pre-release label).
    #[must_use]
    pub fn release(major: u16, minor: u16, patch: u16) -> Self {
        Self {
            major,
            minor,
            patch,
            pre: None,
        }
    }

    /// Create a pre-release version.
    pub fn pre_release(major: u16, minor: u16, patch: u16, pre: impl Into<String>) -> Self {
        Self {
            major,
            minor,
            patch,
            pre: Some(pre.into()),
        }
    }

    /// Returns `true` when this is a pre-release version.
    #[must_use]
    pub fn is_pre_release(&self) -> bool {
        self.pre.is_some()
    }

    /// Returns `true` when this is a stable release (`major >= 1`, no pre-release).
    #[must_use]
    pub fn is_stable(&self) -> bool {
        self.major >= 1 && self.pre.is_none()
    }
}

// ── Ordering ──────────────────────────────────────────────────────────────────

impl PartialOrd for SemVer {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SemVer {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Compare numeric components first.
        let by_numbers =
            (self.major, self.minor, self.patch).cmp(&(other.major, other.minor, other.patch));

        if by_numbers != std::cmp::Ordering::Equal {
            return by_numbers;
        }

        // Same numeric version: a pre-release is LESS than the release.
        // `1.0.0-alpha` < `1.0.0`
        match (&self.pre, &other.pre) {
            (None, None) => std::cmp::Ordering::Equal,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (Some(_), None) => std::cmp::Ordering::Less,
            (Some(a), Some(b)) => a.cmp(b),
        }
    }
}

// ── Display / FromStr ─────────────────────────────────────────────────────────

impl fmt::Display for SemVer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if let Some(pre) = &self.pre {
            write!(f, "-{pre}")?;
        }
        Ok(())
    }
}

/// Parse error for `SemVer::from_str`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemVerParseError(String);

impl fmt::Display for SemVerParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid semver: {}", self.0)
    }
}

impl std::error::Error for SemVerParseError {}

impl FromStr for SemVer {
    type Err = SemVerParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let err = || SemVerParseError(s.to_owned());

        // Split off optional pre-release suffix.
        let (core, pre) = match s.split_once('-') {
            Some((core, pre)) => (core, Some(pre.to_owned())),
            None => (s, None),
        };

        let mut parts = core.split('.');
        let major = parts
            .next()
            .ok_or_else(err)?
            .parse::<u16>()
            .map_err(|_| err())?;
        let minor = parts
            .next()
            .ok_or_else(err)?
            .parse::<u16>()
            .map_err(|_| err())?;
        let patch = parts
            .next()
            .ok_or_else(err)?
            .parse::<u16>()
            .map_err(|_| err())?;

        // Disallow trailing components like "1.2.3.4".
        if parts.next().is_some() {
            return Err(err());
        }

        Ok(SemVer {
            major,
            minor,
            patch,
            pre,
        })
    }
}

// ── Serde — serialize as plain string ─────────────────────────────────────────

impl Serialize for SemVer {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for SemVer {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

// ── FsValue ───────────────────────────────────────────────────────────────────

impl FsValue for SemVer {
    fn type_label_key(&self) -> &'static str {
        "type-semver"
    }

    fn placeholder_key(&self) -> &'static str {
        "placeholder-semver"
    }

    fn help_key(&self) -> &'static str {
        "help-semver"
    }

    fn validate(&self) -> Result<(), &'static str> {
        // A constructed SemVer is always valid; this covers the pre-release label.
        if let Some(pre) = &self.pre {
            if pre.is_empty() {
                return Err("error-validation-semver-pre-empty");
            }
            let valid = pre
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-');
            if !valid {
                return Err("error-validation-semver-pre-chars");
            }
        }
        Ok(())
    }

    fn display(&self) -> String {
        self.to_string()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_release() {
        let v: SemVer = "1.2.3".parse().unwrap();
        assert_eq!(v, SemVer::release(1, 2, 3));
        assert!(!v.is_pre_release());
        assert!(v.is_stable());
    }

    #[test]
    fn parse_pre_release() {
        let v: SemVer = "0.5.0-beta.1".parse().unwrap();
        assert_eq!(v.major, 0u16);
        assert_eq!(v.pre, Some("beta.1".into()));
        assert!(v.is_pre_release());
        assert!(!v.is_stable());
    }

    #[test]
    fn parse_invalid() {
        assert!("1.2".parse::<SemVer>().is_err());
        assert!("1.2.3.4".parse::<SemVer>().is_err());
        assert!("one.two.three".parse::<SemVer>().is_err());
        assert!("".parse::<SemVer>().is_err());
    }

    #[test]
    fn display_roundtrip() {
        for s in &["1.0.0", "0.5.0-beta.1", "2.10.3-rc.2"] {
            let v: SemVer = s.parse().unwrap();
            assert_eq!(v.to_string(), *s);
        }
    }

    #[test]
    fn ordering_release_gt_pre() {
        let release: SemVer = "1.0.0".parse().unwrap();
        let pre: SemVer = "1.0.0-rc.1".parse().unwrap();
        assert!(release > pre);
    }

    #[test]
    fn ordering_major_minor_patch() {
        let v1: SemVer = "1.0.0".parse().unwrap();
        let v2: SemVer = "2.0.0".parse().unwrap();
        let v3: SemVer = "1.1.0".parse().unwrap();
        let v4: SemVer = "1.0.1".parse().unwrap();
        assert!(v1 < v2);
        assert!(v1 < v3);
        assert!(v1 < v4);
        assert!(v3 < v2);
    }

    #[test]
    fn ordering_pre_releases_lexicographic() {
        let alpha: SemVer = "1.0.0-alpha.1".parse().unwrap();
        let beta: SemVer = "1.0.0-beta.1".parse().unwrap();
        assert!(alpha < beta);
    }

    #[test]
    fn serde_roundtrip() {
        let v = SemVer::release(1, 2, 3);
        let json = serde_json::to_string(&v).unwrap();
        assert_eq!(json, "\"1.2.3\"");
        let back: SemVer = serde_json::from_str(&json).unwrap();
        assert_eq!(v, back);
    }

    #[test]
    fn serde_pre_release() {
        let v: SemVer = "0.5.0-beta.1".parse().unwrap();
        let json = serde_json::to_string(&v).unwrap();
        let back: SemVer = serde_json::from_str(&json).unwrap();
        assert_eq!(v, back);
    }

    #[test]
    fn validate_ok() {
        assert!(SemVer::release(1, 0, 0).validate().is_ok());
        assert!("1.0.0-rc.1".parse::<SemVer>().unwrap().validate().is_ok());
    }

    #[test]
    fn fsvalue_keys() {
        let v = SemVer::release(1, 0, 0);
        assert_eq!(v.type_label_key(), "type-semver");
        assert_eq!(v.placeholder_key(), "placeholder-semver");
        assert_eq!(v.help_key(), "help-semver");
    }
}
