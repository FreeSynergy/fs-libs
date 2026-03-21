//! Capability trait — what a service provides and what it needs.
//!
//! Capabilities are the core of the FreeSynergy dependency resolution system.
//! Each service declares what it **provides** (e.g. `"oidc-provider"`, `"smtp"`)
//! and what it **requires** from other services.

// ── Capability trait ──────────────────────────────────────────────────────────

/// Declares the capabilities a service or plugin provides and requires.
///
/// Implemented by package manifests and runtime service records. Used by the
/// dependency resolver to assemble a coherent deployment and by the capability
/// matcher to auto-fill role-typed variables.
///
/// # Example
///
/// ```rust
/// use fs_types::Capability;
///
/// struct KanidmManifest;
///
/// impl Capability for KanidmManifest {
///     fn capability_id(&self) -> &str { "iam/kanidm" }
///     fn provides(&self) -> Vec<String> {
///         vec!["oidc-provider".into(), "scim-server".into(), "mfa".into()]
///     }
///     fn requires(&self) -> Vec<String> {
///         vec!["database.postgres".into()]
///     }
/// }
/// ```
pub trait Capability: Send + Sync {
    /// Stable identifier for this capability declaration, e.g. `"iam/kanidm"`.
    fn capability_id(&self) -> &str;

    /// Capability IDs this entity exports / makes available to others.
    ///
    /// Examples: `"oidc-provider"`, `"smtp"`, `"proxy"`, `"database.postgres"`.
    fn provides(&self) -> Vec<String>;

    /// Capability IDs this entity needs from the environment to function.
    ///
    /// Examples: `"database.postgres"`, `"iam.oidc-discovery-url"`.
    fn requires(&self) -> Vec<String>;

    /// `true` when every required capability is present in `available`.
    ///
    /// Default implementation iterates `requires()` and checks containment.
    fn is_satisfied_by(&self, available: &[String]) -> bool {
        self.requires().iter().all(|req| available.contains(req))
    }

    /// Capability IDs this entity **does not** provide, for UI comparison tables.
    ///
    /// Override when you need to enumerate negatives (e.g. Kanidm has no SAML).
    /// Default returns an empty list.
    fn explicitly_missing(&self) -> Vec<String> {
        Vec::new()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    struct MockService {
        id: &'static str,
        provides: Vec<String>,
        requires: Vec<String>,
    }

    impl Capability for MockService {
        fn capability_id(&self) -> &str { self.id }
        fn provides(&self) -> Vec<String> { self.provides.clone() }
        fn requires(&self) -> Vec<String> { self.requires.clone() }
    }

    #[test]
    fn satisfied_when_all_requirements_available() {
        let svc = MockService {
            id: "wiki/outline",
            provides: vec!["wiki".into()],
            requires: vec!["oidc-provider".into(), "smtp".into()],
        };
        let available = vec!["oidc-provider".into(), "smtp".into(), "proxy".into()];
        assert!(svc.is_satisfied_by(&available));
    }

    #[test]
    fn not_satisfied_when_requirement_missing() {
        let svc = MockService {
            id: "wiki/outline",
            provides: vec!["wiki".into()],
            requires: vec!["oidc-provider".into(), "smtp".into()],
        };
        let available = vec!["oidc-provider".into()]; // smtp missing
        assert!(!svc.is_satisfied_by(&available));
    }

    #[test]
    fn no_requirements_always_satisfied() {
        let svc = MockService {
            id: "proxy/zentinel",
            provides: vec!["proxy".into()],
            requires: vec![],
        };
        assert!(svc.is_satisfied_by(&[]));
    }

    #[test]
    fn explicitly_missing_defaults_empty() {
        let svc = MockService {
            id: "iam/kanidm",
            provides: vec!["oidc-provider".into()],
            requires: vec![],
        };
        assert!(svc.explicitly_missing().is_empty());
    }
}
