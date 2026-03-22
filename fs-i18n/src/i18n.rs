//! Main [`I18n`] struct — loads Fluent `.ftl` files and TOML strings, provides translation lookup.
//!
//! Typed return values:
//!   [`Translation`]  — resolved translated text; implements `Display` + `Deref<str>`
//!   [`LanguageCode`] — a BCP-47 language code; implements `Display` + `Deref<str>`

use std::collections::HashMap;
use std::path::Path;

use fs_error::FsError;

use crate::bundle::LocaleBundle;

// ── Translation ───────────────────────────────────────────────────────────────

/// A resolved, translated string.
///
/// Returned by [`I18n::t`] and [`I18n::t_with`] instead of a plain `String`
/// so callers always work with a typed value.  Access the text via `Display`,
/// `Deref`, or `as_str()`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Translation(String);

impl Translation {
    pub(crate) fn new(s: String) -> Self { Self(s) }

    /// Borrows the translated text as a `&str`.
    pub fn as_str(&self) -> &str { &self.0 }
}

impl std::fmt::Display for Translation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::ops::Deref for Translation {
    type Target = str;
    fn deref(&self) -> &str { &self.0 }
}

impl AsRef<str> for Translation {
    fn as_ref(&self) -> &str { &self.0 }
}

impl PartialEq<str> for Translation {
    fn eq(&self, other: &str) -> bool { self.0 == other }
}

impl PartialEq<&str> for Translation {
    fn eq(&self, other: &&str) -> bool { self.0 == *other }
}

impl PartialEq<String> for Translation {
    fn eq(&self, other: &String) -> bool { self.0 == *other }
}

impl From<Translation> for String {
    fn from(t: Translation) -> String { t.0 }
}

// ── LanguageCode ──────────────────────────────────────────────────────────────

/// A BCP-47 language code (e.g. `"de"`, `"en"`, `"ar"`).
///
/// Returned by [`I18n::lang`] instead of a plain `&str` so callers always
/// hold a typed value.  Access the code via `Display`, `Deref`, or `as_str()`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguageCode(String);

impl LanguageCode {
    pub(crate) fn new(s: impl Into<String>) -> Self { Self(s.into()) }

    /// Borrows the language code as a `&str`.
    pub fn as_str(&self) -> &str { &self.0 }
}

impl std::fmt::Display for LanguageCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::ops::Deref for LanguageCode {
    type Target = str;
    fn deref(&self) -> &str { &self.0 }
}

impl AsRef<str> for LanguageCode {
    fn as_ref(&self) -> &str { &self.0 }
}

// ── I18n ──────────────────────────────────────────────────────────────────────

/// Fluent- and TOML-based i18n system.
///
/// Loads `.ftl` files from `locales/{lang}/` directories **and/or** TOML
/// strings for compile-time-bundled CLI strings.  Provides translation lookup
/// via [`t`](Self::t) and [`t_with`](Self::t_with).
///
/// Fallback chain:
/// active Fluent → active TOML map → fallback Fluent → fallback TOML map → raw key.
pub struct I18n {
    bundles: HashMap<String, LocaleBundle>,
    /// Flat `"section.key" → value` maps loaded from TOML strings.
    toml_maps: HashMap<String, HashMap<String, String>>,
    active_lang: String,
    fallback_lang: String,
}

impl I18n {
    /// Create an empty instance with explicit active and fallback languages.
    pub fn new(active_lang: &str, fallback_lang: &str) -> Self {
        Self {
            bundles: HashMap::new(),
            toml_maps: HashMap::new(),
            active_lang: active_lang.to_string(),
            fallback_lang: fallback_lang.to_string(),
        }
    }

    /// Load all `.ftl` files from `{dir}/{lang}/` for every language
    /// subdirectory found under `dir`.
    ///
    /// Directory structure: `{dir}/de/actions.ftl`, `{dir}/en/actions.ftl`, …
    ///
    /// Active language defaults to `"en"`; fallback language is always `"en"`.
    pub fn load_dir(dir: &Path) -> Result<Self, FsError> {
        Self::load_dir_with_lang(dir, "en", "en")
    }

