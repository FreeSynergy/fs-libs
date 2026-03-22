// registry.rs — ManagerRegistry: central collection of all FsManagers.
//
// The registry lets the Node process, health dashboard, and Desktop iterate
// over all registered managers without coupling to concrete types.
//
// Pattern: Registry + Iterator — managers are registered once at startup,
// then queried by ID or iterated for health checks.

use crate::manager::FsManager;

// ── HealthStatus ──────────────────────────────────────────────────────────────

/// Health snapshot for a single manager.
#[derive(Debug, Clone)]
pub struct HealthStatus {
    /// Stable manager ID (e.g. `"theme"`, `"language"`).
    pub id:      String,
    /// Human-readable manager name.
    pub name:    String,
    /// `true` when the manager reports itself as fully operational.
    pub healthy: bool,
}

// ── ManagerRegistry ───────────────────────────────────────────────────────────

/// Central registry of all active FreeSynergy managers.
///
/// Populated once at startup; queried by the Node API and the Desktop health
/// dashboard without any coupling to concrete manager types.
///
/// # Example
///
/// ```rust,ignore
/// use fs_core::registry::ManagerRegistry;
///
/// let mut registry = ManagerRegistry::new();
/// registry.register(ThemeManager::new());
/// registry.register(LanguageManager::new());
///
/// // Health check all
/// for status in registry.health_check_all() {
///     println!("{}: {}", status.name, if status.healthy { "OK" } else { "FAIL" });
/// }
///
/// // Look up by ID
/// if let Some(mgr) = registry.get("theme") {
///     println!("Theme manager: {}", mgr.name());
/// }
/// ```
#[derive(Default)]
pub struct ManagerRegistry {
    managers: Vec<Box<dyn FsManager>>,
}

impl ManagerRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a manager.
    ///
    /// If a manager with the same `id()` already exists it is replaced.
    pub fn register(&mut self, mgr: impl FsManager + 'static) {
        let id = mgr.id().to_owned();
        self.managers.retain(|m| m.id() != id);
        self.managers.push(Box::new(mgr));
    }

    /// Look up a manager by its stable ID.
    pub fn get(&self, id: &str) -> Option<&dyn FsManager> {
        self.managers.iter().find(|m| m.id() == id).map(Box::as_ref)
    }

    /// Snapshot health status for every registered manager.
    pub fn health_check_all(&self) -> Vec<HealthStatus> {
        self.managers
            .iter()
            .map(|m| HealthStatus {
                id:      m.id().to_owned(),
                name:    m.name().to_owned(),
                healthy: m.is_healthy(),
            })
            .collect()
    }

    /// `true` when every registered manager reports itself as healthy.
    pub fn all_healthy(&self) -> bool {
        self.managers.iter().all(|m| m.is_healthy())
    }

    /// Iterate over all registered managers.
    pub fn all(&self) -> impl Iterator<Item = &dyn FsManager> {
        self.managers.iter().map(Box::as_ref)
    }

    /// Number of registered managers.
    pub fn len(&self) -> usize {
        self.managers.len()
    }

    /// `true` when no managers have been registered.
    pub fn is_empty(&self) -> bool {
        self.managers.is_empty()
    }
}
