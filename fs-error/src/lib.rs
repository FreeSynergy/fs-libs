#![deny(clippy::all, clippy::pedantic, warnings)]
#![allow(clippy::module_name_repetitions)]
// fs-error — Error handling + Repairable trait for the FreeSynergy ecosystem.
//
// Modules:
//   severity    — ErrorSeverity enum (Info/Warn/Error/Fatal)
//   error_trait — FsErrorTrait (code/ftl_key/severity/cause)
//   validation  — IssueSeverity, ValidationIssue
//   repair      — RepairAction, RepairOption, RepairOutcome, Repairable trait
//
// FsError lives here in lib.rs (main error enum, covers all subsystems).

use thiserror::Error;

pub mod error_trait;
pub mod repair;
pub mod severity;
pub mod validation;

// ── Flat re-exports ───────────────────────────────────────────────────────────

pub use error_trait::FsErrorTrait;
pub use repair::{RepairAction, RepairOption, RepairOutcome, Repairable};
pub use severity::ErrorSeverity;
pub use validation::{IssueSeverity, ValidationIssue};

// ── FsError ──────────────────────────────────────────────────────────────────

/// Main error type for all `FreeSynergy` library crates.
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
    Validation {
        /// Dot-separated field path that failed validation, e.g. `"project.name"`.
        field: String,
        /// Human-readable description of the violation (developer-facing).
        message: String,
    },

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

// ── FsErrorTrait impl ─────────────────────────────────────────────────────────

impl FsErrorTrait for FsError {
    fn code(&self) -> &'static str {
        match self {
            Self::Config(_) => "config",
            Self::Io(_) => "io",
            Self::Parse(_) => "parse",
            Self::NotFound(_) => "not_found",
            Self::Validation { .. } => "validation",
            Self::Network(_) => "network",
            Self::Plugin(_) => "plugin",
            Self::Auth(_) => "auth",
            Self::Internal(_) => "internal",
        }
    }

    fn ftl_key(&self) -> &'static str {
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

    fn severity(&self) -> ErrorSeverity {
        match self {
            Self::NotFound(_) | Self::Validation { .. } => ErrorSeverity::Warn,
            Self::Config(_)
            | Self::Parse(_)
            | Self::Io(_)
            | Self::Network(_)
            | Self::Plugin(_)
            | Self::Auth(_) => ErrorSeverity::Error,
            Self::Internal(_) => ErrorSeverity::Fatal,
        }
    }
}

// ── Convenience constructors ──────────────────────────────────────────────────

impl FsError {
    /// Construct a `Config` error.
    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }

    /// Construct a `Parse` error.
    pub fn parse(msg: impl Into<String>) -> Self {
        Self::Parse(msg.into())
    }

    /// Construct a `NotFound` error.
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }

    /// Construct a `Network` error.
    pub fn network(msg: impl Into<String>) -> Self {
        Self::Network(msg.into())
    }

    /// Construct an `Internal` error.
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }

    /// Construct an `Auth` error.
    pub fn auth(msg: impl Into<String>) -> Self {
        Self::Auth(msg.into())
    }

    /// Construct a `Validation` error.
    pub fn validation(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Validation {
            field: field.into(),
            message: message.into(),
        }
    }
}

// ── Additional From impls ─────────────────────────────────────────────────────

impl From<std::num::ParseIntError> for FsError {
    fn from(e: std::num::ParseIntError) -> Self {
        Self::Parse(e.to_string())
    }
}

impl From<std::num::ParseFloatError> for FsError {
    fn from(e: std::num::ParseFloatError) -> Self {
        Self::Parse(e.to_string())
    }
}

impl From<std::string::FromUtf8Error> for FsError {
    fn from(e: std::string::FromUtf8Error) -> Self {
        Self::Parse(e.to_string())
    }
}

impl From<std::str::Utf8Error> for FsError {
    fn from(e: std::str::Utf8Error) -> Self {
        Self::Parse(e.to_string())
    }
}

// ── Backward-compatibility alias ──────────────────────────────────────────────

/// Backward-compatibility alias — FreeSynergy.Node used `FsyError` before the rename.
///
/// Prefer `FsError` in new code.
pub type FsyError = FsError;

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fs_error_display() {
        let e = FsError::config("missing field");
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
    fn fs_error_codes() {
        assert_eq!(FsError::config("x").code(), "config");
        assert_eq!(FsError::parse("x").code(), "parse");
        assert_eq!(FsError::not_found("x").code(), "not_found");
        assert_eq!(FsError::network("x").code(), "network");
        assert_eq!(FsError::internal("x").code(), "internal");
        assert_eq!(FsError::auth("x").code(), "auth");
        assert_eq!(FsError::validation("f", "m").code(), "validation");
    }

    #[test]
    fn fs_error_severity() {
        assert_eq!(FsError::config("x").severity(), ErrorSeverity::Error);
        assert_eq!(FsError::not_found("x").severity(), ErrorSeverity::Warn);
        assert_eq!(
            FsError::validation("f", "m").severity(),
            ErrorSeverity::Warn
        );
        assert_eq!(FsError::internal("x").severity(), ErrorSeverity::Fatal);
        assert_eq!(FsError::auth("x").severity(), ErrorSeverity::Error);
    }

    #[test]
    fn fs_error_all_constructors() {
        assert!(FsError::parse("x").to_string().contains('x'));
        assert!(FsError::not_found("x").to_string().contains('x'));
        assert!(FsError::network("x").to_string().contains('x'));
        assert!(FsError::internal("x").to_string().contains('x'));
        assert!(FsError::auth("x").to_string().contains('x'));
        let e = FsError::validation("field", "bad");
        assert!(e.to_string().contains("field"));
        assert!(e.to_string().contains("bad"));
    }

    #[test]
    fn from_parse_int_error() {
        let e: FsError = "abc".parse::<i32>().unwrap_err().into();
        assert!(matches!(e, FsError::Parse(_)));
    }

    #[test]
    fn from_parse_float_error() {
        let e: FsError = "abc".parse::<f64>().unwrap_err().into();
        assert!(matches!(e, FsError::Parse(_)));
    }

    #[test]
    fn from_utf8_error() {
        let invalid = vec![0xFF, 0xFE];
        let e: FsError = String::from_utf8(invalid).unwrap_err().into();
        assert!(matches!(e, FsError::Parse(_)));
    }
}
