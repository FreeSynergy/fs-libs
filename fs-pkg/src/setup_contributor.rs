// setup_contributor.rs — Cross-package setup contributions.
//
// When Package B is being set up and requires a role that Package A provides,
// Package A can contribute setup steps into Package B's flow automatically.
//
// Example:
//   Forgejo requires "iam.oidc-provider".
//   Kanidm is installed and provides "iam.oidc-provider".
//   Kanidm implements SetupContributor { contributes_to_role: "iam.oidc-provider" }.
//   SetupFlow::add_contributions() asks all installed packages for contributions.
//   Kanidm contributes an OidcClientCreationStep into Forgejo's flow.
//   Forgejo's config fields for OIDC client_id + client_secret are auto-filled.
//
// Patterns used:
//   Visitor   — each package visits other packages' flows and contributes steps
//   Registry  — SetupContributorRegistry collects all available contributors

use crate::setup_flow::{SetupContext, SetupFlow};
use crate::setup_step::SetupStep;

// ── Contribution ──────────────────────────────────────────────────────────────

/// Description of what a contributor will add to the target flow.
///
/// The Manager UI shows this to the user BEFORE the steps are injected,
/// so they understand why a third-party package is participating in their setup.
pub struct Contribution {
    /// ID of the contributing package (e.g. `"kanidm"`).
    pub contributor_id: String,

    /// Human-readable title (e.g. `"Configure Kanidm OIDC client"`).
    pub title: String,

    /// Explanation of what this contribution does — shown in the right sidebar.
    ///
    /// Example: "Kanidm will automatically create an OIDC client for Forgejo
    /// and fill in the client ID and secret so you don't have to do it manually."
    pub description: String,

    /// Steps to inject into the target flow.
    pub steps: Vec<Box<dyn SetupStep>>,
}

impl Contribution {
    pub fn new(
        contributor_id: impl Into<String>,
        title:          impl Into<String>,
        description:    impl Into<String>,
        steps:          Vec<Box<dyn SetupStep>>,
    ) -> Self {
        Self {
            contributor_id: contributor_id.into(),
            title:          title.into(),
            description:    description.into(),
            steps,
        }
    }
}

// ── SetupContributor (trait) ──────────────────────────────────────────────────

/// A package that contributes setup steps to another package's flow.
///
/// Implementing this trait makes a package "aware" of other packages.
/// When a target package's setup flow is built, the `SetupFlow::add_contributions()`
/// method queries all registered contributors and injects their steps.
///
/// # When to implement
///
/// Implement this for packages that:
/// - Provide a role that other packages depend on
/// - Can auto-configure themselves from within another package's setup
///
/// # Example: Kanidm → Forgejo
///
/// ```no_run
/// struct KanidmContributor { api_url: String }
///
/// impl SetupContributor for KanidmContributor {
///     fn contributor_id(&self) -> &str { "kanidm" }
///
///     fn contributes_to_role(&self) -> &str { "iam.oidc-provider" }
///
///     fn build_contribution(&self, for_package: &str, ctx: &SetupContext) -> Option<Contribution> {
///         // Build a CommandStep that calls the Kanidm API to create an OIDC client
///         // for `for_package`, then returns the client_id and client_secret.
///         Some(Contribution::new(
///             "kanidm",
///             "Set up Kanidm OIDC for Forgejo",
///             "Kanidm will create an OIDC client automatically.",
///             vec![ /* steps */ ],
///         ))
///     }
/// }
/// ```
pub trait SetupContributor: Send + Sync {
    /// The ID of this contributing package (e.g. `"kanidm"`).
    fn contributor_id(&self) -> &str;

    /// The role that must be required by the target package for contributions
    /// to be applicable.
    ///
    /// The SetupFlow only calls `build_contribution()` when the target package
    /// declares that it requires (or optionally uses) this role.
    fn contributes_to_role(&self) -> &str;

    /// Build the contribution for the given target package.
    ///
    /// Return `None` if this contribution is not applicable in the current
    /// context (e.g. required service is not reachable, already configured).
    ///
    /// `for_package` — the ID of the package being set up.
    /// `ctx`         — the target package's current setup context.
    fn build_contribution(
        &self,
        for_package: &str,
        ctx:         &SetupContext,
    ) -> Option<Contribution>;

    /// Whether this contribution is required (blocks start if not done).
    ///
    /// Return `true` for contributions that set mandatory config values
    /// (e.g. OIDC client_id). Return `false` for optional enhancements.
    fn is_required(&self) -> bool { false }
}

// ── SetupContributorRegistry ──────────────────────────────────────────────────

/// Registry of all active SetupContributors.
///
/// The Manager builds this from all installed packages that implement
/// `Manageable::setup_contributors()`. When building a package's setup flow,
/// the Manager passes the registry to `SetupFlow::add_contributions()`.
///
/// Pattern: Registry (maps role → contributor list)
pub struct SetupContributorRegistry {
    /// All registered contributors, keyed by the role they serve.
    contributors: Vec<Box<dyn SetupContributor>>,
}

