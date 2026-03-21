// Registry of active bridge connections.

use std::collections::HashMap;

use fs_error::FsError;

use crate::bridge::{BridgeInfo, ProbableBridge};

// ── BridgeRegistry ────────────────────────────────────────────────────────────

/// Registry of active bridge connections.
///
/// Implements the Service Locator pattern: bridges are registered by their
/// `service_id` and looked up at runtime without compile-time coupling.
///
/// # Example
/// ```rust,ignore
/// let mut registry = BridgeRegistry::new();
/// registry.register(my_forgejo_bridge);
/// let results = registry.probe_all().await;
/// ```
pub struct BridgeRegistry {
    bridges: Vec<Box<dyn ProbableBridge>>,
}

impl BridgeRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self { bridges: Vec::new() }
    }

    /// Register a bridge under its `service_id`.
    pub fn register(&mut self, bridge: impl ProbableBridge + 'static) {
        self.bridges.push(Box::new(bridge));
    }

    /// Retrieve a bridge by `service_id`. Returns `None` if not registered.
    pub fn get(&self, service_id: &str) -> Option<&dyn ProbableBridge> {
        self.bridges
            .iter()
            .find(|b| b.service_id() == service_id)
            .map(|b| b.as_ref())
    }

    /// List all registered service IDs.
    pub fn service_ids(&self) -> Vec<&str> {
        self.bridges.iter().map(|b| b.service_id()).collect()
    }

    /// Probe all registered bridges sequentially.
    ///
    /// Returns a list of `(service_id, Result<BridgeInfo>)` pairs in registration order.
    pub async fn probe_all(&self) -> Vec<(String, Result<BridgeInfo, FsError>)> {
        let mut results = Vec::with_capacity(self.bridges.len());
        for bridge in &self.bridges {
            let id = bridge.service_id().to_string();
            let result = bridge.probe().await;
            results.push((id, result));
        }
        results
    }

    /// Probe all registered bridges and return a map of `service_id → Result<BridgeInfo>`.
    pub async fn probe_all_map(&self) -> HashMap<String, Result<BridgeInfo, FsError>> {
        self.probe_all().await.into_iter().collect()
    }
}

impl Default for BridgeRegistry {
    fn default() -> Self {
        Self::new()
    }
}
