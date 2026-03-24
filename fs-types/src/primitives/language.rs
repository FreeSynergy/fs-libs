//! `LanguageCode` — a validated BCP-47 language code.
//!
//! Used in package metadata to declare which languages a package supports.
//!
//! # Difference from `fs_i18n::LanguageCode`
//!
//! `fs_i18n::LanguageCode` is the *runtime active language* of the i18n system.
//! This `LanguageCode` is a *data type* for package manifests and metadata —
//! it lives in `fs-types` so it can be used without pulling in the full i18n stack.
//!
//! # Serialization
//!
//! Serializes transparently as a plain string:
//! ```toml
//! locales = ["en", "de", "ar"]
//! ```

use serde::{Deserialize, Serialize};

use super::FsValue;

// ── LanguageCode ──────────────────────────────────────────────────────────────

/// A BCP-47 language code, e.g. `"en"`, `"de"`, `"ar"`, `"yue"`.
///
/// Validates that the code is non-empty and contains only ASCII letters,
/// digits, and hyphens (the allowed BCP-47 characters).
///
/// # Example
///
/// ```rust
/// use fs_types::primitives::{LanguageCode, FsValue};
///
/// let de = LanguageCode::new("de");
/// assert!(de.validate().is_ok());
///
/// let bad = LanguageCode::new("");
/// assert!(bad.validate().is_err());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct LanguageCode(String);

impl LanguageCode {
    /// Create a `LanguageCode` from any string-like value.
    ///
    /// Does not validate — call [`validate`](FsValue::validate) to check.
    pub fn new(code: impl Into<String>) -> Self {
        Self(code.into())
    }

    /// Borrow the code as a `&str`, e.g. `"de"`.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns `true` for right-to-left language codes (`ar`, `fa`, `ur`, `ps`).
    ///
    /// This is a lightweight check based on the known RTL codes.
    /// For full script metadata, use `fs_i18n::language_meta(code)`.
    pub fn is_rtl(&self) -> bool {
        matches!(self.0.as_str(), "ar" | "fa" | "ur" | "ps")
    }
}

impl FsValue for LanguageCode {
    fn type_label_key(&self) -> &'static str {
        "type.language_code"
    }

    fn placeholder_key(&self) -> &'static str {
        "placeholder.language_code"
    }

    fn help_key(&self) -> &'static str {
        "help.language_code"
    }

    fn validate(&self) -> Result<(), &'static str> {
        if self.0.is_empty() {
            return Err("error.validation.language_code.empty");
        }
        let valid = self
            .0
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-');
        if !valid {
            return Err("error.validation.language_code.chars");
        }
        Ok(())
    }

    fn display(&self) -> String {
        self.0.clone()
    }
}

impl std::fmt::Display for LanguageCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<&str> for LanguageCode {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

impl From<String> for LanguageCode {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl AsRef<str> for LanguageCode {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_simple_code() {
        assert!(LanguageCode::new("en").validate().is_ok());
        assert!(LanguageCode::new("de").validate().is_ok());
        assert!(LanguageCode::new("zh-Hant").validate().is_ok());
        assert!(LanguageCode::new("yue").validate().is_ok());
    }

    #[test]
    fn empty_code_invalid() {
        assert_eq!(
            LanguageCode::new("").validate(),
            Err("error.validation.language_code.empty")
        );
    }

    #[test]
    fn invalid_chars() {
        assert_eq!(
            LanguageCode::new("de DE").validate(),
            Err("error.validation.language_code.chars")
        );
        assert_eq!(
            LanguageCode::new("de_DE").validate(),
            Err("error.validation.language_code.chars")
        );
    }

    #[test]
    fn rtl_detection() {
        assert!(LanguageCode::new("ar").is_rtl());
        assert!(LanguageCode::new("fa").is_rtl());
        assert!(LanguageCode::new("ur").is_rtl());
        assert!(LanguageCode::new("ps").is_rtl());
        assert!(!LanguageCode::new("de").is_rtl());
        assert!(!LanguageCode::new("en").is_rtl());
    }

    #[test]
    fn display_and_as_str() {
        let c = LanguageCode::new("de");
        assert_eq!(c.as_str(), "de");
        assert_eq!(c.to_string(), "de");
        assert_eq!(c.display(), "de");
    }

    #[test]
    fn serde_transparent() {
        let c = LanguageCode::new("de");
        let json = serde_json::to_string(&c).unwrap();
        assert_eq!(json, "\"de\"");
        let back: LanguageCode = serde_json::from_str(&json).unwrap();
        assert_eq!(c, back);
    }

    #[test]
    fn ordering() {
        let mut codes = [
            LanguageCode::new("zh"),
            LanguageCode::new("ar"),
            LanguageCode::new("de"),
        ];
        codes.sort();
        assert_eq!(codes[0].as_str(), "ar");
        assert_eq!(codes[1].as_str(), "de");
        assert_eq!(codes[2].as_str(), "zh");
    }

    #[test]
    fn fsvalue_keys() {
        let c = LanguageCode::new("en");
        assert_eq!(c.type_label_key(), "type.language_code");
        assert_eq!(c.placeholder_key(), "placeholder.language_code");
        assert_eq!(c.help_key(), "help.language_code");
    }
}
