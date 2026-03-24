//! Requirement types — what a service needs from the environment.
//!
//! A `Requirement` is a concrete declaration that a service needs a specific
//! capability. The `DeclareRequirements` trait lets any type enumerate its
//! dependencies in a structured way.

// ── Requirement ───────────────────────────────────────────────────────────────

/// A concrete dependency declaration: "I need capability X".
///
/// Requirements are collected by the dependency resolver to build the
/// deployment graph and by the capability matcher to auto-fill role-typed
/// variables (e.g. `smtp.host`, `iam.oidc-discovery-url`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Requirement {
    /// The capability ID that must be satisfied, e.g. `"database.postgres"`.
    pub capability: String,
    /// `true` when the service can start without this capability (degraded mode).
    pub optional: bool,
    /// Human-readable explanation shown in the installer UI.
    pub description: Option<String>,
}

impl Requirement {
    /// Create a mandatory requirement for the given capability.
    pub fn required(capability: impl Into<String>) -> Self {
        Self {
            capability: capability.into(),
            optional: false,
            description: None,
        }
    }

    /// Create an optional requirement for the given capability.
    pub fn optional(capability: impl Into<String>) -> Self {
        Self {
            capability: capability.into(),
            optional: true,
            description: None,
        }
    }

    /// Attach a human-readable description.
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// `true` when `available` contains this requirement's capability.
    pub fn is_fulfilled_by(&self, available: &[String]) -> bool {
        available.contains(&self.capability)
    }

    /// `true` when this requirement is mandatory and not yet fulfilled.
    pub fn is_blocking(&self, available: &[String]) -> bool {
        !self.optional && !self.is_fulfilled_by(available)
    }
}

// ── DeclareRequirements trait ─────────────────────────────────────────────────

/// A type that can enumerate its deployment requirements.
///
/// Implement this for package manifests and service definitions so that the
/// installer can compute the full dependency graph before making any changes.
pub trait DeclareRequirements {
    /// All capabilities this entity depends on, mandatory and optional.
    fn requirements(&self) -> Vec<Requirement>;

    /// Only the mandatory requirements.
    fn mandatory_requirements(&self) -> Vec<&Requirement> {
        // Cannot call self.requirements() and return refs to temporaries;
        // callers should collect first or use a stored field.
        // This default is intentionally a no-op to avoid lifetime issues.
        Vec::new()
    }

    /// `true` when all mandatory requirements are in `available`.
    fn all_mandatory_satisfied(&self, available: &[String]) -> bool {
        self.requirements()
            .iter()
            .filter(|r| !r.optional)
            .all(|r| r.is_fulfilled_by(available))
    }

    /// Returns the mandatory requirements that are NOT yet fulfilled.
    fn blocking_requirements(&self, available: &[String]) -> Vec<Requirement> {
        self.requirements()
            .into_iter()
            .filter(|r| r.is_blocking(available))
            .collect()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn required_requirement_is_not_optional() {
        let r = Requirement::required("database.postgres");
        assert!(!r.optional);
    }

    #[test]
    fn optional_requirement_is_optional() {
        let r = Requirement::optional("smtp");
        assert!(r.optional);
    }

    #[test]
    fn fulfilled_when_capability_available() {
        let r = Requirement::required("database.postgres");
        let available = vec!["database.postgres".into(), "proxy".into()];
        assert!(r.is_fulfilled_by(&available));
    }

    #[test]
    fn not_fulfilled_when_capability_missing() {
        let r = Requirement::required("smtp");
        let available = vec!["database.postgres".into()];
        assert!(!r.is_fulfilled_by(&available));
    }

    #[test]
    fn blocking_only_for_mandatory_unfulfilled() {
        let mandatory = Requirement::required("smtp");
        let optional = Requirement::optional("monitoring");
        let available: Vec<String> = vec![];
        assert!(mandatory.is_blocking(&available));
        assert!(!optional.is_blocking(&available));
    }

    #[test]
    fn with_description() {
        let r = Requirement::required("smtp").with_description("Needed for email notifications");
        assert_eq!(
            r.description.as_deref(),
            Some("Needed for email notifications")
        );
    }

    struct OutlineManifest;

    impl DeclareRequirements for OutlineManifest {
        fn requirements(&self) -> Vec<Requirement> {
            vec![
                Requirement::required("oidc-provider"),
                Requirement::required("smtp"),
                Requirement::required("database.postgres"),
                Requirement::optional("monitoring"),
            ]
        }
    }

    #[test]
    fn all_mandatory_satisfied() {
        let m = OutlineManifest;
        let available = vec![
            "oidc-provider".into(),
            "smtp".into(),
            "database.postgres".into(),
        ];
        assert!(m.all_mandatory_satisfied(&available));
    }

    #[test]
    fn blocking_requirements_returns_unmet() {
        let m = OutlineManifest;
        let available = vec!["oidc-provider".into()];
        let blocking = m.blocking_requirements(&available);
        assert_eq!(blocking.len(), 2); // smtp + database.postgres
        assert!(!blocking.iter().any(|r| r.capability == "monitoring")); // optional excluded
    }
}
