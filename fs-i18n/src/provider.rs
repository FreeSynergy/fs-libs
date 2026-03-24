//! `LanguageProvider` — trait for dynamic language registration.
//!
//! The built-in language list is loaded from `languages.toml` at startup.
//! This trait allows external sources — plugins, user-installed language packs,
//! or API responses — to contribute additional `LanguageMeta` entries without
//! modifying the built-in table.
//!
//! # Design
//!
//! ```text
//! LanguageProvider trait
//!   ├── BuiltinLanguageProvider   ← reads languages.toml (always registered)
//!   └── (plugin-provided impl)   ← adds new languages at runtime
//!
//! LanguageRegistry               ← collects all providers, deduplicates by code
//! ```
//!
//! # Usage
//!
//! ```rust,ignore
//! use fs_i18n::provider::{LanguageRegistry, LanguageProvider};
//!
//! let mut registry = LanguageRegistry::new();
//! registry.register(Box::new(MyPlugin));
//!
//! let all = registry.all_languages();
//! let de  = registry.find("de");
//! ```

use crate::languages::{all_languages, LanguageMeta};

// ── LanguageProvider ──────────────────────────────────────────────────────────

/// A source of language metadata.
///
/// Implement this trait to register additional languages at runtime —
/// for example from a plugin or a remote language catalog.
///
/// # Requirements
///
/// - `name()` must be unique across all registered providers.
/// - `languages()` must not return duplicates within one provider.
/// - If two providers return the same `code`, the first registered wins.
pub trait LanguageProvider: Send + Sync {
    /// Unique provider name, e.g. `"builtin"`, `"community-pack"`.
    fn name(&self) -> &str;

    /// All language entries this provider contributes.
    fn languages(&self) -> Vec<LanguageMeta>;
}

// ── BuiltinLanguageProvider ───────────────────────────────────────────────────

/// The default provider — wraps the static `languages.toml` table.
///
/// Always registered first in a [`LanguageRegistry`].
pub struct BuiltinLanguageProvider;

impl LanguageProvider for BuiltinLanguageProvider {
    fn name(&self) -> &str {
        "builtin"
    }

    fn languages(&self) -> Vec<LanguageMeta> {
        all_languages().to_vec()
    }
}

// ── LanguageRegistry ──────────────────────────────────────────────────────────

/// Aggregates all registered [`LanguageProvider`]s.
///
/// Languages are deduplicated by code — the first registered provider wins.
/// The builtin provider is always registered on construction.
pub struct LanguageRegistry {
    providers: Vec<Box<dyn LanguageProvider>>,
}

impl LanguageRegistry {
    /// Create a new registry pre-populated with the built-in languages.
    pub fn new() -> Self {
        let mut r = Self {
            providers: Vec::new(),
        };
        r.register(Box::new(BuiltinLanguageProvider));
        r
    }

    /// Register an additional language provider.
    ///
    /// The provider's languages are available immediately via [`all_languages`](Self::all_languages).
    /// If a provider with the same name is already registered, this is a no-op.
    pub fn register(&mut self, provider: Box<dyn LanguageProvider>) {
        let name = provider.name().to_owned();
        if !self.providers.iter().any(|p| p.name() == name) {
            self.providers.push(provider);
        }
    }

    /// All languages from all registered providers, deduplicated by code.
    ///
    /// The first provider that contributes a given code wins.
    pub fn all_languages(&self) -> Vec<LanguageMeta> {
        let mut seen = std::collections::HashSet::new();
        let mut result = Vec::new();

        for provider in &self.providers {
            for lang in provider.languages() {
                if seen.insert(lang.code.to_owned()) {
                    result.push(lang);
                }
            }
        }

        result
    }

    /// Find a language by its code (e.g. `"de"`, `"ar"`).
    ///
    /// Searches all registered providers in registration order.
    /// Returns `None` when no provider knows the code.
    pub fn find(&self, code: &str) -> Option<LanguageMeta> {
        for provider in &self.providers {
            for lang in provider.languages() {
                if lang.code == code {
                    return Some(lang);
                }
            }
        }
        None
    }

    /// Returns the number of registered providers.
    pub fn provider_count(&self) -> usize {
        self.providers.len()
    }
}

impl Default for LanguageRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::languages::TextDirection;

    #[test]
    fn default_registry_has_builtin_languages() {
        let r = LanguageRegistry::new();
        let langs = r.all_languages();
        assert!(!langs.is_empty(), "registry must have builtin languages");
        assert_eq!(r.provider_count(), 1);
    }

    #[test]
    fn find_known_language() {
        let r = LanguageRegistry::new();
        let de = r.find("de").expect("German must be in builtin table");
        assert_eq!(de.name, "German");
        assert_eq!(de.native_name, "Deutsch");
        assert!(!de.is_rtl());
    }

    #[test]
    fn find_rtl_language() {
        let r = LanguageRegistry::new();
        let ar = r.find("ar").expect("Arabic must be in builtin table");
        assert!(ar.is_rtl());
    }

    #[test]
    fn find_unknown_returns_none() {
        let r = LanguageRegistry::new();
        assert!(r.find("xx").is_none());
    }

    #[test]
    fn custom_provider_adds_language() {
        struct TestProvider;
        impl LanguageProvider for TestProvider {
            fn name(&self) -> &str {
                "test"
            }
            fn languages(&self) -> Vec<LanguageMeta> {
                vec![LanguageMeta {
                    code: "xx",
                    name: "Test Language",
                    native_name: "Testsprache",
                    script: "Latin",
                    direction: TextDirection::Ltr,
                    family: "Invented",
                    continent: "Europe",
                }]
            }
        }

        let mut r = LanguageRegistry::new();
        r.register(Box::new(TestProvider));
        assert_eq!(r.provider_count(), 2);

        let xx = r.find("xx").expect("custom language must be found");
        assert_eq!(xx.name, "Test Language");
    }

    #[test]
    fn builtin_wins_over_duplicate_code() {
        struct OverrideProvider;
        impl LanguageProvider for OverrideProvider {
            fn name(&self) -> &str {
                "override"
            }
            fn languages(&self) -> Vec<LanguageMeta> {
                vec![LanguageMeta {
                    code: "de",
                    name: "FAKE German",
                    native_name: "FAKE",
                    script: "Latin",
                    direction: TextDirection::Ltr,
                    family: "Indo-European",
                    continent: "Europe",
                }]
            }
        }

        let mut r = LanguageRegistry::new();
        r.register(Box::new(OverrideProvider));

        // Builtin "de" was registered first — it wins.
        let de = r.find("de").unwrap();
        assert_eq!(de.name, "German");
    }

    #[test]
    fn duplicate_provider_name_is_ignored() {
        struct DupeProvider;
        impl LanguageProvider for DupeProvider {
            fn name(&self) -> &str {
                "builtin"
            } // same name as BuiltinLanguageProvider
            fn languages(&self) -> Vec<LanguageMeta> {
                vec![]
            }
        }

        let mut r = LanguageRegistry::new();
        r.register(Box::new(DupeProvider));
        assert_eq!(r.provider_count(), 1); // not added
    }
}
