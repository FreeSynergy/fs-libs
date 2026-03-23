// setup_flow.rs — SetupFlow: the orchestrator of package setup.
//
// SetupFlow is a persistent, ordered pipeline of SetupSteps. It answers:
//   - Which steps need to run for a given trigger?
//   - Which steps are still pending (blocking start)?
//   - Is the package ready to start?
//
// SetupContext is the shared mutable state that passes through all steps.
// It is serialized to `.setup-state.toml` in the package's config directory
// so the wizard can resume after being closed.
//
// Patterns used:
//   Chain of Responsibility — steps execute in sequence for a given trigger
//   Context (GoF)           — SetupContext carries all shared state
//   Mediator                — SetupFlow coordinates steps and context
//   Repository              — load/save context from/to disk

use std::collections::HashMap;
use std::path::Path;

use fs_error::FsError;

use crate::setup_step::{SetupStep, SetupTrigger, StepOutput, StepState};

// ── ServiceRef ────────────────────────────────────────────────────────────────

/// A reference to another installed service — used for cross-package setup.
///
/// The SetupFlow queries the Inventory for all installed services and passes
/// their references in the context so steps can interact with them
/// (e.g. creating an OIDC client in Kanidm from Forgejo's setup flow).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ServiceRef {
    /// Package ID (e.g. `"kanidm"`).
    pub id: String,
    /// Roles this service provides (e.g. `["iam", "iam.oidc-provider"]`).
    pub roles: Vec<String>,
    /// Base API URL (e.g. `"https://auth.example.com"`).
    pub api_url: Option<String>,
}

impl ServiceRef {
    pub fn new(id: impl Into<String>, roles: Vec<String>) -> Self {
        Self { id: id.into(), roles, api_url: None }
    }

    pub fn with_api_url(mut self, url: impl Into<String>) -> Self {
        self.api_url = Some(url.into());
        self
    }

    /// Returns true when this service provides the given role.
    pub fn provides_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role || r.starts_with(&format!("{role}.")))
    }
}

// ── SetupContext ──────────────────────────────────────────────────────────────

/// Shared mutable state that flows through all setup steps.
///
/// Persisted to `.setup-state.toml` alongside the package config file.
/// This allows the setup wizard to resume exactly where it left off.
///
/// # Responsibility
///
/// - Holds all user-entered and auto-generated config values
/// - Tracks which steps are done, failed, or pending
/// - Provides read/write access for steps during execution
/// - References other installed services (for cross-package steps)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SetupContext {
    /// Package this context belongs to.
    pub package_id: String,

    /// All config values (user input + defaults + generated secrets).
    ///
    /// Keys match `SetupField::key` and `InputField::key`.
    /// These are written into the actual package config via `WriteConfigStep`.
    #[serde(default)]
    pub config: HashMap<String, String>,

    /// Persisted state of each step, keyed by step ID.
    #[serde(default)]
    pub step_states: HashMap<String, StepState>,

    /// Installation directory of the package (used by CommandStep).
    #[serde(default)]
    pub install_path: Option<String>,

    /// Config directory of the package (used for saving this context).
    #[serde(default)]
    pub config_dir: Option<String>,

    /// References to other installed services (populated by SetupFlow before
    /// running cross-package steps contributed by SetupContributors).
    #[serde(default)]
    pub available_services: Vec<ServiceRef>,
}

impl SetupContext {
    // ── Constructors ──────────────────────────────────────────────────────────

    pub fn new(package_id: impl Into<String>) -> Self {
        Self {
            package_id:         package_id.into(),
            config:             HashMap::new(),
            step_states:        HashMap::new(),
            install_path:       None,
            config_dir:         None,
            available_services: vec![],
        }
    }

    pub fn with_install_path(mut self, path: impl Into<String>) -> Self {
        self.install_path = Some(path.into());
        self
    }

    pub fn with_config_dir(mut self, path: impl Into<String>) -> Self {
        self.config_dir = Some(path.into());
        self
    }

    pub fn with_services(mut self, services: Vec<ServiceRef>) -> Self {
        self.available_services = services;
        self
    }

    // ── Config values ─────────────────────────────────────────────────────────

    /// Get a config value by key.
    pub fn config_value(&self, key: &str) -> Option<&str> {
        self.config.get(key).map(|s| s.as_str())
    }

    /// Set a config value (overwrites any existing value).
    pub fn set_config_value(&mut self, key: &str, value: &str) {
        self.config.insert(key.to_string(), value.to_string());
    }