impl SetupContributorRegistry {
    pub fn new() -> Self {
        Self { contributors: vec![] }
    }

    /// Register a contributor.
    pub fn register(&mut self, c: Box<dyn SetupContributor>) {
        self.contributors.push(c);
    }

    /// Find all contributors whose role matches any of the given required roles.
    pub fn contributors_for_roles(&self, required_roles: &[String]) -> Vec<&dyn SetupContributor> {
        self.contributors.iter()
            .filter(|c| {
                let role = c.contributes_to_role();
                required_roles.iter().any(|r| {
                    r == role || r.starts_with(&format!("{role}.")) || role.starts_with(&format!("{r}."))
                })
            })
            .map(|c| c.as_ref())
            .collect()
    }
}

impl Default for SetupContributorRegistry {
    fn default() -> Self { Self::new() }
}

// ── SetupFlow extension ───────────────────────────────────────────────────────

impl SetupFlow {
    /// Inject contributed steps from all matching contributors in the registry.
    ///
    /// Called by the Manager after building the package's own flow, before
    /// presenting the wizard to the user.
    ///
    /// Steps are injected at the END of the flow — after the package's own steps.
    /// Contribution order matches registration order in the registry.
    pub fn add_contributions(
        &mut self,
        for_package:      &str,
        required_roles:   &[String],
        registry:         &SetupContributorRegistry,
    ) {
        for contributor in registry.contributors_for_roles(required_roles) {
            if let Some(contribution) = contributor.build_contribution(for_package, &self.context) {
                for step in contribution.steps {
                    self.add_step(step);
                }
            }
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::setup_flow::SetupContext;
    use crate::setup_step::{InputField, InputStep, SetupTrigger, StepOutput};
    use crate::manageable::ConfigFieldKind;
    use fs_error::FsError;

    // A fake OIDC contributor that injects one input step.
    struct FakeOidcContributor;

    impl SetupContributor for FakeOidcContributor {
        fn contributor_id(&self)      -> &str { "kanidm" }
        fn contributes_to_role(&self) -> &str { "iam.oidc-provider" }

        fn build_contribution(&self, for_package: &str, _ctx: &SetupContext) -> Option<Contribution> {
            let step = InputStep::new(
                "kanidm_oidc_client",
                "Create Kanidm OIDC client",
                "Kanidm will create an OIDC client for this application automatically.",
                vec![SetupTrigger::FirstInstall],
            )
            .with_field(InputField::new(
                "oidc_client_id",
                "OIDC Client ID",
                "The client ID registered in Kanidm.",
                ConfigFieldKind::Text,
            ).with_default(&format!("{for_package}-client")));

            Some(Contribution::new(
                "kanidm",
                "Configure Kanidm OIDC",
                "Kanidm creates an OIDC client for this service automatically.",
                vec![Box::new(step)],
            ))
        }
    }

    #[test]
    fn registry_matches_exact_role() {
        let mut reg = SetupContributorRegistry::new();
        reg.register(Box::new(FakeOidcContributor));

        let matching = reg.contributors_for_roles(&["iam.oidc-provider".to_string()]);
        assert_eq!(matching.len(), 1);
        assert_eq!(matching[0].contributor_id(), "kanidm");
    }

    #[test]
    fn registry_matches_parent_role() {
        let mut reg = SetupContributorRegistry::new();
        reg.register(Box::new(FakeOidcContributor));

        // Contributor role is "iam.oidc-provider", required role is "iam" — should match.
        let matching = reg.contributors_for_roles(&["iam".to_string()]);
        assert_eq!(matching.len(), 1);
    }

    #[test]
    fn registry_no_match_for_unrelated_role() {
        let mut reg = SetupContributorRegistry::new();
        reg.register(Box::new(FakeOidcContributor));

        let matching = reg.contributors_for_roles(&["smtp".to_string()]);
        assert!(matching.is_empty());
    }

    #[test]
    fn flow_add_contributions_injects_steps() {
        let ctx = SetupContext::new("forgejo");
        let mut flow = SetupFlow::new(ctx);

        let mut reg = SetupContributorRegistry::new();
        reg.register(Box::new(FakeOidcContributor));

        flow.add_contributions(
            "forgejo",
            &["iam.oidc-provider".to_string()],
            &reg,
        );

        // The contributed step is now in the flow.
        let trigger_steps = flow.steps_for_trigger(&SetupTrigger::FirstInstall);
        assert_eq!(trigger_steps.len(), 1);
        assert_eq!(trigger_steps[0].id(), "kanidm_oidc_client");
    }

    #[test]
    fn contributed_step_applies_default() {
        let ctx = SetupContext::new("forgejo");
        let mut flow = SetupFlow::new(ctx);

        let mut reg = SetupContributorRegistry::new();
        reg.register(Box::new(FakeOidcContributor));

        flow.add_contributions("forgejo", &["iam.oidc-provider".to_string()], &reg);

        let results = flow.execute_trigger(SetupTrigger::FirstInstall);
        assert!(results[0].succeeded());
        assert_eq!(flow.context.config_value("oidc_client_id"), Some("forgejo-client"));
    }
}
