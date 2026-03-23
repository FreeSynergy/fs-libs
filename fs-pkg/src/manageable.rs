// manageable.rs — Manageable trait: packages describe themselves to the Manager.
//
// Design principle: The Manager offers the HOW. The Package owns the WHAT.
//
// The Manager is a standardized UI shell — it always has the same tabs (Info,
// Config, Builder), the same icons, the same sidebar structure. But what fills
// those tabs is determined entirely by the package that implements Manageable.
//
// The Manager NEVER knows what a specific package needs to configure.
// It asks the package: "give me your config fields" — and renders them.
//
// Every package type implements Manageable:
//   - App       — single instance, no sub-instances
//   - Container — multiple instances possible
//   - Bot       — multiple instances possible
//   - Bridge    — multiple instances possible
//   - Theme     — no runtime, only config/build
//   - Widget    — similar to Theme
//   - Language  — language pack, minimal config
//
// Patterns: Strategy (config_fields), Template Method (Manageable trait),
//           Observer (apply_config + change events),
//           Chain of Responsibility (setup_flow — ordered steps with triggers)

use fs_error::FsError;

use crate::manifest::{PackageMeta, PackageType};
use crate::setup_contributor::SetupContributor;
use crate::setup_flow::{SetupContext, SetupFlow};

// ── ConfigValue ───────────────────────────────────────────────────────────────

/// Typed value of a config field.
///
/// The Manager passes this to [`Manageable::apply_config`] when the user edits
/// a field. The package validates and persists the new value.
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigValue {
    Text(String),
    Bool(bool),
    Number(f64),
    Port(u16),
    /// Field cleared / not yet set.
    Empty,
}

impl ConfigValue {
    pub fn as_text(&self) -> Option<&str> {
        if let Self::Text(s) = self { Some(s) } else { None }
    }

    pub fn as_bool(&self) -> Option<bool> {
        if let Self::Bool(b) = self { Some(*b) } else { None }
    }

    pub fn as_number(&self) -> Option<f64> {
        if let Self::Number(n) = self { Some(*n) } else { None }
    }

    pub fn as_port(&self) -> Option<u16> {
        if let Self::Port(p) = self { Some(*p) } else { None }
    }

    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }
}

// ── ConfigFieldKind ───────────────────────────────────────────────────────────

/// Widget type for a config field — the Manager selects the right input.
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigFieldKind {
    /// Single-line text input.
    Text,
    /// Password input (value masked).
    Password,
    /// Numeric input with optional min/max range.
    Number { min: Option<f64>, max: Option<f64> },
    /// Boolean toggle switch.
    Bool,
    /// Dropdown with a fixed set of choices.
    Select { options: Vec<SelectOption> },
    /// Port number (validated as 1–65535).
    Port,
    /// File system path with optional picker.
    Path,
    /// Multi-line text area.
    Textarea,
}

/// One option in a [`ConfigFieldKind::Select`] field.
#[derive(Debug, Clone, PartialEq)]
pub struct SelectOption {
    pub value: String,
    pub label: String,
}

impl SelectOption {
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self { value: value.into(), label: label.into() }
    }
}

// ── ConfigField ───────────────────────────────────────────────────────────────

/// One configurable field a package exposes for the Manager's Config tab.
///
/// The `help` text is MANDATORY — every setting must have a description so
/// users know what they are changing. The Manager will display a warning for
/// any field whose `help` is empty.
#[derive(Debug, Clone, PartialEq)]
pub struct ConfigField {
    /// Machine-readable key (matches the TOML key being edited).
    pub key: String,

    /// Human-readable label displayed next to the input.
    pub label: String,

    /// Explanation of what this setting does — MANDATORY.
    ///
    /// Shown in the Manager's inline help and in the Settings Manager.
    /// An empty `help` string indicates a documentation gap.
    pub help: String,

    /// Input widget type.
    pub kind: ConfigFieldKind,

    /// Current value.
    pub value: ConfigValue,

    /// Whether this field must be filled before the package can start.
    pub required: bool,

    /// Whether a restart is required for the change to take effect.
    pub needs_restart: bool,
}

