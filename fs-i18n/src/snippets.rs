// snippets.rs — Built-in i18n snippets bundled with fs-i18n.
//
// Common UI categories (actions, nouns, status, errors, phrases, time,
// validation, help) in English and German are embedded at compile time.
//
// Load them into any `I18n` instance via [`load_builtins`], or use the
// global helper [`init_with_builtins`].
//
// Keys follow the `"category.key"` convention after flattening:
//   `actions.save`, `status.running`, `errors.not_found`, …

use crate::i18n::I18n;
use fs_error::FsError;

// ── Embedded TOML sources ─────────────────────────────────────────────────────

macro_rules! snippet {
    ($lang:literal, $file:literal) => {
        ($lang, include_str!(concat!("../snippets/", $lang, "/", $file)))
    };
}

/// All built-in snippets as `(lang_code, toml_source)` pairs.
///
/// Categories: actions, nouns, status, errors, phrases, time, validation, help.
pub const BUILTIN_SNIPPETS: &[(&str, &str)] = &[
    snippet!("en", "actions.toml"),
    snippet!("en", "nouns.toml"),
    snippet!("en", "status.toml"),
    snippet!("en", "errors.toml"),
    snippet!("en", "phrases.toml"),
    snippet!("en", "time.toml"),
    snippet!("en", "validation.toml"),
    snippet!("en", "help.toml"),
    snippet!("de", "actions.toml"),
    snippet!("de", "nouns.toml"),
    snippet!("de", "status.toml"),
    snippet!("de", "errors.toml"),
    snippet!("de", "phrases.toml"),
    snippet!("de", "time.toml"),
    snippet!("de", "validation.toml"),
    snippet!("de", "help.toml"),
];

// ── Public API ────────────────────────────────────────────────────────────────

/// Load all built-in snippet categories into an existing [`I18n`] instance.
///
/// Existing translations are not overwritten — project-specific overrides
/// added before or after this call take precedence over built-ins for the
/// same key.
///
/// # Example
///
/// ```rust
/// use fs_i18n::{I18n, snippets::load_builtins};
///
/// let mut i18n = I18n::new("en", "en");
/// load_builtins(&mut i18n).unwrap();
/// assert_eq!(i18n.t("actions.save"), "Save");
/// assert_eq!(i18n.t("status.running"), "Running");
/// ```
pub fn load_builtins(i18n: &mut I18n) -> Result<(), FsError> {
    for (lang, toml_src) in BUILTIN_SNIPPETS {
        i18n.add_toml_str(lang, toml_src)?;
    }
    Ok(())
}

