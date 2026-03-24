// setup_step.rs — Setup steps: the atomic units of package configuration.
//
// A SetupStep represents one discrete operation in the setup lifecycle of a
// package — collecting user input, writing a config file, running a command,
// waiting for a service, or displaying generated output.
//
// Design principle: each step carries its own meaning (title, help), its own
// trigger conditions, and its own execution logic. The SetupFlow orchestrates
// the steps; the steps themselves are self-describing objects.
//
// Patterns used:
//   Strategy      — each concrete step type encapsulates its own behavior
//   Template Method — SetupStep provides default implementations (is_done, can_skip)
//   Self-description — every step MUST carry non-empty help text (enforced)

use std::collections::HashMap;

use fs_error::FsError;

use crate::manageable::{ConfigField, ConfigFieldKind, ConfigValue};

// ── SetupTrigger ──────────────────────────────────────────────────────────────

/// When a setup step should execute.
///
/// A step can declare multiple triggers — it runs whenever ANY of them fires.
/// Triggers are the "when". The step itself is the "what".
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum SetupTrigger {
    /// Run exactly once, on first install. Never again (unless reset).
    FirstInstall,

    /// Run every time the user saves a configuration change.
    ///
    /// Used for steps that must apply config to disk (e.g. rewriting
    /// `server.toml` after domain or port change).
    OnConfigSave,

    /// Run before the service starts.
    ///
    /// Used for steps that must be complete before the binary/container
    /// can start (e.g. TLS cert must exist, database must be initialized).
    OnStart,

    /// Run when another installed package first provides the given role.
    ///
    /// Example: when Stalwart (smtp.sender) is installed, Kanidm's
    /// email-verification step becomes relevant.
    OnDependencyInstalled { role: String },

    /// Run when a package providing the given role is removed.
    ///
    /// Allows packages to clean up connections or revert to a fallback.
    OnDependencyRemoved { role: String },
}

impl SetupTrigger {
    pub fn label(&self) -> &str {
        match self {
            Self::FirstInstall => "First install",
            Self::OnConfigSave => "On config save",
            Self::OnStart => "On start",
            Self::OnDependencyInstalled { .. } => "On dependency installed",
            Self::OnDependencyRemoved { .. } => "On dependency removed",
        }
    }

    /// Returns true when this trigger matches a `OnDependencyInstalled` event
    /// for the given role.
    pub fn matches_dep_installed(&self, role: &str) -> bool {
        matches!(self, Self::OnDependencyInstalled { role: r } if r == role)
    }

    /// Returns true when this trigger matches a `OnDependencyRemoved` event
    /// for the given role.
    pub fn matches_dep_removed(&self, role: &str) -> bool {
        matches!(self, Self::OnDependencyRemoved { role: r } if r == role)
    }
}

// ── StepOutput ────────────────────────────────────────────────────────────────

/// Output produced by a successfully executed step.
///
/// Steps that generate values (passwords, tokens, URLs) return them here.
/// The SetupFlow writes them back into the SetupContext so later steps and
/// the package's config fields can consume them.
#[derive(Debug, Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StepOutput {
    /// Key-value pairs to merge into the context config after execution.
    ///
    /// Example: `{ "admin_password": "g3n3rat3d!" }` from a CommandStep
    /// that ran `kanidm recover-account admin`.
    #[serde(default)]
    pub config_values: HashMap<String, String>,

    /// Human-readable message shown in the wizard after this step completes.
    ///
    /// Use for generated credentials or important one-time information.
    /// Keep this empty for routine steps.
    #[serde(default)]
    pub message: Option<String>,

    /// Whether to display this output prominently (large banner in UI).
    ///
    /// Set to `true` for generated passwords or one-time tokens that the
    /// user MUST see and copy before continuing.
    #[serde(default)]
    pub highlight: bool,
}

impl StepOutput {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn with_value(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.config_values.insert(key.into(), value.into());
        self
    }