impl ConfigField {
    /// Build a config field. `help` is required by convention.
    pub fn new(
        key:   impl Into<String>,
        label: impl Into<String>,
        help:  impl Into<String>,
        kind:  ConfigFieldKind,
    ) -> Self {
        Self {
            key:           key.into(),
            label:         label.into(),
            help:          help.into(),
            kind,
            value:         ConfigValue::Empty,
            required:      false,
            needs_restart: false,
        }
    }

    /// Set an initial value.
    pub fn with_value(mut self, v: ConfigValue) -> Self {
        self.value = v;
        self
    }

    /// Mark the field as required.
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// Mark that changes require a restart.
    pub fn needs_restart(mut self) -> Self {
        self.needs_restart = true;
        self
    }

    /// Returns `true` when the help text is present (non-empty).
    pub fn has_help(&self) -> bool {
        !self.help.is_empty()
    }
}

// ── HealthCheck ───────────────────────────────────────────────────────────────

/// Result of one individual self-check.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheck {
    /// Short check name (e.g. `"config readable"`, `"port reachable"`).
    pub name: String,
    /// Whether the check passed.
    pub passed: bool,
    /// Optional human-readable detail (error text or success note).
    pub message: Option<String>,
}

impl HealthCheck {
    pub fn ok(name: impl Into<String>) -> Self {
        Self { name: name.into(), passed: true, message: None }
    }

    pub fn ok_with(name: impl Into<String>, msg: impl Into<String>) -> Self {
        Self { name: name.into(), passed: true, message: Some(msg.into()) }
    }

    pub fn fail(name: impl Into<String>, msg: impl Into<String>) -> Self {
        Self { name: name.into(), passed: false, message: Some(msg.into()) }
    }
}

/// Aggregated health result returned by [`Manageable::check_health`].
#[derive(Debug, Clone, PartialEq)]
pub struct PackageHealth {
    pub checks: Vec<HealthCheck>,
}

impl PackageHealth {
    pub fn new(checks: Vec<HealthCheck>) -> Self { Self { checks } }

    /// Returns `true` when every individual check passed.
    pub fn is_healthy(&self) -> bool {
        self.checks.iter().all(|c| c.passed)
    }

    /// Iterator over failed checks only.
    pub fn failures(&self) -> impl Iterator<Item = &HealthCheck> {
        self.checks.iter().filter(|c| !c.passed)
    }

    /// A single "not installed" health result.
    pub fn not_installed() -> Self {
        Self { checks: vec![HealthCheck::fail("installed", "Package is not installed")] }
    }
}

// ── RunStatus ─────────────────────────────────────────────────────────────────

/// Current runtime status of a package or one of its instances.
///
/// Packages carry their own status — they know whether they are running or not.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunStatus {
    Running,
    Stopped,
    Starting,
    Stopping,
    Error(String),
    /// Package is not installed — cannot be started.
    NotInstalled,
    /// Package is installed but setup has not been completed yet.
    ///
    /// Required setup steps are still pending (e.g. first-time configuration
    /// wizard not finished). The Manager shows "Setup required" and prevents
    /// the "Start" button from activating.
    SetupRequired,
}

impl RunStatus {
    pub fn is_running(&self)  -> bool { matches!(self, Self::Running) }
    pub fn is_stopped(&self)  -> bool { matches!(self, Self::Stopped) }
    pub fn is_error(&self)    -> bool { matches!(self, Self::Error(_)) }
    pub fn is_setup_required(&self) -> bool { matches!(self, Self::SetupRequired) }
    pub fn in_transition(&self) -> bool {
        matches!(self, Self::Starting | Self::Stopping)
    }

    /// Short human-readable label for display.
    pub fn label(&self) -> &str {
        match self {
            Self::Running        => "Running",
            Self::Stopped        => "Stopped",
            Self::Starting       => "Starting",
            Self::Stopping       => "Stopping",
            Self::Error(_)       => "Error",
            Self::NotInstalled   => "Not installed",
            Self::SetupRequired  => "Setup required",
        }
    }

    /// CSS class for status badges in the Manager UI.
    pub fn css_class(&self) -> &str {
        match self {
            Self::Running                   => "fs-status--running",
            Self::Stopped                   => "fs-status--stopped",
            Self::Starting | Self::Stopping => "fs-status--transitioning",
            Self::Error(_)                  => "fs-status--error",
            Self::NotInstalled              => "fs-status--not-installed",
            Self::SetupRequired             => "fs-status--setup-required",
        }
    }
}

// ── InstanceRef ───────────────────────────────────────────────────────────────

