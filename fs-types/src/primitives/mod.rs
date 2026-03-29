//! First-class value types for `FreeSynergy`.
//!
//! Every type here implements [`FsValue`], the self-describing value trait.
//! The GUI layer reads `type_label_key`, `placeholder_key`, and `help_key`
//! to render consistent input widgets with built-in i18n help — without any
//! knowledge of the concrete type.
//!
//! # Design
//!
//! ```text
//! FsValue trait        ← uniform interface for GUI + validation
//!   ├── FsUrl          ← URL + display label ("https://…", "FreeSynergy Docs")
//!   ├── LanguageCode   ← BCP-47 language code ("de", "ar", "yue")
//!   ├── SemVer         ← semantic version (1.2.3, 0.5.0-beta.1)
//!   └── FsPort         ← validated TCP/UDP port number (1–65535)
//! ```
//!
//! All types are serialized to/from their natural string form so TOML manifests
//! remain human-readable.

pub mod language;
pub mod port;
pub mod semver;
pub mod url;

pub use language::LanguageCode;
pub use port::FsPort;
pub use semver::SemVer;
pub use url::FsUrl;

// ── FsValue ───────────────────────────────────────────────────────────────────

/// Trait implemented by every first-class value type in `FreeSynergy`.
///
/// # Purpose
///
/// Decouples concrete types from the GUI and validation layers.
/// A form field of type `&dyn FsValue` can render itself, validate itself,
/// and provide user-facing help — all through i18n keys, never hardcoded text.
///
/// # i18n key convention
///
/// | Method              | Key pattern                 | Example value               |
/// |---------------------|-----------------------------|-----------------------------|
/// | `type_label_key`    | `type.<name>`               | `type.url`                  |
/// | `placeholder_key`   | `placeholder.<name>`        | `placeholder.url`           |
/// | `help_key`          | `help.<name>`               | `help.url`                  |
/// | `validate` error    | `error.validation.<reason>` | `error.validation.url`      |
///
/// # Object safety
///
/// This trait is fully object-safe — `Box<dyn FsValue>` and `&dyn FsValue` work.
pub trait FsValue: std::fmt::Debug + Send + Sync {
    /// i18n key for the human-readable type name.
    ///
    /// Shown in form labels, tooltips, and error messages.
    /// Example: `"type.url"` → `"URL"` (en), `"URL"` (de)
    fn type_label_key(&self) -> &'static str;

    /// i18n key for the placeholder text shown inside an empty input field.
    ///
    /// Example: `"placeholder.url"` → `"https://example.com"`
    fn placeholder_key(&self) -> &'static str;

    /// i18n key for the default help text shown below an input field.
    ///
    /// Used when no field-specific help text has been provided.
    /// Example: `"help.url"` → `"Enter a valid URL starting with https://"`
    fn help_key(&self) -> &'static str;

    /// Validate the current value.
    ///
    /// Returns `Ok(())` when valid.
    ///
    /// # Errors
    ///
    /// Returns `Err(key)` with an i18n key for the error message when invalid.
    fn validate(&self) -> Result<(), &'static str>;

    /// Format the value for read-only display (not for editing).
    ///
    /// For `FsUrl` this is the label, for `SemVer` the full version string, etc.
    fn display(&self) -> String;
}