    pub fn with_message(mut self, msg: impl Into<String>) -> Self {
        self.message = Some(msg.into());
        self
    }

    pub fn highlighted(mut self) -> Self {
        self.highlight = true;
        self
    }
}

// ── StepState ─────────────────────────────────────────────────────────────────

/// Persisted state of one setup step.
///
/// Saved in `SetupContext::step_states` so the wizard can resume after
/// the user closes it mid-way. The state is written to `.setup-state.toml`
/// next to the package's config file.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case", tag = "state")]
pub enum StepState {
    /// Step has not run yet (or was reset).
    Pending,

    /// Step completed successfully.
    Done {
        /// Output produced by the step (may be empty).
        output: StepOutput,
    },

    /// Step failed — the error message is shown in the wizard.
    Failed { error: String },

    /// Step was skipped by the user (only allowed if `can_skip()` is true).
    Skipped,
}

impl StepState {
    pub fn is_done(&self) -> bool {
        matches!(self, Self::Done { .. })
    }
    pub fn is_failed(&self) -> bool {
        matches!(self, Self::Failed { .. })
    }
    pub fn is_pending(&self) -> bool {
        matches!(self, Self::Pending)
    }

    pub fn output(&self) -> Option<&StepOutput> {
        if let Self::Done { output } = self {
            Some(output)
        } else {
            None
        }
    }
}

// ── SetupStep (trait) ─────────────────────────────────────────────────────────

/// One atomic operation in the setup lifecycle of a package.
///
/// # Responsibilities of the implementor
///
/// - Declare WHEN to run (`triggers()`)
/// - Describe WHAT this does (`title()`, `help()`)
/// - Know WHEN it is already done (`is_done()`)
/// - Do the work (`execute()`)
///
/// # Help is mandatory
///
/// Every step MUST provide a non-empty `help()` string. This text appears in
/// the wizard's right-side help panel. A step without help is incomplete.
/// The `SetupFlow` will log a warning for any step with empty help.
pub trait SetupStep: Send + Sync {
    // ── Identity ─────────────────────────────────────────────────────────────

    /// Stable identifier for this step. Used as key in `SetupContext::step_states`.
    ///
    /// Must be unique within the package's setup flow.
    fn id(&self) -> &str;

    /// Short title shown in the wizard progress list (e.g. "Configure domain").
    fn title(&self) -> &str;

    /// Full help text shown in the wizard's right-side help panel.
    ///
    /// Explain what this step does, why it matters, and what the user
    /// should enter or expect. MANDATORY — must not be empty.
    fn help(&self) -> &str;

    // ── Trigger conditions ────────────────────────────────────────────────────

    /// Returns the list of trigger conditions that cause this step to run.
    fn triggers(&self) -> &[SetupTrigger];

    /// Returns `true` when this step should run for the given trigger.
    fn matches_trigger(&self, trigger: &SetupTrigger) -> bool {
        self.triggers().contains(trigger)
    }

    // ── State ─────────────────────────────────────────────────────────────────

    /// Whether this step must complete before the service can start.
    ///
    /// Required steps block the "Start" button in the Manager UI.
    /// Optional steps are informational or for enhanced functionality.
    fn is_required(&self) -> bool {
        true
    }

    /// Whether the user can skip this step.
    ///
    /// Non-technical users should almost always use the default `false`.
    /// Only offer skipping for genuinely optional enhancements.
    fn can_skip(&self) -> bool {
        false
    }

    /// Returns `true` if this step does not need to run again.
    ///
    /// The default implementation checks `SetupContext::step_states`.
    /// Concrete steps may override this to check external state instead
    /// (e.g. a WriteConfigStep could check whether the config file exists).
    fn is_done(&self, ctx: &SetupContext) -> bool {
        ctx.step_state(self.id()).is_done()
    }

    // ── Execution ─────────────────────────────────────────────────────────────

