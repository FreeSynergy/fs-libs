//! `FsUrl` — a URL with a human-readable display label.
//!
//! Used wherever a link needs both a target and a visible text:
//! package documentation links, external project websites, FTL help files.
//!
//! # Serialization
//!
//! TOML / JSON:
//! ```toml
//! website = { url = "https://freesynergy.net", label = "FreeSynergy" }
//! ```
//!
//! When only a URL string is needed (no label), use a plain `String`.
//! `FsUrl` is for display-rich contexts.

use serde::{Deserialize, Serialize};

use super::FsValue;

// ── FsUrl ─────────────────────────────────────────────────────────────────────

/// A URL paired with a human-readable display label.
///
/// The `label` is shown to users instead of the raw URL wherever space permits.
/// When `label` is empty, the `url` is used as the display text.
///
/// # Example
///
/// ```rust
/// use fs_types::primitives::FsUrl;
///
/// let link = FsUrl::new("https://freesynergy.net", "FreeSynergy");
/// assert_eq!(link.display_label(), "FreeSynergy");
///
/// let bare = FsUrl::from_url("https://freesynergy.net");
/// assert_eq!(bare.display_label(), "https://freesynergy.net");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FsUrl {
    /// The full URL, e.g. `"https://github.com/FreeSynergy/fs-node"`.
    pub url: String,
    /// Human-readable display text, e.g. `"FreeSynergy Node"`.
    ///
    /// When empty, [`display_label`](Self::display_label) falls back to `url`.
    pub label: String,
}

impl FsUrl {
    /// Create an `FsUrl` with both a URL and a display label.
    pub fn new(url: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            label: label.into(),
        }
    }

    /// Create an `FsUrl` from a URL only; the label will be the URL itself.
    pub fn from_url(url: impl Into<String>) -> Self {
        let url = url.into();
        Self {
            label: url.clone(),
            url,
        }
    }

    /// The text to show to the user: label if set, otherwise the raw URL.
    #[must_use]
    pub fn display_label(&self) -> &str {
        if self.label.is_empty() {
            &self.url
        } else {
            &self.label
        }
    }

    /// Returns `true` when the URL uses HTTPS.
    #[must_use]
    pub fn is_https(&self) -> bool {
        self.url.starts_with("https://")
    }
}

impl FsValue for FsUrl {
    fn type_label_key(&self) -> &'static str {
        "type-url"
    }

    fn placeholder_key(&self) -> &'static str {
        "placeholder-url"
    }

    fn help_key(&self) -> &'static str {
        "help-url"
    }

    fn validate(&self) -> Result<(), &'static str> {
        if self.url.is_empty() {
            return Err("error-validation-url-empty");
        }
        if !self.url.starts_with("http://") && !self.url.starts_with("https://") {
            return Err("error-validation-url-scheme");
        }
        Ok(())
    }

    fn display(&self) -> String {
        self.display_label().to_owned()
    }
}

impl std::fmt::Display for FsUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.display_label(), self.url)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_label_uses_label_when_set() {
        let u = FsUrl::new("https://example.com", "Example");
        assert_eq!(u.display_label(), "Example");
    }

    #[test]
    fn display_label_falls_back_to_url() {
        let u = FsUrl::from_url("https://example.com");
        assert_eq!(u.display_label(), "https://example.com");
    }

    #[test]
    fn validate_valid_https_url() {
        let u = FsUrl::new("https://example.com", "Example");
        assert!(u.validate().is_ok());
    }

    #[test]
    fn validate_valid_http_url() {
        let u = FsUrl::from_url("http://localhost:8080");
        assert!(u.validate().is_ok());
    }

    #[test]
    fn validate_empty_url() {
        let u = FsUrl::new("", "label");
        assert_eq!(u.validate(), Err("error-validation-url-empty"));
    }

    #[test]
    fn validate_bad_scheme() {
        let u = FsUrl::from_url("ftp://example.com");
        assert_eq!(u.validate(), Err("error-validation-url-scheme"));
    }

    #[test]
    fn is_https() {
        assert!(FsUrl::from_url("https://x.com").is_https());
        assert!(!FsUrl::from_url("http://x.com").is_https());
    }

    #[test]
    fn fsvalue_keys() {
        let u = FsUrl::from_url("https://x.com");
        assert_eq!(u.type_label_key(), "type-url");
        assert_eq!(u.placeholder_key(), "placeholder-url");
        assert_eq!(u.help_key(), "help-url");
    }

    #[test]
    fn serde_roundtrip() {
        let u = FsUrl::new("https://example.com", "Example Site");
        let json = serde_json::to_string(&u).unwrap();
        let back: FsUrl = serde_json::from_str(&json).unwrap();
        assert_eq!(u, back);
    }
}