    /// Apply multiple config values at once.
    pub fn apply_config_values(&mut self, values: &HashMap<String, String>) {
        for (k, v) in values {
            self.config.insert(k.clone(), v.clone());
        }
    }

    // ── Step state ────────────────────────────────────────────────────────────

    /// Get the current state of a step.
    pub fn step_state(&self, step_id: &str) -> &StepState {
        self.step_states.get(step_id).unwrap_or(&StepState::Pending)
    }

    /// Mark a step as done with its output.
    pub fn mark_done(&mut self, step_id: &str, output: StepOutput) {
        self.step_states.insert(step_id.to_string(), StepState::Done { output });
    }

    /// Mark a step as failed with an error message.
    pub fn mark_failed(&mut self, step_id: &str, error: impl Into<String>) {
        self.step_states.insert(step_id.to_string(), StepState::Failed { error: error.into() });
    }

    /// Mark a step as skipped.
    pub fn mark_skipped(&mut self, step_id: &str) {
        self.step_states.insert(step_id.to_string(), StepState::Skipped);
    }

    /// Reset a step to pending (for re-running).
    pub fn reset_step(&mut self, step_id: &str) {
        self.step_states.remove(step_id);
    }

    // ── Service queries ───────────────────────────────────────────────────────

    /// Find all available services that provide the given role.
    pub fn services_with_role(&self, role: &str) -> Vec<&ServiceRef> {
        self.available_services.iter()
            .filter(|s| s.provides_role(role))
            .collect()
    }

    /// Find the first available service that provides the given role.
    pub fn first_service_with_role(&self, role: &str) -> Option<&ServiceRef> {
        self.available_services.iter().find(|s| s.provides_role(role))
    }

    // ── Persistence ───────────────────────────────────────────────────────────

    /// Load the context from disk (`.setup-state.toml`).
    ///
    /// Returns a fresh context if the file does not exist.
    pub fn load(package_id: &str, config_dir: &Path) -> Self {
        let path = config_dir.join(".setup-state.toml");
        if !path.exists() {
            return Self::new(package_id)
                .with_config_dir(config_dir.to_string_lossy().as_ref());
        }
        let text = std::fs::read_to_string(&path).unwrap_or_default();
        toml::from_str::<Self>(&text).unwrap_or_else(|_| {
            Self::new(package_id)
                .with_config_dir(config_dir.to_string_lossy().as_ref())
        })
    }

    /// Persist the context to disk (`.setup-state.toml`).
    pub fn save(&self) -> Result<(), FsError> {
        let dir = self.config_dir.as_deref().unwrap_or(".");
        std::fs::create_dir_all(dir)
            .map_err(|e| FsError::Internal(format!("Cannot create config dir: {e}")))?;

        let path = Path::new(dir).join(".setup-state.toml");
        let text = toml::to_string_pretty(self)
            .map_err(|e| FsError::Internal(format!("Serialize error: {e}")))?;

        std::fs::write(&path, text)
            .map_err(|e| FsError::Internal(format!("Cannot write setup state: {e}")))?;

        Ok(())
    }
}

// ── StepExecution ─────────────────────────────────────────────────────────────

/// Result of one step execution within a flow run.
///
/// Returned by `SetupFlow::execute_trigger()` so the UI can display per-step
/// success/failure without running everything in a black box.
#[derive(Debug)]
pub struct StepExecution {
    /// ID of the step.
    pub step_id:    String,
    /// Human-readable title of the step.
    pub step_title: String,
    /// Result of execution.
    pub result:     Result<StepOutput, FsError>,
    /// Whether the step was skipped (already done, not re-run).
    pub skipped:    bool,
}

impl StepExecution {
    pub fn succeeded(&self) -> bool { self.result.is_ok() && !self.skipped }
    pub fn failed(&self)    -> bool { self.result.is_err() }
}

// ── SetupFlow ─────────────────────────────────────────────────────────────────

/// The ordered pipeline of setup steps for one package.
///
/// Responsibilities:
/// - Hold all steps (required + contributed by other packages)
/// - Execute steps matching a given trigger, in order
/// - Persist context after each step
/// - Report which steps are still pending (blocking start)
///
/// # Ordering
///
/// Steps execute in the order they were added. Dependencies between steps
/// are expressed by ordering — a `WaitForServiceStep` placed after a
/// `CommandStep` that starts the service will wait correctly.
///
/// # Usage
///
/// ```no_run
/// use fs_pkg::setup_flow::{SetupFlow, SetupContext};
/// use fs_pkg::setup_step::SetupTrigger;
///
/// let mut flow = SetupFlow::new(SetupContext::new("kanidm"));
/// // add steps...
/// let results = flow.execute_trigger(SetupTrigger::FirstInstall);
/// ```
pub struct SetupFlow {
    /// Ordered list of steps.
    steps:      Vec<Box<dyn SetupStep>>,
    /// Shared mutable context.
    pub context: SetupContext,
}

