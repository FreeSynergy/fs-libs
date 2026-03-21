//! Wrapper around a single Fluent locale bundle.

use fluent_bundle::concurrent::FluentBundle;
use fluent_bundle::{FluentArgs, FluentResource, FluentValue};
use unic_langid::LanguageIdentifier;

use fs_error::FsError;

// ── LocaleBundle ──────────────────────────────────────────────────────────────

/// A compiled Fluent bundle for one locale.
///
/// Uses [`fluent_bundle::concurrent::FluentBundle`] which is `Send + Sync`,
/// allowing the bundle to be shared across threads via `OnceLock<RwLock<I18n>>`.
pub(crate) struct LocaleBundle {
    bundle: FluentBundle<FluentResource>,
}

impl LocaleBundle {
    /// Build a bundle from a locale ID string and a slice of FTL source strings.
    ///
    /// All sources are added to the same bundle in order; later messages
    /// silently overwrite earlier ones with the same id.
    pub fn new(lang: &str, ftl_sources: &[String]) -> Result<Self, FsError> {
        let lang_id: LanguageIdentifier = lang
            .parse()
            .map_err(|e| FsError::parse(format!("invalid locale id `{lang}`: {e}")))?;

        let mut bundle = FluentBundle::new_concurrent(vec![lang_id]);

        for source in ftl_sources {
            let resource = FluentResource::try_new(source.clone()).map_err(|(_, errors)| {
                FsError::parse(format!(
                    "FTL parse error in locale `{lang}`: {:?}",
                    errors
                ))
            })?;
            // Duplicate message ids are skipped by fluent-bundle — ignore the
            // `Err` variant which only carries override errors, not fatal ones.
            let _ = bundle.add_resource(resource);
        }

        Ok(Self { bundle })
    }

    /// Look up a message by key. Returns `None` when the key is absent.
    pub fn get(&self, key: &str) -> Option<String> {
        let msg = self.bundle.get_message(key)?;
        let pattern = msg.value()?;
        let mut errors = vec![];
        let value = self.bundle.format_pattern(pattern, None, &mut errors);
        Some(value.into_owned())
    }

    /// Look up a message with named string arguments.
    ///
    /// `args` is a slice of `(name, value)` pairs that map to Fluent variables.
    pub fn get_with_args(&self, key: &str, args: &[(&str, &str)]) -> Option<String> {
        let msg = self.bundle.get_message(key)?;
        let pattern = msg.value()?;

        let mut fluent_args = FluentArgs::new();
        for (k, v) in args {
            fluent_args.set(*k, FluentValue::from(*v));
        }

        let mut errors = vec![];
        let value = self.bundle.format_pattern(pattern, Some(&fluent_args), &mut errors);
        Some(value.into_owned())
    }
}