    /// Execute this step.
    ///
    /// The step reads what it needs from `ctx` (config values, available
    /// services) and writes results back via `ctx.set_config_value()` or
    /// returns them in `StepOutput`.
    ///
    /// On success: return the output (may be empty).
    /// On failure: return an `FsError` — the flow records it as `StepState::Failed`.
    fn execute(&self, ctx: &mut SetupContext) -> Result<StepOutput, FsError>;
}

// ── SetupContext (forward-declared here for use in SetupStep) ─────────────────
// Full definition is in setup_flow.rs. We need the type here for the trait.

use crate::setup_flow::SetupContext;

// ── InputStep ─────────────────────────────────────────────────────────────────

/// A setup step that collects user input via a form.
///
/// The wizard renders all `fields` as an input form. The user fills in values,
/// which are written into the `SetupContext`. Fields with `auto_generate = true`
/// are pre-filled with a random value (shown to the user, editable).
///
/// Default values are shown pre-filled so non-technical users can just click OK.
pub struct InputStep {
    pub id: String,
    pub title: String,
    /// Right-sidebar help text explaining the purpose of these fields.
    pub help: String,
    pub triggers: Vec<SetupTrigger>,
    /// The form fields shown to the user.
    pub fields: Vec<InputField>,
    /// Whether all fields must be filled before continuing.
    pub required: bool,
}

/// One field in an [`InputStep`].
pub struct InputField {
    /// Key written into `SetupContext::config` when saved.
    pub key: String,
    pub label: String,
    /// Per-field help shown as tooltip/inline hint (mandatory).
    pub field_help: String,
    pub kind: ConfigFieldKind,
    /// Default value — shown pre-filled in the wizard.
    pub default: Option<String>,
    /// Auto-generate a random secret (overrides `default`).
    pub auto_generate: bool,
    pub required: bool,
}

impl InputField {
    pub fn new(
        key: impl Into<String>,
        label: impl Into<String>,
        field_help: impl Into<String>,
        kind: ConfigFieldKind,
    ) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            field_help: field_help.into(),
            kind,
            default: None,
            auto_generate: false,
            required: true,
        }
    }

    pub fn with_default(mut self, v: impl Into<String>) -> Self {
        self.default = Some(v.into());
        self
    }

    pub fn auto_generated(mut self) -> Self {
        self.auto_generate = true;
        self
    }

    pub fn optional(mut self) -> Self {
        self.required = false;
        self
    }

    /// Convert to a `ConfigField` for rendering in the wizard UI.
    pub fn to_config_field(&self, ctx: &SetupContext) -> ConfigField {
        let value = ctx
            .config_value(&self.key)
            .map(|v| ConfigValue::Text(v.to_string()))
            .or_else(|| {
                self.default
                    .as_deref()
                    .map(|d| ConfigValue::Text(d.to_string()))
            })
            .unwrap_or(ConfigValue::Empty);

        ConfigField::new(&self.key, &self.label, &self.field_help, self.kind.clone())
            .with_value(value)
            .required_if(self.required)
    }
}

impl InputStep {
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        help: impl Into<String>,
        triggers: Vec<SetupTrigger>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            help: help.into(),
            triggers,
            fields: vec![],
            required: true,
        }
    }

    pub fn with_field(mut self, field: InputField) -> Self {
        self.fields.push(field);
        self
    }

    pub fn optional(mut self) -> Self {
        self.required = false;
        self
    }
}

impl SetupStep for InputStep {
    fn id(&self) -> &str {
        &self.id
    }
    fn title(&self) -> &str {
        &self.title
    }
    fn help(&self) -> &str {
        &self.help
    }

    fn triggers(&self) -> &[SetupTrigger] {
        &self.triggers
    }
    fn is_required(&self) -> bool {
        self.required
    }

    fn is_done(&self, ctx: &SetupContext) -> bool {
        // Done when all required fields are present in the context.
        self.fields
            .iter()
            .filter(|f| f.required)
            .all(|f| ctx.config_value(&f.key).is_some())
    }

