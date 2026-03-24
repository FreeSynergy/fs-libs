/// Severity level of a single validation issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum IssueSeverity {
    /// Informational — deployment still works.
    Info,
    /// Degraded — deployment works but something is sub-optimal.
    Warning,
    /// Broken — deployment will fail without a fix.
    Error,
}

impl std::fmt::Display for IssueSeverity {
    /// Renders the compact single-character indicator: `i`, `⚠`, or `✗`.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            IssueSeverity::Info => "i",
            IssueSeverity::Warning => "⚠",
            IssueSeverity::Error => "✗",
        };
        f.write_str(s)
    }
}

// ── ValidationIssue ───────────────────────────────────────────────────────────

/// A single finding from a validation run.
#[derive(Debug, Clone)]
pub struct ValidationIssue {
    /// The field or path that has the problem (e.g. `"project.meta.id"`).
    pub field: String,
    /// Human-readable description of the problem.
    pub message: String,
    /// How bad is it?
    pub severity: IssueSeverity,
}

impl ValidationIssue {
    /// Construct an Error-severity issue.
    pub fn error(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
            severity: IssueSeverity::Error,
        }
    }

    /// Construct a Warning-severity issue.
    pub fn warning(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
            severity: IssueSeverity::Warning,
        }
    }

    /// Construct an Info-severity issue.
    pub fn info(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
            severity: IssueSeverity::Info,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn severity_ordering() {
        assert!(IssueSeverity::Error > IssueSeverity::Warning);
        assert!(IssueSeverity::Warning > IssueSeverity::Info);
    }

    #[test]
    fn indicator_chars() {
        assert_eq!(IssueSeverity::Info.to_string(), "i");
        assert_eq!(IssueSeverity::Warning.to_string(), "⚠");
        assert_eq!(IssueSeverity::Error.to_string(), "✗");
    }

    #[test]
    fn constructors_set_severity() {
        assert_eq!(
            ValidationIssue::error("f", "m").severity,
            IssueSeverity::Error
        );
        assert_eq!(
            ValidationIssue::warning("f", "m").severity,
            IssueSeverity::Warning
        );
        assert_eq!(
            ValidationIssue::info("f", "m").severity,
            IssueSeverity::Info
        );
    }
}
