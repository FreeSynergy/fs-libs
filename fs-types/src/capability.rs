//! Capability trait — what a service provides and what it needs.
//!
//! Capabilities are the core of the FreeSynergy dependency resolution system.
//! Each service declares what it **provides** (e.g. `"oidc-provider"`, `"smtp"`)
//! and what it **requires** from other services.
//!
//! Requirements are expressed as [`Requirement`] structs (via [`DeclareRequirements`])
//! rather than plain strings, so the resolver understands optionality and can show
//! meaningful UI descriptions.

use crate::requirement::DeclareRequirements;

// ── Capability trait ──────────────────────────────────────────────────────────

/// Declares the capabilities a service or plugin provides and requires.
///
/// Implemented by package manifests and runtime service records. Used by the
/// dependency resolver to assemble a coherent deployment and by the capability
/// matcher to auto-fill role-typed variables.
///
/// Requirements are declared via the [`DeclareRequirements`] supertrait using
/// rich [`Requirement`] structs instead of bare strings, so the resolver knows
/// which deps are optional and can display human-readable descriptions.
///
/// # Example
///
/// ```rust
/// use fs_types::{Capability, requirement::{DeclareRequirements, Requirement}};
///
/// struct KanidmManifest;
///
/// impl DeclareRequirements for KanidmManifest {
///     fn requirements(&self) -> Vec<Requirement> {
///         vec![Requirement::required("database.postgres")]
///     }
/// }
///
/// impl Capability for KanidmManifest {
///     fn capability_id(&self) -> &str { "iam/kanidm" }
///     fn provides(&self) -> Vec<String> {
///         vec!["oidc-provider".into(), "scim-server".into(), "mfa".into()]
///     }
/// }
/// ```
pub trait Capability: DeclareRequirements + Send + Sync {
    /// Stable identifier for this capability declaration, e.g. `"iam/kanidm"`.
    fn capability_id(&self) -> &str;

    /// Capability IDs this entity exports / makes available to others.
    ///
    /// Examples: `"oidc-provider"`, `"smtp"`, `"proxy"`, `"database.postgres"`.
    fn provides(&self) -> Vec<String>;

    /// `true` when every **mandatory** required capability is present in `available`.
    ///
    /// Delegates to [`DeclareRequirements::all_mandatory_satisfied`].
    fn is_satisfied_by(&self, available: &[String]) -> bool {
        self.all_mandatory_satisfied(available)
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
    use crate::requirement::Requirement;

    struct MockService {
        id: &'static str,
        provides: Vec<String>,
        reqs: Vec<Requirement>,
    }

    impl DeclareRequirements for MockService {
        fn requirements(&self) -> Vec<Requirement> { self.reqs.clone() }
    }

    impl Capability for MockService {
        fn capability_id(&self) -> &str { self.id }
        fn provides(&self) -> Vec<String> { self.provides.clone() }
    }

    #[test]
    fn satisfied_when_all_requirements_available() {
        let svc = MockService {
            id: "wiki/outline",
            provides: vec!["wiki".into()],
            reqs: vec![
                Requirement::required("oidc-provider"),
                Requirement::required("smtp"),
            ],
        };
        let available = vec!["oidc-provider".into(), "smtp".into(), "proxy".into()];
        assert!(svc.is_satisfied_by(&available));
    }

    #[test]
    fn not_satisfied_when_requirement_missing() {
        let svc = MockService {
            id: "wiki/outline",
            provides: vec!["wiki".into()],
            reqs: vec![
                Requirement::required("oidc-provider"),
                Requirement::required("smtp"),
            ],
        };
        let available = vec!["oidc-provider".into()]; // smtp missing
        assert!(!svc.is_satisfied_by(&available));
    }

    #[test]
    fn no_requirements_always_satisfied() {
        let svc = MockService {
            id: "proxy/zentinel",
            provides: vec!["proxy".into()],
            reqs: vec![],
        };
        assert!(svc.is_satisfied_by(&[]));
    }

    #[test]
    fn explicitly_missing_defaults_empty() {
        let svc = MockService {
            id: "iam/kanidm",
            provides: vec!["oidc-provider".into()],
            reqs: vec![],
        };
        assert!(svc.explicitly_missing().is_empty());
    }

    #[test]
    fn optional_requirement_does_not_block_satisfaction() {
        let svc = MockService {
            id: "wiki/outline",
            provides: vec!["wiki".into()],
            reqs: vec![
                Requirement::required("oidc-provider"),
                Requirement::optional("monitoring"),  // optional — must not block
            ],
        };
        let available = vec!["oidc-provider".into()]; // monitoring absent but optional
        assert!(svc.is_satisfied_by(&available));
    }
}