/// Reference to a sub-instance of a multi-instance package (Bot, Container, Bridge).
///
/// The Manager sidebar shows sub-instances beneath the parent package entry,
/// like a sub-folder with a back arrow at the top to return to the package view.
#[derive(Debug, Clone, PartialEq)]
pub struct InstanceRef {
    /// Unique instance ID (UUID or slug).
    pub id: String,
    /// User-assigned display name (e.g. `"main-iam"`, `"backup"`).
    pub name: String,
    /// Current runtime status.
    pub status: RunStatus,
}

impl InstanceRef {
    pub fn new(
        id:     impl Into<String>,
        name:   impl Into<String>,
        status: RunStatus,
    ) -> Self {
        Self { id: id.into(), name: name.into(), status }
    }
}

// ── Manageable ────────────────────────────────────────────────────────────────

/// Core trait: a package describes itself to the Manager.
///
/// # The principle
///
/// The Manager is a standardized shell — it offers the **HOW** (the UI chrome:
/// tabs, sidebar, icons, buttons).  The Package owns the **WHAT** (which fields
/// to configure, what the builder needs, how to check its own health).
///
/// The Manager NEVER hardcodes knowledge about individual packages. It always
/// asks the package what it needs via this trait.
///
/// # Required methods
///
/// - [`meta()`](Self::meta) — package identity
/// - [`package_type()`](Self::package_type)
/// - [`is_installed()`](Self::is_installed)
/// - [`run_status()`](Self::run_status)
/// - [`config_fields()`](Self::config_fields) — what the Config tab shows
/// - [`apply_config()`](Self::apply_config) — handle a field change
/// - [`check_health()`](Self::check_health) — self-check
///
/// # Optional methods (have sane defaults)
///
/// - [`instances()`](Self::instances) — sub-instances for Bot/Container/Bridge
/// - [`build_fields()`](Self::build_fields) — Builder tab content
/// - [`can_start()`](Self::can_start) / [`can_stop()`](Self::can_stop)
/// - [`can_persist()`](Self::can_persist) — systemd registration
pub trait Manageable {
    // ── Identity ─────────────────────────────────────────────────────────────

    /// Package metadata: name, version, description, author, icon, category.
    fn meta(&self) -> &PackageMeta;

    /// The type of this package (App, Container, Bot, Widget, …).
    fn package_type(&self) -> PackageType;

    // ── Installation state ────────────────────────────────────────────────────

    /// Whether this package is installed on the current node.
    ///
    /// The package checks the inventory itself — not an external service.
    fn is_installed(&self) -> bool;

    // ── Runtime state ─────────────────────────────────────────────────────────

    /// Current runtime status.
    ///
    /// The package knows its own state (it was persisted in the inventory).
    fn run_status(&self) -> RunStatus;

    // ── Config tab ────────────────────────────────────────────────────────────

    /// Config fields this package exposes for the Manager's **Config** tab.
    ///
    /// The Manager renders these fields; the package defines them.
    /// Every field MUST include a non-empty `help` text.
    fn config_fields(&self) -> Vec<ConfigField>;

    /// Apply a config value change initiated by the user in the Manager.
    ///
    /// The package validates the value and updates its internal state.
    /// Return an error when validation fails — the Manager will display it.
    fn apply_config(&mut self, key: &str, value: ConfigValue) -> Result<(), FsError>;

    // ── Health ────────────────────────────────────────────────────────────────

    /// Run all self-checks and return the aggregated result.
    ///
    /// The package decides what to check: config file readable? port
    /// reachable? dependency present? The Manager shows the results in the
    /// Info tab status section.
    fn check_health(&self) -> PackageHealth;

    // ── Sub-instances (Bot, Container, Bridge) ────────────────────────────────

    /// Sub-instances for multi-instance packages.
    ///
    /// Returns an empty vec for single-instance packages (App, Theme, Widget).
    /// The Manager sidebar shows these as collapsible sub-entries.
    fn instances(&self) -> Vec<InstanceRef> { vec![] }

    // ── Builder tab ───────────────────────────────────────────────────────────

    /// Fields for the Manager's **Builder** tab.
    ///
    /// Replaces the standalone fs-builder. The package knows what it needs
    /// in order to be built/created — the Manager renders the form.
    fn build_fields(&self) -> Vec<ConfigField> { vec![] }

