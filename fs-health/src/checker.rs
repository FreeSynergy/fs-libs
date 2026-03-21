// HealthCheck trait — implemented by any resource that can assess itself.

use crate::reporter::HealthStatus;

// ── HealthCheck ───────────────────────────────────────────────────────────────

/// A type that can assess its own health.
///
/// Implement this for any resource (project, host, service) to make it
/// compatible with the generic health reporting infrastructure.
///
/// # Example
/// ```rust,ignore
/// use fs_health::{HealthCheck, HealthRules, HealthStatus};
///
/// struct MyService { name: String, host: Option<String> }
///
/// impl HealthCheck for MyService {
///     fn health(&self) -> HealthStatus {
///         HealthRules::new()
///             .require(self.host.is_some(), "health.service.no_host")
///             .build()
///     }
/// }
/// ```
pub trait HealthCheck {
    /// Run all health checks and return the aggregated status.
    fn health(&self) -> HealthStatus;
}
