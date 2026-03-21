//! Developer tools for the i18n system.
//!
//! - [`find_missing`] — find keys present in one language but not another.
//! - [`SnippetPlugin`] — trait for extending the i18n system with additional snippets.

use crate::i18n::I18n;
use fsn_error::FsnError;

// ── SnippetPlugin ─────────────────────────────────────────────────────────────

/// Plugin trait for registering additional snippet collections.
///
/// Implement this trait to provide a named set of snippets that can be
/// loaded into any [`I18n`] instance.  Useful for crates that ship their
/// own translatable strings.
pub trait SnippetPlugin {
    /// Plugin identifier (e.g. `"fsn-health"`, `"my-app"`).
    fn name(&self) -> &str;

    /// All snippets provided by this plugin as `(lang_code, toml_source)` pairs.
    fn snippets(&self) -> &[(&str, &str)];

    /// Load all snippets from this plugin into `i18n`.
    fn load_into(&self, i18n: &mut I18n) -> Result<(), FsnError> {
        for (lang, toml_src) in self.snippets() {
            i18n.add_toml_str(lang, toml_src)?;
        }
        Ok(())
    }
}

// ── find_missing ──────────────────────────────────────────────────────────────

/// Find translation keys present in `reference_lang` but missing in `target_lang`.
///
/// Compares the key sets of two languages in the same [`I18n`] instance and
/// returns every key that has a translation in `reference_lang` but not in
/// `target_lang`.
///
/// Useful during development to detect incomplete translations.
///
/// # Example
///
/// ```rust
/// use fsn_i18n::{I18n, tools::find_missing};
///
/// let mut i18n = I18n::new("en", "en");
/// i18n.add_toml_str("en", "[actions]\nsave = \"Save\"\ncancel = \"Cancel\"\n").unwrap();
/// i18n.add_toml_str("de", "[actions]\nsave = \"Speichern\"\n").unwrap();
///
/// let missing = find_missing("en", "de", &i18n);
/// assert_eq!(missing, vec!["actions.cancel"]);
/// ```
pub fn find_missing(reference_lang: &str, target_lang: &str, i18n: &I18n) -> Vec<String> {
    let reference_keys = i18n.keys_for_lang(reference_lang);
    let target_keys: std::collections::HashSet<String> =
        i18n.keys_for_lang(target_lang).into_iter().collect();
    let mut missing: Vec<String> = reference_keys
        .into_iter()
        .filter(|k| !target_keys.contains(k))
        .collect();
    missing.sort();
    missing
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_missing_one_key() {
        let mut i18n = I18n::new("en", "en");
        i18n.add_toml_str("en", "[actions]\nsave = \"Save\"\ncancel = \"Cancel\"\n")
            .unwrap();
        i18n.add_toml_str("de", "[actions]\nsave = \"Speichern\"\n")
            .unwrap();

        let missing = find_missing("en", "de", &i18n);
        assert_eq!(missing, vec!["actions.cancel"]);
    }

    #[test]
    fn find_missing_all_present_returns_empty() {
        let mut i18n = I18n::new("en", "en");
        i18n.add_toml_str("en", "save = \"Save\"\n").unwrap();
        i18n.add_toml_str("de", "save = \"Speichern\"\n").unwrap();

        let missing = find_missing("en", "de", &i18n);
        assert!(missing.is_empty());
    }

    #[test]
    fn find_missing_target_has_no_translations() {
        let mut i18n = I18n::new("en", "en");
        i18n.add_toml_str("en", "key1 = \"A\"\nkey2 = \"B\"\n")
            .unwrap();
        // "fr" has no data at all

        let missing = find_missing("en", "fr", &i18n);
        assert_eq!(missing, vec!["key1", "key2"]);
    }

    #[test]
    fn find_missing_result_is_sorted() {
        let mut i18n = I18n::new("en", "en");
        i18n.add_toml_str("en", "zebra = \"Z\"\napple = \"A\"\nmango = \"M\"\n")
            .unwrap();
        // "fr" has none

        let missing = find_missing("en", "fr", &i18n);
        assert_eq!(missing, vec!["apple", "mango", "zebra"]);
    }

    #[test]
    fn find_missing_empty_reference_returns_empty() {
        let mut i18n = I18n::new("en", "en");
        i18n.add_toml_str("de", "key = \"Wert\"\n").unwrap();

        let missing = find_missing("en", "de", &i18n);
        assert!(missing.is_empty());
    }

    struct TestPlugin;

    impl SnippetPlugin for TestPlugin {
        fn name(&self) -> &str {
            "test-plugin"
        }

        fn snippets(&self) -> &[(&str, &str)] {
            &[
                ("en", "plugin.label = \"Hello\"\n"),
                ("de", "plugin.label = \"Hallo\"\n"),
            ]
        }
    }

    #[test]
    fn snippet_plugin_load_into() {
        let mut i18n = I18n::new("en", "en");
        TestPlugin.load_into(&mut i18n).unwrap();

        assert_eq!(i18n.t("plugin.label"), "Hello");
    }

    #[test]
    fn snippet_plugin_name() {
        assert_eq!(TestPlugin.name(), "test-plugin");
    }
}
