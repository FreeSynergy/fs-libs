// languages.rs — Language metadata embedded from languages.toml.
//
// Provides typed access to RTL/LTR direction, script name, display names,
// language family, and continent for all 50 languages supported by fs-i18n.

use std::sync::OnceLock;

use serde::Deserialize;

// ── Types ─────────────────────────────────────────────────────────────────────

/// Text direction for a language.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextDirection {
    /// Left-to-right (most scripts).
    Ltr,
    /// Right-to-left (Arabic, Persian, Urdu, Pashto).
    Rtl,
}

impl TextDirection {
    /// Returns `true` for right-to-left languages.
    pub fn is_rtl(self) -> bool {
        matches!(self, TextDirection::Rtl)
    }
}

impl std::fmt::Display for TextDirection {
    /// Renders `"ltr"` or `"rtl"`.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TextDirection::Ltr => f.write_str("ltr"),
            TextDirection::Rtl => f.write_str("rtl"),
        }
    }
}

/// Metadata for a single supported language.
#[derive(Debug, Clone)]
pub struct LanguageMeta {
    /// ISO 639-1 language code (e.g. `"de"`, `"ar"`, `"yue"`).
    pub code: &'static str,
    /// English display name (e.g. `"German"`, `"Arabic"`).
    pub name: &'static str,
    /// Name in the language itself (e.g. `"Deutsch"`, `"العربية"`).
    pub native_name: &'static str,
    /// Writing system (e.g. `"Latin"`, `"Cyrillic"`, `"Arabic"`, `"Devanagari"`).
    pub script: &'static str,
    /// Text direction.
    pub direction: TextDirection,
    /// Linguistic family (e.g. `"Indo-European"`, `"Afro-Asiatic"`, `"Sino-Tibetan"`).
    pub family: &'static str,
    /// Primary continent of origin / main speaker region (e.g. `"Europe"`, `"Asia"`, `"Africa"`).
    pub continent: &'static str,
}

impl LanguageMeta {
    /// Returns `true` if this language is written right-to-left.
    pub fn is_rtl(&self) -> bool {
        self.direction.is_rtl()
    }
}

// ── Raw TOML deserialization helper ───────────────────────────────────────────

#[derive(Deserialize)]
struct RawMeta {
    name: String,
    native_name: String,
    script: String,
    direction: String,
    family: String,
    continent: String,
}

// ── Static table ──────────────────────────────────────────────────────────────

static LANGUAGES: OnceLock<Vec<LanguageMeta>> = OnceLock::new();

const LANGUAGES_TOML: &str = include_str!("../languages.toml");

fn build_table() -> Vec<LanguageMeta> {
    let raw: std::collections::BTreeMap<String, RawMeta> =
        toml::from_str(LANGUAGES_TOML).expect("languages.toml is malformed");

    // Leak the strings so we can hand out `&'static str` references.
    raw.into_iter()
        .map(|(code, m)| {
            let code: &'static str = Box::leak(code.into_boxed_str());
            let name: &'static str = Box::leak(m.name.into_boxed_str());
            let native_name: &'static str = Box::leak(m.native_name.into_boxed_str());
            let script: &'static str = Box::leak(m.script.into_boxed_str());
            let family: &'static str = Box::leak(m.family.into_boxed_str());
            let continent: &'static str = Box::leak(m.continent.into_boxed_str());
            let direction = if m.direction == "rtl" {
                TextDirection::Rtl
            } else {
                TextDirection::Ltr
            };
            LanguageMeta {
                code,
                name,
                native_name,
                script,
                direction,
                family,
                continent,
            }
        })
        .collect()
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Returns all supported language metadata, sorted by language code.
pub fn all_languages() -> &'static [LanguageMeta] {
    LANGUAGES.get_or_init(build_table)
}

/// Look up metadata for a single language by its code (e.g. `"de"`, `"ar"`).
///
/// Returns `None` if the code is not in the built-in table.
pub fn language_meta(code: &str) -> Option<&'static LanguageMeta> {
    all_languages().iter().find(|m| m.code == code)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn de_is_ltr_latin() {
        let m = language_meta("de").unwrap();
        assert_eq!(m.name, "German");
        assert_eq!(m.native_name, "Deutsch");
        assert_eq!(m.script, "Latin");
        assert!(!m.is_rtl());
    }

    #[test]
    fn ar_is_rtl() {
        let m = language_meta("ar").unwrap();
        assert_eq!(m.name, "Arabic");
        assert!(m.is_rtl());
        assert_eq!(m.direction.to_string(), "rtl");
    }

    #[test]
    fn fa_is_rtl() {
        assert!(language_meta("fa").unwrap().is_rtl());
    }

    #[test]
    fn ur_is_rtl() {
        assert!(language_meta("ur").unwrap().is_rtl());
    }

    #[test]
    fn ps_is_rtl() {
        assert!(language_meta("ps").unwrap().is_rtl());
    }

    #[test]
    fn yue_cantonese() {
        let m = language_meta("yue").unwrap();
        assert_eq!(m.name, "Cantonese");
        assert_eq!(m.script, "CJK");
        assert!(!m.is_rtl());
    }

    #[test]
    fn all_50_languages_present() {
        assert_eq!(all_languages().len(), 50);
    }

    #[test]
    fn exactly_4_rtl_languages() {
        let rtl: Vec<_> = all_languages().iter().filter(|m| m.is_rtl()).collect();
        assert_eq!(rtl.len(), 4);
        let codes: Vec<_> = rtl.iter().map(|m| m.code).collect();
        assert!(codes.contains(&"ar"));
        assert!(codes.contains(&"fa"));
        assert!(codes.contains(&"ur"));
        assert!(codes.contains(&"ps"));
    }

    #[test]
    fn unknown_code_returns_none() {
        assert!(language_meta("xx").is_none());
    }

    #[test]
    fn de_family_and_continent() {
        let m = language_meta("de").unwrap();
        assert_eq!(m.family, "Indo-European");
        assert_eq!(m.continent, "Europe");
    }

    #[test]
    fn ar_family_and_continent() {
        let m = language_meta("ar").unwrap();
        assert_eq!(m.family, "Afro-Asiatic");
        assert_eq!(m.continent, "Asia");
    }

    #[test]
    fn yue_family_and_continent() {
        let m = language_meta("yue").unwrap();
        assert_eq!(m.family, "Sino-Tibetan");
        assert_eq!(m.continent, "Asia");
    }
}
