//! `BridgeDispatcher` — high-level entry point for role-based bridge calls.
//!
//! Given a role name (e.g. `"iam"`), the dispatcher:
//! 1. Queries the Inventory for an active `BridgeInstance` serving that role.
//! 2. Loads the matching `BridgeResource` definition from the catalog.
//! 3. Delegates execution to `BridgeExecutor`.
//!
//! The Bus (Phase J) will use `BridgeDispatcher` to route role-addressed
//! messages to the correct concrete service.

use crate::{
    catalog::BuiltinCatalog,
    error::BridgeError,
    executor::BridgeExecutor,
};
use fs_inventory::Inventory;
use fs_types::resources::bridge::BridgeResource;
use serde_json::Value;
use std::sync::Arc;
use tracing::instrument;

// ── BridgeCatalog ─────────────────────────────────────────────────────────────

/// Resolves a bridge ID to its resource definition.
///
/// Implement this trait to extend the dispatcher with bridges loaded from disk,
/// the store, or any other source. The built-in implementation is [`BuiltinCatalog`].
pub trait BridgeCatalog: Send + Sync {
    /// Return the bridge definition for `bridge_id`, or `None` if not known.
    fn load(&self, bridge_id: &str) -> Option<BridgeResource>;
}

/// Routes a standardized role API call through the Inventory to the right bridge.
pub struct BridgeDispatcher {
    inventory: Arc<Inventory>,
    catalog:   Arc<dyn BridgeCatalog>,
}

impl BridgeDispatcher {
    /// Create a dispatcher with the built-in bridge catalog.
    pub fn new(inventory: Arc<Inventory>) -> Self {
        Self { inventory, catalog: Arc::new(BuiltinCatalog) }
    }

    /// Create a dispatcher with a custom catalog (e.g. loaded from disk or the store).
    pub fn with_catalog(inventory: Arc<Inventory>, catalog: Arc<dyn BridgeCatalog>) -> Self {
        Self { inventory, catalog }
    }

    /// Execute a standard role API method.
    ///
    /// Looks up the active bridge for `role` in the Inventory, then forwards
    /// the call through the matching `BridgeExecutor`.
    #[instrument(name = "dispatcher.execute", skip(self, params))]
    pub async fn execute(
        &self,
        role: &str,
        method: &str,
        params: Value,
    ) -> Result<Value, BridgeError> {
        // 1. Find the active bridge instance for this role.
        let bridges = self
            .inventory
            .bridges_for_role(role)
            .await
            .map_err(|e| BridgeError::Inventory(e.to_string()))?;

        let instance = bridges
            .into_iter()
            .find(|b| {
                use fs_inventory::models::BridgeStatus;
                b.status == BridgeStatus::Active
            })
            .ok_or_else(|| BridgeError::NoBridgeForRole { role: role.to_owned() })?;

        // 2. Load the bridge resource definition from the catalog.
        let resource = self.catalog.load(&instance.bridge_id).ok_or_else(|| {
            BridgeError::MethodNotFound {
                method: method.to_owned(),
                bridge_id: instance.bridge_id.clone(),
            }
        })?;

        // 3. Execute via BridgeExecutor.
        let executor = BridgeExecutor::new(resource, &instance.api_base_url);
        executor.execute(method, params).await
    }
}