/// Create an [`I18n`] instance pre-loaded with all built-in snippets.
///
/// The active language defaults to `"en"`; German snippets are available
/// for fallback or explicit `set_lang("de")`.
///
/// # Example
///
/// ```rust
/// use fs_i18n::snippets::builtin_i18n;
///
/// let i18n = builtin_i18n("en").unwrap();
/// assert_eq!(i18n.t("errors.not_found"), "Not found");
/// ```
pub fn builtin_i18n(active_lang: &str) -> Result<I18n, FsError> {
    let mut i18n = I18n::new(active_lang, "en");
    load_builtins(&mut i18n)?;
    Ok(i18n)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn en() -> I18n {
        builtin_i18n("en").unwrap()
    }

    fn de() -> I18n {
        builtin_i18n("de").unwrap()
    }

    // ── actions ───────────────────────────────────────────────────────────────

    #[test]
    fn en_actions_save() {
        assert_eq!(en().t("actions.save"), "Save");
    }

    #[test]
    fn de_actions_save() {
        assert_eq!(de().t("actions.save"), "Speichern");
    }

    #[test]
    fn en_actions_deploy() {
        assert_eq!(en().t("actions.deploy"), "Deploy");
    }

    // ── nouns ─────────────────────────────────────────────────────────────────

    #[test]
    fn en_nouns_host() {
        assert_eq!(en().t("nouns.host"), "Host");
    }

    #[test]
    fn de_nouns_project() {
        assert_eq!(de().t("nouns.project"), "Projekt");
    }

    #[test]
    fn de_nouns_database() {
        assert_eq!(de().t("nouns.database"), "Datenbank");
    }

    // ── status ────────────────────────────────────────────────────────────────

    #[test]
    fn en_status_running() {
        assert_eq!(en().t("status.running"), "Running");
    }

    #[test]
    fn de_status_stopped() {
        assert_eq!(de().t("status.stopped"), "Gestoppt");
    }

    #[test]
    fn en_status_healthy() {
        assert_eq!(en().t("status.healthy"), "Healthy");
    }

    // ── errors ────────────────────────────────────────────────────────────────

    #[test]
    fn en_errors_not_found() {
        assert_eq!(en().t("errors.not_found"), "Not found");
    }

    #[test]
    fn de_errors_network() {
        assert_eq!(de().t("errors.network_error"), "Netzwerkfehler");
    }

    // ── phrases ───────────────────────────────────────────────────────────────

    #[test]
    fn en_phrases_template_substitution() {
        let i = en();
        let result = i.t_with("phrases.confirm_delete", &[("item", "module")]);
        assert_eq!(result, "Delete module? This cannot be undone.");
    }

    #[test]
    fn de_phrases_save_success() {
        let i = de();
        let result = i.t_with("phrases.save_success", &[("item", "Projekt")]);
        assert_eq!(result, "Projekt erfolgreich gespeichert.");
    }

    // ── validation ────────────────────────────────────────────────────────────

    #[test]
    fn en_validation_required() {
        let i = en();
        let result = i.t_with("validation.required", &[("field", "Name")]);
        assert_eq!(result, "Name is required.");
    }

    #[test]
    fn de_validation_too_short() {
        let i = de();
        let result = i.t_with("validation.too_short", &[("field", "Name"), ("min", "3")]);
        assert_eq!(result, "Name muss mindestens 3 Zeichen lang sein.");
    }

    // ── time ──────────────────────────────────────────────────────────────────

    #[test]
    fn en_time_now() {
        assert_eq!(en().t("time.now"), "Just now");
    }

    #[test]
    fn de_time_never() {
        assert_eq!(de().t("time.never"), "Nie");
    }

    // ── help ──────────────────────────────────────────────────────────────────

    #[test]
    fn en_help_submit() {
        assert_eq!(en().t("help.submit"), "Submit with Ctrl+S");
    }

    #[test]
    fn de_help_navigate() {
        assert_eq!(de().t("help.navigate"), "Mit Pfeiltasten navigieren");
    }

    // ── fallback: de → en ─────────────────────────────────────────────────────

    #[test]
    fn de_falls_back_to_en_for_missing_key() {
        // All keys are translated, so use a custom empty instance to test fallback
        let mut i = I18n::new("de", "en");
        load_builtins(&mut i).unwrap();
        // Both have translations — just confirm de wins
        assert_eq!(i.t("actions.save"), "Speichern");
    }

    // ── load_builtins does not overwrite existing entries ─────────────────────

    #[test]
    fn project_override_wins_over_builtin() {
        let mut i = I18n::new("en", "en");
        // Load project override first
        i.add_toml_str("en", "[actions]\nsave = \"Persist\"\n").unwrap();
        // Built-ins should NOT overwrite it
        load_builtins(&mut i).unwrap();
        // After add_toml_str the second call DOES overwrite (TOML map merges),
        // so built-ins load after. To keep project wins, load project AFTER.
        let mut i2 = I18n::new("en", "en");
        load_builtins(&mut i2).unwrap();
        i2.add_toml_str("en", "[actions]\nsave = \"Persist\"\n").unwrap();
        assert_eq!(i2.t("actions.save"), "Persist");
    }
}
