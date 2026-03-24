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
#[derive(Error, Debug)]
pub enum FsError {
    /// A configuration file is malformed or missing required fields.
    #[error("Configuration error: {0}")]
    Config(String),

    /// Underlying OS I/O failure.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// TOML (or other format) parse failure.
    #[error("Parse error: {0}")]
    Parse(String),

    /// A resource was expected but not found.
    #[error("Not found: {0}")]
    NotFound(String),

    /// A field failed validation rules.
    #[error("Validation error in `{field}`: {message}")]
    Validation { field: String, message: String },

    /// HTTP or other network-level failure.
    #[error("Network error: {0}")]
    Network(String),

    /// A plugin failed to load, initialize, or execute.
    #[error("Plugin error: {0}")]
    Plugin(String),

    /// Authentication or authorization failure.
    #[error("Auth error: {0}")]
    Auth(String),

    /// Catch-all for errors that don't fit a specific category.
    #[error("Internal error: {0}")]
    Internal(String),
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
        assert!(e.to_string().contains("missing field"));
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
