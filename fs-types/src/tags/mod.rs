//! Tag system for FreeSynergy store resources.
//!
//! Tags are the primary search and filter instrument in the Store.
//! They must be **typed** (from a known library) and **translatable**
//! (i18n key, not a hardcoded string) so the display name works in every language
//! while the key stays stable across translations.
//!
//! # Design
//!
//! ```text
//! FsTag                ← a validated tag key (always lowercase, dot-separated)
//! TagLibrary trait     ← a set of known tag keys (implemented by each domain module)
//!   ├── PackageTags    ← for store packages: database, security, ai, …
//!   ├── PlatformTags   ← platform/system tags: linux, macos, requires-systemd, …
//!   └── ApiTags        ← API / protocol tags: rest, grpc, graphql, websocket, …
//! ```
//!
//! # i18n key convention
//!
//! Every `FsTag` key maps to an i18n key: `tag.<key>`.
//! Example: tag key `"package.database"` → i18n key `"tag.package.database"`
//! → translated as `"Database"` (en), `"Datenbank"` (de), …
//!
//! # Tests
//!
//! All tag library keys are checked in tests to ensure every key
//! follows the naming convention.

pub mod api;
pub mod package;
pub mod platform_tags;

pub use api::ApiTags;
pub use package::PackageTags;
pub use platform_tags::PlatformTags;

use serde::{Deserialize, Serialize};

// ── FsTag ─────────────────────────────────────────────────────────────────────

/// A typed store tag.
///
/// Tags are always lowercase, dot-separated keys like `"package.database"`.
/// The i18n key is derived as `"tag.<key>"`.
///
/// Tags from the built-in libraries are created with the library functions,
/// e.g. `PackageTags::database()`.  Custom tags (from plugins or user input)
/// use `FsTag::new("custom.key")` but will be marked as non-standard in the Store.
///
/// # Serialization
///
/// Serializes transparently as a plain string so TOML manifests remain readable:
/// ```toml
/// tags = ["package.database", "package.security", "platform.linux"]
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FsTag(String);

impl FsTag {
    /// Create a tag from any string key.
    ///
    /// Does not validate against a library — use `FsTag::is_known()` for that.
    pub fn new(key: impl Into<String>) -> Self {
        Self(key.into())
    }

    /// The raw tag key, e.g. `"package.database"`.
    pub fn key(&self) -> &str {
        &self.0
    }

    /// The i18n key for the translated display name of this tag.
    ///
    /// Convention: `"tag.<key>"`, e.g. `"tag.package.database"`.
    pub fn i18n_key(&self) -> String {
        format!("tag.{}", self.0)
    }

    /// Returns `true` when this tag is registered in any known library.
    ///
    /// Checks `PackageTags`, `PlatformTags`, and `ApiTags`.
    pub fn is_known(&self) -> bool {
        PackageTags::contains(self.key())
            || PlatformTags::contains(self.key())
            || ApiTags::contains(self.key())
    }

    /// Returns `true` when the key follows the naming convention:
    /// lowercase, letters and digits, separated by dots or hyphens.
    pub fn is_valid_key(&self) -> bool {
        !self.0.is_empty()
            && self
                .0
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '.' || c == '-')
    }
}

impl std::fmt::Display for FsTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<&str> for FsTag {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

impl AsRef<str> for FsTag {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

// ── TagLibrary ────────────────────────────────────────────────────────────────

/// A named set of well-known tag keys.
///
/// Implement this trait for each tag domain (packages, platform, APIs, …).
/// The `all_keys()` slice is used by the Store for auto-complete, validation,
/// and ensuring every key has a corresponding i18n entry.
///
/// # Example
///
/// ```rust
/// use fs_types::tags::{FsTag, TagLibrary, PackageTags};
///
/// assert!(PackageTags::contains("package.database"));
/// assert!(!PackageTags::contains("unknown.thing"));
/// ```
pub trait TagLibrary {
    /// Every tag key in this library.
    fn all_keys() -> &'static [&'static str]
    where
        Self: Sized;

    /// Returns `true` when `key` is a member of this library.
    fn contains(key: &str) -> bool
    where
        Self: Sized,
    {
        Self::all_keys().contains(&key)
    }

    /// All tags in this library as `FsTag` instances.
    fn all() -> Vec<FsTag>
    where
        Self: Sized,
    {
        Self::all_keys().iter().map(|k| FsTag::new(*k)).collect()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tag_key_convention() {
        // All library tags must follow the naming convention.
        for key in PackageTags::all_keys()
            .iter()
            .chain(PlatformTags::all_keys())
            .chain(ApiTags::all_keys())
        {
            let tag = FsTag::new(*key);
            assert!(
                tag.is_valid_key(),
                "Tag key does not follow naming convention: {key}"
            );
        }
    }

    #[test]
    fn tag_i18n_keys_have_prefix() {
        for key in PackageTags::all_keys() {
            let tag = FsTag::new(*key);
            assert!(
                tag.i18n_key().starts_with("tag."),
                "i18n key missing 'tag.' prefix: {}",
                tag.i18n_key()
            );
        }
    }

    #[test]
    fn no_duplicate_keys_across_libraries() {
        let mut all: Vec<&str> = Vec::new();
        all.extend_from_slice(PackageTags::all_keys());
        all.extend_from_slice(PlatformTags::all_keys());
        all.extend_from_slice(ApiTags::all_keys());

        let mut seen = std::collections::HashSet::new();
        for key in &all {
            assert!(
                seen.insert(*key),
                "Duplicate tag key across libraries: {key}"
            );
        }
    }

    #[test]
    fn tag_is_known() {
        assert!(FsTag::new("package.database").is_known());
        assert!(FsTag::new("platform.linux").is_known());
        assert!(FsTag::new("api.rest").is_known());
        assert!(!FsTag::new("unknown.thing").is_known());
    }

    #[test]
    fn tag_serde_transparent() {
        let t = FsTag::new("package.database");
        let json = serde_json::to_string(&t).unwrap();
        assert_eq!(json, "\"package.database\"");
        let back: FsTag = serde_json::from_str(&json).unwrap();
        assert_eq!(t, back);
    }

    #[test]
    fn tag_is_valid_key() {
        assert!(FsTag::new("package.database").is_valid_key());
        assert!(FsTag::new("platform.linux").is_valid_key());
        assert!(FsTag::new("api.rest").is_valid_key());
        assert!(!FsTag::new("").is_valid_key());
        assert!(!FsTag::new("Has Spaces").is_valid_key());
        assert!(!FsTag::new("UPPERCASE").is_valid_key());
    }
}