    fn execute(&self, ctx: &mut SetupContext) -> Result<StepOutput, FsError> {
        // Apply defaults and auto-generated values for fields not yet set.
        let mut output = StepOutput::empty();

        for field in &self.fields {
            if ctx.config_value(&field.key).is_some() {
                continue; // Already set by the user
            }
            if field.auto_generate {
                let secret = generate_secret();
                ctx.set_config_value(&field.key, &secret);
                output = output
                    .with_value(&field.key, &secret)
                    .with_message(format!("Generated {}: {}", field.label, secret))
                    .highlighted();
            } else if let Some(default) = &field.default {
                ctx.set_config_value(&field.key, default);
                output = output.with_value(&field.key, default);
            }
        }

        Ok(output)
    }
}

// ── CommandStep ───────────────────────────────────────────────────────────────

/// A setup step that runs a shell command.
///
/// Used for operations like initializing a database, recovering an admin
/// account, or running a migration. The command runs in the package's
/// installation directory.
///
/// If `capture_key` is set, the first line of stdout is written into the
/// context under that key (e.g. to capture a generated admin password).
pub struct CommandStep {
    pub id: String,
    pub title: String,
    pub help: String,
    pub triggers: Vec<SetupTrigger>,
    /// Command binary (e.g. `"kanidm"`).
    pub command: String,
    /// Arguments (e.g. `["server", "recover-account", "admin"]`).
    pub args: Vec<String>,
    /// If set, capture stdout and write it to the context under this key.
    pub capture_key: Option<String>,
    pub required: bool,
}

impl CommandStep {
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        help: impl Into<String>,
        triggers: Vec<SetupTrigger>,
        command: impl Into<String>,
        args: Vec<impl Into<String>>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            help: help.into(),
            triggers,
            command: command.into(),
            args: args.into_iter().map(Into::into).collect(),
            capture_key: None,
            required: true,
        }
    }

    pub fn capturing(mut self, key: impl Into<String>) -> Self {
        self.capture_key = Some(key.into());
        self
    }

    pub fn optional(mut self) -> Self {
        self.required = false;
        self
    }
}

impl SetupStep for CommandStep {
    fn id(&self) -> &str {
        &self.id
    }
    fn title(&self) -> &str {
        &self.title
    }
    fn help(&self) -> &str {
        &self.help
    }
    fn triggers(&self) -> &[SetupTrigger] {
        &self.triggers
    }
    fn is_required(&self) -> bool {
        self.required
    }

    fn execute(&self, ctx: &mut SetupContext) -> Result<StepOutput, FsError> {
        use std::process::Command;

        let child = Command::new(&self.command)
            .args(&self.args)
            .current_dir(ctx.install_path.as_deref().unwrap_or("."))
            .output()
            .map_err(FsError::Io)?;

        if !child.status.success() {
            let stderr = String::from_utf8_lossy(&child.stderr).into_owned();
            return Err(FsError::Internal(format!(
                "Command `{} {}` failed: {}",
                self.command,
                self.args.join(" "),
                stderr
            )));
        }

        let mut output = StepOutput::empty();

        if let Some(key) = &self.capture_key {
            let stdout = String::from_utf8_lossy(&child.stdout).into_owned();
            let first_line = stdout.lines().next().unwrap_or("").trim().to_string();
            ctx.set_config_value(key, &first_line);
            output = output
                .with_value(key, &first_line)
                .with_message(format!("{}: {}", self.title, first_line))
                .highlighted();
        }

        Ok(output)
    }
}

// ── WriteConfigStep ───────────────────────────────────────────────────────────

/// A setup step that writes the package's config file from a Tera template.
///
/// Runs on `OnConfigSave` and `OnStart` to ensure the config on disk always
/// matches what the user has configured. This step is idempotent: writing
/// the same config twice is safe.
pub struct WriteConfigStep {
    pub id: String,
    pub title: String,
    pub help: String,
    pub triggers: Vec<SetupTrigger>,
    /// Tera template string. Variables: all keys in `SetupContext::config`.
    pub template: String,
    /// Destination path for the config file.
    pub config_path: String,
}