impl SetupFlow {
    // ── Constructors ──────────────────────────────────────────────────────────

    pub fn new(context: SetupContext) -> Self {
        Self { steps: vec![], context }
    }

    /// Add a step at the end of the pipeline.
    pub fn add_step(&mut self, step: Box<dyn SetupStep>) {
        if step.help().is_empty() {
            // Enforce: every step must have help text.
            eprintln!(
                "[SetupFlow] WARNING: step '{}' has empty help text — user has no explanation",
                step.id()
            );
        }
        self.steps.push(step);
    }

    /// Fluent API: add a step and return self.
    pub fn with_step(mut self, step: Box<dyn SetupStep>) -> Self {
        self.add_step(step);
        self
    }

    // ── Queries ───────────────────────────────────────────────────────────────

    /// Returns all steps that match the given trigger.
    pub fn steps_for_trigger(&self, trigger: &SetupTrigger) -> Vec<&dyn SetupStep> {
        self.steps.iter()
            .filter(|s| s.matches_trigger(trigger))
            .map(|s| s.as_ref())
            .collect()
    }

    /// Returns all required steps that are not yet done.
    ///
    /// These are the steps blocking the "Start" button in the Manager UI.
    pub fn pending_required_steps(&self) -> Vec<&dyn SetupStep> {
        self.steps.iter()
            .filter(|s| s.is_required() && !s.is_done(&self.context))
            .map(|s| s.as_ref())
            .collect()
    }

    /// Returns all steps (required + optional) that are not yet done.
    pub fn pending_steps(&self) -> Vec<&dyn SetupStep> {
        self.steps.iter()
            .filter(|s| !s.is_done(&self.context))
            .map(|s| s.as_ref())
            .collect()
    }

    /// Returns true when all required steps are done.
    ///
    /// This is the gate for the Manager's "Start" button — if false,
    /// the button shows "Setup required" instead.
    pub fn is_ready_to_start(&self) -> bool {
        self.pending_required_steps().is_empty()
    }

    /// Progress as a fraction (completed required steps / total required steps).
    ///
    /// Returns `(done, total)`. Use for a progress bar in the wizard.
    pub fn progress(&self) -> (usize, usize) {
        let total = self.steps.iter().filter(|s| s.is_required()).count();
        let done  = self.steps.iter()
            .filter(|s| s.is_required() && s.is_done(&self.context))
            .count();
        (done, total)
    }

    // ── Execution ─────────────────────────────────────────────────────────────

    /// Execute all steps matching the given trigger, in order.
    ///
    /// Steps that are already done are skipped (idempotent).
    /// After each step, the context is persisted to disk.
    /// Execution continues even if a step fails (all failures are collected).
    ///
    /// Returns one `StepExecution` per matching step.
    pub fn execute_trigger(&mut self, trigger: SetupTrigger) -> Vec<StepExecution> {
        // Collect matching step IDs first to avoid borrow issues.
        let matching_ids: Vec<(String, String)> = self.steps.iter()
            .filter(|s| s.matches_trigger(&trigger))
            .map(|s| (s.id().to_string(), s.title().to_string()))
            .collect();

        let mut executions = Vec::new();

        for (step_id, step_title) in matching_ids {
            // Find the step.
            let step = self.steps.iter()
                .find(|s| s.id() == step_id)
                .expect("step must exist — we collected its ID just above");

            // Skip if already done.
            if step.is_done(&self.context) {
                executions.push(StepExecution {
                    step_id:    step_id.clone(),
                    step_title: step_title.clone(),
                    result:     Ok(StepOutput::empty()),
                    skipped:    true,
                });
                continue;
            }

            // Execute.
            let result = step.execute(&mut self.context);

            // Update context state.
            match &result {
                Ok(output) => {
                    // Write output values back into context config.
                    for (k, v) in &output.config_values {
                        self.context.set_config_value(k, v);
                    }
                    self.context.mark_done(&step_id, output.clone());
                }
                Err(e) => {
                    self.context.mark_failed(&step_id, e.to_string());
                }
            }

            // Persist after each step.
            let _ = self.context.save();

            executions.push(StepExecution {
                step_id,
                step_title,
                result,
                skipped: false,
            });
        }

        executions
    }

