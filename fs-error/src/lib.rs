// fs-error — Error handling + Repairable trait for the FreeSynergy ecosystem.
//
// Modules:
//   validation  — IssueSeverity, ValidationIssue
//   repair      — RepairAction, RepairOption, RepairOutcome, Repairable trait
//
// FsError lives here in lib.rs (main error enum, covers all subsystems).

use thiserror::Error;

pub mod repair;
pub mod validation;

// ── Flat re-exports ───────────────────────────────────────────────────────────

pub use repair::{RepairAction, RepairOption, RepairOutcome, Repairable};
pub use validation::{IssueSeverity, ValidationIssue};

// ── FsError ──────────────────────────────────────────────────────────────────

/// Main error type for all FreeSynergy library crates.
///
/// Each variant carries a machine-readable FTL key (see [`FsError::ftl_key`])
/// and an optional technical detail string (for logs / developer context).
/// UI code should resolve the key via `fs_i18n` — never display the raw detail.
#[derive(Error, Debug)]
pub enum FsError {
    /// A configuration file is malformed or missing required fields.
    #[error("error-config: {0}")]
    Config(String),

    /// Underlying OS I/O failure.
    #[error("error-io: {0}")]
    Io(#[from] std::io::Error),

    /// TOML (or other format) parse failure.
    #[error("error-parse: {0}")]
    Parse(String),

    /// A resource was expected but not found.
    #[error("error-not-found: {0}")]
    NotFound(String),

    /// A field failed validation rules.
    #[error("error-validation: field={field} {message}")]
    Validation { field: String, message: String },

    /// HTTP or other network-level failure.
    #[error("error-network: {0}")]
    Network(String),

    /// A plugin failed to load, initialize, or execute.
    #[error("error-plugin: {0}")]
    Plugin(String),

    /// Authentication or authorization failure.
    #[error("error-auth: {0}")]
    Auth(String),

    /// Catch-all for errors that don't fit a specific category.
    #[error("error-internal: {0}")]
    Internal(String),
}

impl FsError {
    /// Returns the i18n snippet key for this error variant.
    ///
    /// Keys follow the `"category.name"` convention used by `fs_i18n` snippets.
    /// Use `i18n.t(error.ftl_key())` to get the user-facing translated message.
    /// Never display the raw `Display` output to end users — it is for logs only.
    #[must_use]
    pub fn ftl_key(&self) -> &'static str {
        match self {
            Self::Config(_) => "errors.config_error",
            Self::Io(_) => "errors.io_error",
            Self::Parse(_) => "errors.parse_error",
            Self::NotFound(_) => "errors.not_found",
            Self::Validation { .. } => "errors.validation_required",
            Self::Network(_) => "errors.network_error",
            Self::Plugin(_) => "errors.plugin_error",
            Self::Auth(_) => "errors.authentication_failed",
            Self::Internal(_) => "errors.internal_error",
        }
    }
}

/// Backward-compatibility alias — FreeSynergy.Node used `FsyError` before the rename.
///
/// Prefer `FsError` in new code.
pub type FsyError = FsError;

impl FsError {
    /// Convenience constructor for Config errors.
    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }
    /// Convenience constructor for Parse errors.
    pub fn parse(msg: impl Into<String>) -> Self {
        Self::Parse(msg.into())
    }
    /// Convenience constructor for NotFound errors.
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }
    /// Convenience constructor for Network errors.
    pub fn network(msg: impl Into<String>) -> Self {
        Self::Network(msg.into())
    }
    /// Convenience constructor for Internal errors.
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }
    /// Convenience constructor for Auth errors.
    pub fn auth(msg: impl Into<String>) -> Self {
        Self::Auth(msg.into())
    }
    /// Convenience constructor for Validation errors.
    pub fn validation(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Validation {
            field: field.into(),
            message: message.into(),
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fs_error_display() {
        let e = FsError::config("missing field");
        // Display shows "error-config: <detail>" — key prefix + technical detail
        assert!(e.to_string().contains("missing field"));
        assert!(e.to_string().contains("error-config"));
    }

    #[test]
    fn fs_error_ftl_keys() {
        assert_eq!(FsError::config("x").ftl_key(), "errors.config_error");
        assert_eq!(FsError::parse("x").ftl_key(), "errors.parse_error");
        assert_eq!(FsError::not_found("x").ftl_key(), "errors.not_found");
        assert_eq!(FsError::network("x").ftl_key(), "errors.network_error");
        assert_eq!(FsError::internal("x").ftl_key(), "errors.internal_error");
        assert_eq!(FsError::auth("x").ftl_key(), "errors.authentication_failed");
    }

    #[test]
    fn fs_error_all_constructors() {
        assert!(FsError::parse("x").to_string().contains("x"));
        assert!(FsError::not_found("x").to_string().contains("x"));
        assert!(FsError::network("x").to_string().contains("x"));
        assert!(FsError::internal("x").to_string().contains("x"));
        assert!(FsError::auth("x").to_string().contains("x"));
        let e = FsError::validation("field", "bad");
        assert!(e.to_string().contains("field"));
        assert!(e.to_string().contains("bad"));
    }
}