impl WriteConfigStep {
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        help: impl Into<String>,
        triggers: Vec<SetupTrigger>,
        template: impl Into<String>,
        config_path: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            help: help.into(),
            triggers,
            template: template.into(),
            config_path: config_path.into(),
        }
    }
}

impl SetupStep for WriteConfigStep {
    fn id(&self) -> &str {
        &self.id
    }
    fn title(&self) -> &str {
        &self.title
    }
    fn help(&self) -> &str {
        &self.help
    }
    fn triggers(&self) -> &[SetupTrigger] {
        &self.triggers
    }

    fn is_done(&self, ctx: &SetupContext) -> bool {
        // Done when the config file exists AND the step state is Done.
        std::path::Path::new(&self.config_path).exists() && ctx.step_state(self.id()).is_done()
    }

    fn execute(&self, ctx: &mut SetupContext) -> Result<StepOutput, FsError> {
        let mut tera = tera::Tera::default();
        tera.add_raw_template("config", &self.template)
            .map_err(|e| FsError::Internal(format!("Template error: {e}")))?;

        let mut vars = tera::Context::new();
        for (k, v) in &ctx.config {
            vars.insert(k, v);
        }

        let rendered = tera
            .render("config", &vars)
            .map_err(|e| FsError::Internal(format!("Template render error: {e}")))?;

        if let Some(parent) = std::path::Path::new(&self.config_path).parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| FsError::Internal(format!("Failed to create config dir: {e}")))?;
        }

        std::fs::write(&self.config_path, rendered)
            .map_err(|e| FsError::Internal(format!("Failed to write config: {e}")))?;

        Ok(StepOutput::empty())
    }
}

// ── WaitForServiceStep ────────────────────────────────────────────────────────

/// A setup step that waits for a TCP port to become reachable.
///
/// Used after starting a service to confirm it is ready before proceeding
/// with configuration steps that require the API to be available.
pub struct WaitForServiceStep {
    pub id: String,
    pub title: String,
    pub help: String,
    pub triggers: Vec<SetupTrigger>,
    pub host: String,
    pub port: u16,
    pub timeout_secs: u64,
}

impl WaitForServiceStep {
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        help: impl Into<String>,
        triggers: Vec<SetupTrigger>,
        host: impl Into<String>,
        port: u16,
        timeout_secs: u64,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            help: help.into(),
            triggers,
            host: host.into(),
            port,
            timeout_secs,
        }
    }
}

impl SetupStep for WaitForServiceStep {
    fn id(&self) -> &str {
        &self.id
    }
    fn title(&self) -> &str {
        &self.title
    }
    fn help(&self) -> &str {
        &self.help
    }
    fn triggers(&self) -> &[SetupTrigger] {
        &self.triggers
    }

    fn execute(&self, _ctx: &mut SetupContext) -> Result<StepOutput, FsError> {
        use std::net::TcpStream;
        use std::time::{Duration, Instant};

        let deadline = Instant::now() + Duration::from_secs(self.timeout_secs);
        let addr = format!("{}:{}", self.host, self.port);

        while Instant::now() < deadline {
            if TcpStream::connect(&addr).is_ok() {
                return Ok(StepOutput::empty());
            }
            std::thread::sleep(Duration::from_millis(500));
        }

        Err(FsError::Internal(format!(
            "Service not reachable at {} after {}s",
            addr, self.timeout_secs
        )))
    }
}

// ── DisplayOutputStep ─────────────────────────────────────────────────────────

/// A step that displays output from a previous step — no execution, only display.
///
/// Use this after a `CommandStep` that generates credentials, to show the
/// generated value to the user before they continue.
///
/// This step is always `can_skip() == false` because the user must see the
/// output before proceeding (e.g. an admin password they must save).
pub struct DisplayOutputStep {
    pub id: String,
    pub title: String,
    pub help: String,
    pub triggers: Vec<SetupTrigger>,
    /// ID of the step whose output to display.
    pub source_step_id: String,
}