    /// Run a single step by ID, regardless of trigger.
    ///
    /// Used when the user clicks "Retry" on a failed step in the wizard.
    pub fn retry_step(&mut self, step_id: &str) -> Option<StepExecution> {
        let step = self.steps.iter().find(|s| s.id() == step_id)?;
        let title = step.title().to_string();

        // Reset the state so is_done() returns false.
        self.context.reset_step(step_id);

        let result = step.execute(&mut self.context);

        match &result {
            Ok(output) => {
                for (k, v) in &output.config_values {
                    self.context.set_config_value(k, v);
                }
                self.context.mark_done(step_id, output.clone());
            }
            Err(e) => {
                self.context.mark_failed(step_id, e.to_string());
            }
        }

        let _ = self.context.save();

        Some(StepExecution {
            step_id:    step_id.to_string(),
            step_title: title,
            result,
            skipped:    false,
        })
    }

    // ── Context persistence ───────────────────────────────────────────────────

    /// Reload the context from disk (e.g. after an external change).
    pub fn reload_context(&mut self) {
        if let Some(dir) = &self.context.config_dir.clone() {
            let pkg_id = self.context.package_id.clone();
            self.context = SetupContext::load(&pkg_id, Path::new(dir));
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::setup_step::{InputField, InputStep, SetupTrigger};
    use crate::manageable::ConfigFieldKind;

    fn make_input_step(id: &str, trigger: SetupTrigger, field_key: &str) -> Box<dyn SetupStep> {
        Box::new(
            InputStep::new(id, "Test step", "Help text.", vec![trigger])
                .with_field(InputField::new(
                    field_key, "Field", "Field help.", ConfigFieldKind::Text,
                ).with_default("default_val")),
        )
    }

    #[test]
    fn flow_is_not_ready_with_pending_required_steps() {
        let ctx = SetupContext::new("pkg");
        let mut flow = SetupFlow::new(ctx);
        flow.add_step(make_input_step("s1", SetupTrigger::FirstInstall, "domain"));

        assert!(!flow.is_ready_to_start());
    }

    #[test]
    fn flow_executes_trigger_and_applies_defaults() {
        let ctx = SetupContext::new("pkg");
        let mut flow = SetupFlow::new(ctx);
        flow.add_step(make_input_step("s1", SetupTrigger::FirstInstall, "domain"));

        let results = flow.execute_trigger(SetupTrigger::FirstInstall);
        assert_eq!(results.len(), 1);
        assert!(results[0].succeeded());
        assert_eq!(flow.context.config_value("domain"), Some("default_val"));
    }

    #[test]
    fn flow_skips_done_steps() {
        let ctx = SetupContext::new("pkg");
        let mut flow = SetupFlow::new(ctx);
        flow.add_step(make_input_step("s1", SetupTrigger::FirstInstall, "domain"));

        // Pre-set the field so the step is already done.
        flow.context.set_config_value("domain", "auth.example.com");

        let results = flow.execute_trigger(SetupTrigger::FirstInstall);
        assert_eq!(results.len(), 1);
        assert!(results[0].skipped);
    }

    #[test]
    fn flow_progress() {
        let ctx = SetupContext::new("pkg");
        let mut flow = SetupFlow::new(ctx);
        flow.add_step(make_input_step("s1", SetupTrigger::FirstInstall, "domain"));
        flow.add_step(make_input_step("s2", SetupTrigger::OnStart, "port"));

        let (done, total) = flow.progress();
        assert_eq!(done, 0);
        assert_eq!(total, 2);

        flow.context.set_config_value("domain", "x");
        let (done, _) = flow.progress();
        assert_eq!(done, 1);
    }

    #[test]
    fn context_service_role_query() {
        let mut ctx = SetupContext::new("forgejo");
        ctx.available_services.push(ServiceRef::new("kanidm", vec!["iam".into(), "iam.oidc-provider".into()]));
        ctx.available_services.push(ServiceRef::new("stalwart", vec!["smtp".into(), "smtp.sender".into()]));

        let iam = ctx.services_with_role("iam");
        assert_eq!(iam.len(), 1);
        assert_eq!(iam[0].id, "kanidm");

        let smtp = ctx.services_with_role("smtp");
        assert_eq!(smtp.len(), 1);
        assert_eq!(smtp[0].id, "stalwart");
    }
}
