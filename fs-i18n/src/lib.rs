//! Fluent-based i18n for FreeSynergy.
//!
//! Loads `.ftl` translation files from `locales/{lang}/` directories.
//!
//! # Quick start
//! ```rust,ignore
//! use fs_i18n::I18n;
//!
//! let i18n = I18n::load_dir(Path::new("locales")).unwrap();
//! let save_label = i18n.t("action-save");     // "Save" or "Speichern"
//! let msg = i18n.t_with("phrase-confirm-delete", &[("item", "module")]);
//! ```
//!
//! # Global instance
//! ```rust,ignore
//! use fs_i18n;
//!
//! fs_i18n::init(Path::new("locales")).unwrap();
//! let label = fs_i18n::t("action-save");
//! ```

mod bundle;
mod i18n;
pub mod languages;
pub mod locale;
pub mod macros;
pub mod snippets;
pub mod tools;

pub use i18n::{I18n, LanguageCode, Translation};
pub use languages::{language_meta, all_languages, LanguageMeta, TextDirection};
pub use locale::{DateFmt, Locale, TimeFmt};
pub use tools::SnippetPlugin;

use std::path::Path;
use std::sync::{OnceLock, RwLock};

use fs_error::FsError;

// ── Global instance ───────────────────────────────────────────────────────────

static GLOBAL_I18N: OnceLock<RwLock<I18n>> = OnceLock::new();

/// Initialize the global [`I18n`] instance from a `locales/` directory.
///
/// The active language is determined by the directory contents; defaults to `"en"`.
/// Must be called before using [`t`] or [`t_with`].
pub fn init(dir: &Path) -> Result<(), FsError> {
    let instance = I18n::load_dir(dir)?;
    GLOBAL_I18N
        .set(RwLock::new(instance))
        .map_err(|_| FsError::internal("global I18n already initialized"))
}

/// Initialize the global [`I18n`] instance with an explicit active language.
///
/// Must be called before using [`t`] or [`t_with`].
pub fn init_with_lang(dir: &Path, lang: &str) -> Result<(), FsError> {
    let instance = I18n::load_dir_with_lang(dir, lang, "en")?;
    GLOBAL_I18N
        .set(RwLock::new(instance))
        .map_err(|_| FsError::internal("global I18n already initialized"))
}

/// Initialize the global [`I18n`] instance with all built-in snippets.
///
/// Loads actions, nouns, status, errors, phrases, time, validation, and help
/// categories for English and German.  The active language is set to
/// `active_lang`; `"en"` is always the fallback.
///
/// Must be called before using [`t`] or [`t_with`].
pub fn init_with_builtins(active_lang: &str) -> Result<(), FsError> {
    let instance = snippets::builtin_i18n(active_lang)?;
    GLOBAL_I18N
        .set(RwLock::new(instance))
        .map_err(|_| FsError::internal("global I18n already initialized"))
}

/// Initialize the global [`I18n`] instance with built-in snippets **plus** all
/// provided [`SnippetPlugin`] implementations.
///
/// This is the recommended entry point for applications. Each crate that ships
/// translations implements [`SnippetPlugin`] and is passed here as a trait object.
/// All plugins are loaded into a single [`I18n`] instance before it is stored in
/// the global `OnceLock` — no partial-initialization window, no ordering issues.
///
/// # Example
///
/// ```rust,ignore
/// use fs_i18n::{init_with_plugins, SnippetPlugin};
///
/// struct MyPlugin;
/// impl SnippetPlugin for MyPlugin {
///     fn name(&self) -> &str { "my-app" }
///     fn snippets(&self) -> &[(&str, &str)] {
///         &[("en", include_str!("../assets/i18n/en.toml"))]
///     }
/// }
///
/// fs_i18n::init_with_plugins("en", &[&MyPlugin]).unwrap();
/// assert_eq!(fs_i18n::t("my.key"), "My Value");
/// ```
pub fn init_with_plugins(active_lang: &str, plugins: &[&dyn SnippetPlugin]) -> Result<(), FsError> {
    let mut instance = snippets::builtin_i18n(active_lang)?;
    for plugin in plugins {
        plugin.load_into(&mut instance)?;
    }
    GLOBAL_I18N
        .set(RwLock::new(instance))
        .map_err(|_| FsError::internal("global I18n already initialized"))
}

/// Switch the active language of the global [`I18n`] instance at runtime.
///
/// Has no effect when the global instance has not been initialized yet.
pub fn set_active_lang(lang: &str) {
    if let Some(lock) = GLOBAL_I18N.get() {
        if let Ok(mut i18n) = lock.write() {
            i18n.set_lang(lang);
        }
    }
}

