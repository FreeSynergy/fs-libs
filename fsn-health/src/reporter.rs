// Health reporting types: HealthLevel, HealthIssue, HealthStatus, HealthRules.

use serde::{Deserialize, Serialize};

// ── HealthLevel ───────────────────────────────────────────────────────────────

/// Overall health level of a resource.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthLevel {
    /// All required conditions satisfied — deployment is possible.
    #[default]
    Ok,
    /// Optional component missing — deployment works but is degraded.
    Warning,
    /// Required component missing — deployment will fail.
    Error,
}

impl HealthLevel {
    /// Single-character Unicode indicator for compact TUI display.
    pub fn indicator(self) -> &'static str {
        match self {
            HealthLevel::Ok      => "✓",
            HealthLevel::Warning => "⚠",
            HealthLevel::Error   => "✗",
        }
    }

    /// Plain-text label for accessibility / screenreaders.
    pub fn indicator_text(self) -> &'static str {
        match self {
            HealthLevel::Ok      => "ok",
            HealthLevel::Warning => "warning",
            HealthLevel::Error   => "error",
        }
    }

    /// Combined indicator: `"✓ (ok)"`.
    pub fn indicator_with_text(self) -> String {
        format!("{} ({})", self.indicator(), self.indicator_text())
    }

    /// i18n key for the level label.
    pub fn i18n_key(self) -> &'static str {
        match self {
            HealthLevel::Ok      => "health.ok",
            HealthLevel::Warning => "health.warning",
            HealthLevel::Error   => "health.error",
        }
    }

    /// `true` when the level indicates the resource is fully operational.
    pub fn is_ok(self) -> bool {
        self == HealthLevel::Ok
    }
}

// ── HealthIssue ───────────────────────────────────────────────────────────────

/// A single health issue found during validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthIssue {
    /// Severity of this issue.
    pub level: HealthLevel,
    /// i18n key for the issue message (resolved by the UI via its translator).
    pub msg_key: String,
}

impl HealthIssue {
    /// Construct an Error-level issue.
    pub fn error(msg_key: &'static str) -> Self {
        Self { level: HealthLevel::Error, msg_key: msg_key.to_string() }
    }

    /// Construct a Warning-level issue.
    pub fn warning(msg_key: &'static str) -> Self {
        Self { level: HealthLevel::Warning, msg_key: msg_key.to_string() }
    }

    /// Construct an info issue (level = Ok — informational only).
    pub fn info(msg_key: &'static str) -> Self {
        Self { level: HealthLevel::Ok, msg_key: msg_key.to_string() }
    }
}

// ── HealthStatus ──────────────────────────────────────────────────────────────

/// Aggregated health result for one resource.
///
/// `overall` is the maximum level among all issues (Ok when no issues).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HealthStatus {
    /// Worst level among all issues.
    pub overall: HealthLevel,
    /// All issues found, in the order they were pushed.
    pub issues: Vec<HealthIssue>,
}

impl HealthStatus {
    /// Create a clean (Ok, no issues) status.
    pub fn ok() -> Self {
        Self::default()
    }

    /// Add an issue, updating `overall` if the issue is more severe.
    pub fn push(&mut self, issue: HealthIssue) {
        if issue.level > self.overall {
            self.overall = issue.level;
        }
        self.issues.push(issue);
    }

    /// Push an Error-level issue.
    pub fn error(&mut self, msg_key: &'static str) {
        self.push(HealthIssue::error(msg_key));
    }

    /// Push a Warning-level issue.
    pub fn warning(&mut self, msg_key: &'static str) {
        self.push(HealthIssue::warning(msg_key));
    }

    /// `true` when there are no issues at all.
    pub fn is_ok(&self) -> bool {
        self.issues.is_empty()
    }

    /// All issues at or above `min_level`.
    pub fn issues_at_level(&self, min_level: HealthLevel) -> impl Iterator<Item = &HealthIssue> {
        self.issues.iter().filter(move |i| i.level >= min_level)
    }

    /// Merge another `HealthStatus` into this one (takes the worst overall level).
    pub fn merge(&mut self, other: HealthStatus) {
        for issue in other.issues {
            self.push(issue);
        }
    }
}

// ── HealthRules ───────────────────────────────────────────────────────────────

/// Fluent builder for constructing `HealthStatus` from named rules.
///
/// ```ignore
/// let status = HealthRules::new()
///     .require(!host.is_empty(), "health.host.no_proxy")
///     .warn(!services.is_empty(), "health.project.no_monitoring")
///     .build();
/// ```
#[derive(Default)]
pub struct HealthRules {
    status: HealthStatus,
}

impl HealthRules {
    /// Start a new rule set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an Error if `condition` is `false`.
    pub fn require(mut self, condition: bool, msg_key: &'static str) -> Self {
        if !condition {
            self.status.error(msg_key);
        }
        self
    }

    /// Add a Warning if `condition` is `false`.
    pub fn warn(mut self, condition: bool, msg_key: &'static str) -> Self {
        if !condition {
            self.status.warning(msg_key);
        }
        self
    }

    /// Finalize and return the `HealthStatus`.
    pub fn build(self) -> HealthStatus {
        self.status
    }
}