    /// Load from a directory with an explicit active and fallback language.
    pub fn load_dir_with_lang(
        dir: &Path,
        active: &str,
        fallback: &str,
    ) -> Result<Self, FsError> {
        let mut instance = Self::new(active, fallback);

        let entries = std::fs::read_dir(dir).map_err(|e| {
            FsError::config(format!(
                "cannot open i18n directory `{}`: {e}",
                dir.display()
            ))
        })?;

        for entry in entries.flatten() {
            let lang_path = entry.path();
            if !lang_path.is_dir() {
                continue;
            }
            let lang = match lang_path.file_name().and_then(|n| n.to_str()) {
                Some(s) if !s.is_empty() => s.to_string(),
                _ => continue,
            };

            let sources = collect_ftl_sources(&lang_path)?;
            instance.add_ftl(&lang, &sources)?;
        }

        Ok(instance)
    }

    /// Add FTL source strings for a language programmatically.
    ///
    /// Replaces any previously loaded Fluent bundle for that language.
    pub fn add_ftl(&mut self, lang: &str, ftl_sources: &[String]) -> Result<(), FsError> {
        let bundle = LocaleBundle::new(lang, ftl_sources)?;
        self.bundles.insert(lang.to_string(), bundle);
        Ok(())
    }

    /// Parse a TOML string and add the flattened key/value pairs for `lang`.
    ///
    /// Hierarchical keys are flattened with dot notation:
    /// `[section] key = "val"` becomes `"section.key" → "val"`.
    ///
    /// Existing entries for `lang` are merged; conflicting keys are overwritten.
    pub fn add_toml_str(&mut self, lang: &str, toml_src: &str) -> Result<(), FsError> {
        let value: toml::Value = toml_src
            .parse()
            .map_err(|e| FsError::config(format!("invalid TOML for lang `{lang}`: {e}")))?;

        let map = self.toml_maps.entry(lang.to_string()).or_default();
        flatten_toml("", &value, map);
        Ok(())
    }

    /// Insert a pre-flattened `"section.key" → value` map for `lang`.
    ///
    /// Existing entries for `lang` are merged; conflicting keys are overwritten.
    pub fn add_toml_map(&mut self, lang: &str, map: HashMap<String, String>) {
        self.toml_maps
            .entry(lang.to_string())
            .or_default()
            .extend(map);
    }

    /// Set the active language.
    pub fn set_lang(&mut self, lang: &str) {
        self.active_lang = lang.to_string();
    }

    /// Return the active language code.
    pub fn lang(&self) -> LanguageCode {
        LanguageCode::new(&self.active_lang)
    }

    /// Translate a key using the active language.
    ///
    /// Fallback chain:
    /// active Fluent → active TOML → fallback Fluent → fallback TOML → raw key.
    pub fn t(&self, key: &str) -> Translation {
        // active Fluent
        if let Some(v) = self.bundles.get(&self.active_lang).and_then(|b| b.get(key)) {
            return Translation::new(v);
        }
        // active TOML
        if let Some(v) = self.toml_maps.get(&self.active_lang).and_then(|m| m.get(key)) {
            return Translation::new(v.clone());
        }
        if self.active_lang != self.fallback_lang {
            // fallback Fluent
            if let Some(v) = self.bundles.get(&self.fallback_lang).and_then(|b| b.get(key)) {
                return Translation::new(v);
            }
            // fallback TOML
            if let Some(v) = self.toml_maps.get(&self.fallback_lang).and_then(|m| m.get(key)) {
                return Translation::new(v.clone());
            }
        }
        Translation::new(key.to_string())
    }