/// Add a TOML language pack to the already-initialized global [`I18n`] instance.
///
/// Call this before [`set_active_lang`] when loading a user-installed language pack
/// from disk at runtime (e.g. `~/.local/share/fsn/i18n/{lang}/ui.toml`).
///
/// Has no effect when the global instance has not been initialized yet.
pub fn add_toml_lang(lang: &str, toml_src: &str) -> Result<(), FsError> {
    if let Some(lock) = GLOBAL_I18N.get() {
        if let Ok(mut i18n) = lock.write() {
            i18n.add_toml_str(lang, toml_src)?;
        }
    }
    Ok(())
}

/// Return the [`Locale`] (formatting rules) for the active language.
///
/// Use this to format numbers, floats, dates, and times according to the
/// user's language — regardless of where the data comes from.
///
/// Returns English/ISO rules when the global instance is not initialized yet.
///
/// # Example
///
/// ```rust,ignore
/// let price = fs_i18n::locale().fmt_float(price_f64, 2);
/// let date  = fs_i18n::locale().fmt_date(2026, 3, 22);
/// ```
pub fn locale() -> Locale {
    match GLOBAL_I18N.get() {
        Some(lock) => match lock.read() {
            Ok(i18n) => i18n.locale(),
            Err(_)   => Locale::for_lang("en"),
        },
        None => Locale::for_lang("en"),
    }
}

/// Return the active language code of the global [`I18n`] instance.
///
/// Returns `"en"` when the global instance has not been initialized yet.
pub fn active_lang() -> LanguageCode {
    match GLOBAL_I18N.get() {
        Some(lock) => match lock.read() {
            Ok(i18n) => i18n.lang(),
            Err(_) => LanguageCode::new("en"),
        },
        None => LanguageCode::new("en"),
    }
}

/// Translate a key using the global [`I18n`] instance.
///
/// Returns the key itself when the global instance is not initialized or the
/// key is not found.
pub fn t(key: &str) -> Translation {
    match GLOBAL_I18N.get() {
        Some(lock) => match lock.read() {
            Ok(i18n) => i18n.t(key),
            Err(_) => Translation::new(key.to_string()),
        },
        None => Translation::new(key.to_string()),
    }
}

/// Initialize the global [`I18n`] instance from a list of `(lang_code, toml_str)` pairs.
///
/// The first entry whose `lang_code` equals `"en"` (or the very first entry if none
/// is `"en"`) becomes the fallback language.
///
/// Must be called before using [`t`] or [`t_with`].
pub fn init_with_toml_strs(active_lang: &str, locales: &[(&str, &str)]) -> Result<(), FsError> {
    let fallback = locales
        .iter()
        .find(|(lang, _)| *lang == "en")
        .or_else(|| locales.first())
        .map(|(lang, _)| *lang)
        .unwrap_or("en");

    let mut instance = I18n::new(active_lang, fallback);
    for (lang, toml_src) in locales {
        instance.add_toml_str(lang, toml_src)?;
    }
    GLOBAL_I18N
        .set(RwLock::new(instance))
        .map_err(|_| FsError::internal("global I18n already initialized"))
}

/// Translate a key with named arguments using the global [`I18n`] instance.
///
/// Returns the key itself when the global instance is not initialized or the
/// key is not found.
pub fn t_with(key: &str, args: &[(&str, &str)]) -> Translation {
    match GLOBAL_I18N.get() {
        Some(lock) => match lock.read() {
            Ok(i18n) => i18n.t_with(key, args),
            Err(_) => Translation::new(key.to_string()),
        },
        None => Translation::new(key.to_string()),
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn en_bundle(src: &str) -> I18n {
        let mut i = I18n::new("en", "en");
        i.add_ftl("en", &[src.to_string()]).unwrap();
        i
    }

    #[test]
    fn simple_lookup() {
        let i = en_bundle("action-save = Save\n");
        assert_eq!(i.t("action-save"), "Save");
    }

    #[test]
    fn fallback_to_en() {
        let src = "action-save = Save\n";
        let mut i = I18n::new("de", "en");
        i.add_ftl("en", &[src.to_string()]).unwrap();
        // "de" has no bundle — falls back to "en"
        assert_eq!(i.t("action-save"), "Save");
    }

    #[test]
    fn fallback_to_key_when_missing() {
        let i = I18n::new("en", "en");
        assert_eq!(i.t("missing-key"), "missing-key");
    }

    #[test]
    fn variable_substitution() {
        let i = en_bundle("phrase-confirm-delete = Delete { $item }?\n");
        let result = i.t_with("phrase-confirm-delete", &[("item", "module")]);
        // Fluent wraps substituted values in Unicode bidi-isolation marks (\u{2068}/\u{2069})
        let stripped: String = result.chars()
            .filter(|&c| c != '\u{2068}' && c != '\u{2069}')
            .collect();
        assert_eq!(stripped, "Delete module?");
    }

    #[test]
    fn has_returns_true_for_existing_key() {
        let i = en_bundle("action-save = Save\n");
        assert!(i.has("action-save"));
        assert!(!i.has("nonexistent"));
    }
}
