/// Runtime error severity — describes how serious an [`super::FsError`] is.
///
/// Unlike [`super::validation::IssueSeverity`] (which classifies config issues),
/// `ErrorSeverity` classifies runtime errors and drives decisions like
/// crash vs. retry vs. log-and-continue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorSeverity {
    /// Informational — the operation succeeded but something noteworthy happened.
    Info,
    /// Degraded — the operation partially succeeded; may need attention.
    Warn,
    /// Failed — the operation could not complete; caller should handle.
    Error,
    /// Critical — the system cannot continue; process should abort.
    Fatal,
}

impl std::fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Info => "info",
            Self::Warn => "warn",
            Self::Error => "error",
            Self::Fatal => "fatal",
        })
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn severity_ordering() {
        assert!(ErrorSeverity::Fatal > ErrorSeverity::Error);
        assert!(ErrorSeverity::Error > ErrorSeverity::Warn);
        assert!(ErrorSeverity::Warn > ErrorSeverity::Info);
    }

    #[test]
    fn severity_display() {
        assert_eq!(ErrorSeverity::Info.to_string(), "info");
        assert_eq!(ErrorSeverity::Warn.to_string(), "warn");
        assert_eq!(ErrorSeverity::Error.to_string(), "error");
        assert_eq!(ErrorSeverity::Fatal.to_string(), "fatal");
    }
}