    // ── Actions ───────────────────────────────────────────────────────────────

    /// Whether the package can be started right now.
    ///
    /// Returns `false` when setup is not complete — the Manager shows
    /// "Setup required" instead of "Start" in that case.
    fn can_start(&self) -> bool {
        self.is_installed()
            && !self.needs_setup()
            && !matches!(self.run_status(), RunStatus::Running | RunStatus::Starting)
    }

    /// Whether the package can be stopped right now.
    fn can_stop(&self) -> bool {
        matches!(self.run_status(), RunStatus::Running | RunStatus::Starting)
    }

    /// Whether the package can be registered as a persistent systemd service.
    ///
    /// Returns `false` by default — only container/service packages override this.
    fn can_persist(&self) -> bool { false }

    // ── Setup flow ────────────────────────────────────────────────────────────

    /// The setup flow for this package.
    ///
    /// Returns an ordered pipeline of [`SetupStep`]s that the Manager
    /// executes to bring the package from installed → configured → running.
    ///
    /// # Default implementation
    ///
    /// Converts the manifest's `[setup]` fields into a single `InputStep`
    /// triggered by `FirstInstall`. Packages that need richer setup
    /// (commands, waits, cross-package steps) override this method.
    ///
    /// # Triggers
    ///
    /// - `FirstInstall` — one-time wizard, first time the package is set up.
    /// - `OnConfigSave` — re-applied whenever the user changes config.
    /// - `OnStart`      — checks before the service starts.
    ///
    /// [`SetupStep`]: crate::setup_step::SetupStep
    fn setup_flow(&self) -> SetupFlow {
        SetupFlow::new(SetupContext::new(self.meta().id.as_str()))
    }

    /// Steps this package contributes to OTHER packages' setup flows.
    ///
    /// Implement this when your package provides a role that other packages
    /// depend on, and you want to auto-configure that integration from within
    /// the other package's setup wizard.
    ///
    /// Example: Kanidm implements this to contribute OIDC client creation
    /// steps into any package that requires the `iam.oidc-provider` role.
    fn setup_contributors(&self) -> Vec<Box<dyn SetupContributor>> {
        vec![]
    }

    /// Whether setup must be completed before the package can start.
    ///
    /// The default checks whether the setup flow has pending required steps.
    /// The Manager uses this to gate the "Start" button.
    fn needs_setup(&self) -> bool {
        !self.setup_flow().is_ready_to_start()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_value_accessors() {
        assert_eq!(ConfigValue::Text("hi".into()).as_text(), Some("hi"));
        assert_eq!(ConfigValue::Bool(true).as_bool(), Some(true));
        assert_eq!(ConfigValue::Number(3.14).as_number(), Some(3.14));
        assert_eq!(ConfigValue::Port(8080).as_port(), Some(8080));
        assert!(ConfigValue::Empty.is_empty());
    }

    #[test]
    fn config_field_has_help() {
        let f = ConfigField::new("k", "Label", "Explanation", ConfigFieldKind::Text);
        assert!(f.has_help());

        let f2 = ConfigField::new("k", "Label", "", ConfigFieldKind::Text);
        assert!(!f2.has_help());
    }

    #[test]
    fn run_status_predicates() {
        assert!(RunStatus::Running.is_running());
        assert!(RunStatus::Stopped.is_stopped());
        assert!(RunStatus::Error("x".into()).is_error());
        assert!(RunStatus::Starting.in_transition());
    }

    #[test]
    fn package_health_is_healthy() {
        let h = PackageHealth::new(vec![
            HealthCheck::ok("a"),
            HealthCheck::ok("b"),
        ]);
        assert!(h.is_healthy());
        assert_eq!(h.failures().count(), 0);
    }

    #[test]
    fn package_health_failures() {
        let h = PackageHealth::new(vec![
            HealthCheck::ok("a"),
            HealthCheck::fail("b", "broken"),
        ]);
        assert!(!h.is_healthy());
        assert_eq!(h.failures().count(), 1);
    }

    #[test]
    fn run_status_css_classes() {
        assert_eq!(RunStatus::Running.css_class(),       "fs-status--running");
        assert_eq!(RunStatus::Error("x".into()).css_class(), "fs-status--error");
        assert_eq!(RunStatus::Starting.css_class(),      "fs-status--transitioning");
    }
}