    /// Translate a key with named string arguments.
    ///
    /// For Fluent strings the Fluent variable syntax is used (`{ $name }`).
    /// For TOML strings simple `{name}` placeholder substitution is applied.
    ///
    /// Fallback chain mirrors [`t`](Self::t):
    /// active Fluent → active TOML → fallback Fluent → fallback TOML → raw key.
    ///
    /// # Example
    /// ```rust,ignore
    /// i18n.t_with("phrase-confirm-delete", &[("item", "module")])
    /// ```
    pub fn t_with(&self, key: &str, args: &[(&str, &str)]) -> Translation {
        // active Fluent
        if let Some(v) = self
            .bundles
            .get(&self.active_lang)
            .and_then(|b| b.get_with_args(key, args))
        {
            return Translation::new(v);
        }
        // active TOML
        if let Some(template) = self.toml_maps.get(&self.active_lang).and_then(|m| m.get(key)) {
            return Translation::new(apply_args(template, args));
        }
        if self.active_lang != self.fallback_lang {
            // fallback Fluent
            if let Some(v) = self
                .bundles
                .get(&self.fallback_lang)
                .and_then(|b| b.get_with_args(key, args))
            {
                return Translation::new(v);
            }
            // fallback TOML
            if let Some(template) =
                self.toml_maps.get(&self.fallback_lang).and_then(|m| m.get(key))
            {
                return Translation::new(apply_args(template, args));
            }
        }
        Translation::new(key.to_string())
    }

    /// Return `true` if the active language bundle contains `key`.
    pub fn has(&self, key: &str) -> bool {
        self.bundles
            .get(&self.active_lang)
            .and_then(|b| b.get(key))
            .is_some()
    }

    /// Return all translation keys available for `lang` (both Fluent and TOML).
    ///
    /// Keys from both the Fluent bundle and the TOML map are collected and
    /// de-duplicated. The order is unspecified.
    pub fn keys_for_lang(&self, lang: &str) -> Vec<String> {
        let mut keys: std::collections::HashSet<String> = std::collections::HashSet::new();

        if let Some(toml_map) = self.toml_maps.get(lang) {
            keys.extend(toml_map.keys().cloned());
        }

        // Fluent keys: iterate the bundle's message IDs via known keys.
        // LocaleBundle does not expose a key iterator, so we check every key
        // collected from other languages against this bundle.
        // Instead we use a separate approach: expose keys via a helper on I18n
        // itself by trying every key we know about from all TOML maps.
        // This covers the practical case where Fluent and TOML share the same key space.
        // For completeness also check Fluent-only keys by probing the bundle with
        // keys from all other languages' TOML maps.
        if let Some(bundle) = self.bundles.get(lang) {
            // Collect all known keys across all TOML maps as candidate Fluent keys.
            let candidates: Vec<String> = self
                .toml_maps
                .values()
                .flat_map(|m| m.keys().cloned())
                .collect();
            for key in candidates {
                if !keys.contains(&key) && bundle.get(&key).is_some() {
                    keys.insert(key);
                }
            }
        }

        keys.into_iter().collect()
    }
}

// ── helpers ───────────────────────────────────────────────────────────────────

/// Read every `.ftl` file in `dir` and return their contents as a `Vec<String>`.
fn collect_ftl_sources(dir: &Path) -> Result<Vec<String>, FsError> {
    let mut sources = Vec::new();

    let entries = std::fs::read_dir(dir).map_err(|e| {
        FsError::config(format!("cannot read directory `{}`: {e}", dir.display()))
    })?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("ftl") {
            continue;
        }
        let src = std::fs::read_to_string(&path).map_err(|e| {
            FsError::config(format!("cannot read `{}`: {e}", path.display()))
        })?;
        sources.push(src);
    }

    Ok(sources)
}

/// Flatten a [`toml::Value`] tree into a `key → value` map using dot notation.
///
/// - `String` values are inserted as-is.
/// - Non-string scalars are converted with `to_string()`.
/// - Tables are recursed with `{prefix}.{key}` (leading dot stripped at root).
/// - Arrays are skipped.
fn flatten_toml(prefix: &str, value: &toml::Value, out: &mut HashMap<String, String>) {
    match value {
        toml::Value::Table(table) => {
            for (k, v) in table {
                let new_prefix = if prefix.is_empty() {
                    k.clone()
                } else {
                    format!("{prefix}.{k}")
                };
                flatten_toml(&new_prefix, v, out);
            }
        }
        toml::Value::String(s) => {
            if !prefix.is_empty() {
                out.insert(prefix.to_string(), s.clone());
            }
        }
        other => {
            if !prefix.is_empty() {
                out.insert(prefix.to_string(), other.to_string());
            }
        }
    }
}

