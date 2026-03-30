// HealthMonitor — Observer Pattern.
//
// Registers multiple `HealthCheck` implementors and aggregates their results
// into a single `HealthStatus` on demand.

use std::collections::HashMap;

use crate::{checker::HealthCheck, reporter::HealthStatus};

// ── HealthMonitor ─────────────────────────────────────────────────────────────

/// Observes a set of named health-check subjects and aggregates their results.
///
/// # Example
/// ```rust,ignore
/// use fs_health::{HealthMonitor, HealthRules, HealthStatus, HealthCheck};
///
/// struct MyService { alive: bool }
/// impl HealthCheck for MyService {
///     fn health(&self) -> HealthStatus {
///         HealthRules::new().require(self.alive, "health-service-down").build()
///     }
/// }
///
/// let mut monitor = HealthMonitor::new();
/// monitor.register("my-service", MyService { alive: true });
/// assert!(monitor.overall().is_ok());
/// ```
#[derive(Default)]
pub struct HealthMonitor {
    subjects: Vec<(&'static str, Box<dyn HealthCheck>)>,
}

impl HealthMonitor {
    /// Create an empty monitor.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a named health-check subject.
    pub fn register(&mut self, name: &'static str, subject: impl HealthCheck + 'static) {
        self.subjects.push((name, Box::new(subject)));
    }

    /// Run all registered subjects and return their individual results.
    #[must_use]
    pub fn run_all(&self) -> HashMap<&'static str, HealthStatus> {
        self.subjects
            .iter()
            .map(|(name, check)| (*name, check.health()))
            .collect()
    }

    /// Run all subjects and merge the results into one aggregated `HealthStatus`.
    #[must_use]
    pub fn overall(&self) -> HealthStatus {
        self.subjects
            .iter()
            .fold(HealthStatus::ok(), |mut acc, (_, check)| {
                acc.merge(check.health());
                acc
            })
    }
}