impl DisplayOutputStep {
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        help: impl Into<String>,
        triggers: Vec<SetupTrigger>,
        source_step_id: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            help: help.into(),
            triggers,
            source_step_id: source_step_id.into(),
        }
    }
}

impl SetupStep for DisplayOutputStep {
    fn id(&self) -> &str {
        &self.id
    }
    fn title(&self) -> &str {
        &self.title
    }
    fn help(&self) -> &str {
        &self.help
    }
    fn triggers(&self) -> &[SetupTrigger] {
        &self.triggers
    }
    fn can_skip(&self) -> bool {
        false
    }

    fn is_done(&self, ctx: &SetupContext) -> bool {
        // Done once the source step is done.
        ctx.step_state(&self.source_step_id).is_done()
    }

    fn execute(&self, ctx: &mut SetupContext) -> Result<StepOutput, FsError> {
        // The source step's output is already in the context — nothing to do.
        // The UI renders this step by reading ctx.step_state(source_step_id).output().
        let state = ctx.step_state(&self.source_step_id);
        if let StepState::Done { output } = state {
            Ok(output.clone())
        } else {
            Err(FsError::Internal(format!(
                "DisplayOutputStep: source step '{}' has not completed yet",
                self.source_step_id
            )))
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Generate a random 24-character alphanumeric secret.
pub fn generate_secret() -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::time::SystemTime;

    // A fast, portable secret generator that avoids adding a rand dependency.
    // For production use, prefer `getrandom` or `rand`.
    let mut h = DefaultHasher::new();
    SystemTime::now().hash(&mut h);
    std::process::id().hash(&mut h);
    let seed = h.finish();

    // XorShift64 pseudo-random expansion
    let mut x = seed ^ 0xdeadbeefcafe1234;
    let charset = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    (0..32)
        .map(|_| {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            charset[(x as usize) % charset.len()] as char
        })
        .collect()
}

// ── ConfigField extension ─────────────────────────────────────────────────────

// We need `required_if` on ConfigField which does not exist yet.
// This extension trait adds it without touching the original.
pub trait ConfigFieldExt {
    fn required_if(self, required: bool) -> Self;
}

impl ConfigFieldExt for ConfigField {
    fn required_if(mut self, required: bool) -> Self {
        if required {
            self = self.required();
        }
        self
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::setup_flow::SetupContext;

    fn make_ctx() -> SetupContext {
        SetupContext::new("test-pkg")
    }

    #[test]
    fn input_step_done_when_all_required_fields_set() {
        let step = InputStep::new(
            "collect_domain",
            "Configure domain",
            "Enter the domain for this service.",
            vec![SetupTrigger::FirstInstall],
        )
        .with_field(InputField::new(
            "domain",
            "Domain",
            "The public hostname.",
            ConfigFieldKind::Text,
        ));

        let mut ctx = make_ctx();
        assert!(!step.is_done(&ctx));

        ctx.set_config_value("domain", "auth.example.com");
        assert!(step.is_done(&ctx));
    }

    #[test]
    fn trigger_matches() {
        let t = SetupTrigger::OnDependencyInstalled { role: "iam".into() };
        assert!(t.matches_dep_installed("iam"));
        assert!(!t.matches_dep_installed("smtp"));
    }

    #[test]
    fn step_output_builder() {
        let o = StepOutput::empty()
            .with_value("key", "val")
            .with_message("hello")
            .highlighted();

        assert!(o.config_values.contains_key("key"));
        assert_eq!(o.message.as_deref(), Some("hello"));
        assert!(o.highlight);
    }

    #[test]
    fn secret_generation_is_nonempty() {
        let s = generate_secret();
        assert_eq!(s.len(), 32);
        assert!(s.chars().all(|c| c.is_ascii_alphanumeric()));
    }
}