/// Replace `{name}` placeholders in `template` with the corresponding value
/// from `args`.
///
/// Placeholders that have no matching argument are left unchanged.
fn apply_args(template: &str, args: &[(&str, &str)]) -> String {
    let mut result = template.to_string();
    for (name, value) in args {
        result = result.replace(&format!("{{{name}}}"), value);
    }
    result
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── TOML: add_toml_str ─────────────────────────────────────────────────

    #[test]
    fn toml_simple_lookup() {
        let mut i = I18n::new("en", "en");
        i.add_toml_str("en", "action-save = \"Save\"\n").unwrap();
        assert_eq!(i.t("action-save"), "Save");
    }

    #[test]
    fn toml_nested_keys_are_flattened() {
        let src = "[actions]\nsave = \"Save\"\ncancel = \"Cancel\"\n";
        let mut i = I18n::new("en", "en");
        i.add_toml_str("en", src).unwrap();
        assert_eq!(i.t("actions.save"), "Save");
        assert_eq!(i.t("actions.cancel"), "Cancel");
    }

    #[test]
    fn toml_fallback_to_en() {
        let mut i = I18n::new("de", "en");
        i.add_toml_str("en", "action-save = \"Save\"\n").unwrap();
        // "de" has no TOML map — should fall back to "en"
        assert_eq!(i.t("action-save"), "Save");
    }

    #[test]
    fn toml_active_lang_wins_over_fallback() {
        let mut i = I18n::new("de", "en");
        i.add_toml_str("en", "action-save = \"Save\"\n").unwrap();
        i.add_toml_str("de", "action-save = \"Speichern\"\n").unwrap();
        assert_eq!(i.t("action-save"), "Speichern");
    }

    #[test]
    fn toml_fallback_to_key_when_missing() {
        let i = I18n::new("en", "en");
        assert_eq!(i.t("missing-key"), "missing-key");
    }

    #[test]
    fn toml_variable_substitution() {
        let mut i = I18n::new("en", "en");
        i.add_toml_str("en", "phrase-delete = \"Delete {item}?\"\n")
            .unwrap();
        let result = i.t_with("phrase-delete", &[("item", "module")]);
        assert_eq!(result, "Delete module?");
    }

    #[test]
    fn toml_multiple_placeholders() {
        let mut i = I18n::new("en", "en");
        i.add_toml_str("en", "msg = \"Hello {name}, you have {count} messages.\"\n")
            .unwrap();
        let result = i.t_with("msg", &[("name", "Alice"), ("count", "3")]);
        assert_eq!(result, "Hello Alice, you have 3 messages.");
    }

    #[test]
    fn toml_add_map_direct_insert() {
        let mut map = HashMap::new();
        map.insert("label.ok".to_string(), "OK".to_string());
        let mut i = I18n::new("en", "en");
        i.add_toml_map("en", map);
        assert_eq!(i.t("label.ok"), "OK");
    }

    #[test]
    fn toml_invalid_toml_returns_error() {
        let mut i = I18n::new("en", "en");
        let result = i.add_toml_str("en", "not valid toml ===");
        assert!(result.is_err());
    }

    // ── Fluent still works after TOML additions ────────────────────────────

    #[test]
    fn fluent_unaffected_by_toml_additions() {
        let mut i = I18n::new("en", "en");
        i.add_ftl("en", &["action-save = Save\n".to_string()])
            .unwrap();
        i.add_toml_str("en", "other-key = \"Other\"\n").unwrap();
        assert_eq!(i.t("action-save"), "Save");
        assert_eq!(i.t("other-key"), "Other");
    }

    #[test]
    fn fluent_wins_over_toml_for_same_key() {
        let mut i = I18n::new("en", "en");
        // Both Fluent and TOML define the same key — Fluent wins.
        i.add_ftl("en", &["action-save = FluentSave\n".to_string()])
            .unwrap();
        i.add_toml_str("en", "action-save = \"TomlSave\"\n").unwrap();
        assert_eq!(i.t("action-save"), "FluentSave");
    }
}
