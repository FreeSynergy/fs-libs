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
    catalog::{forgejo_git_bridge, kanidm_iam_bridge, outline_wiki_bridge},
    error::BridgeError,
    executor::BridgeExecutor,
};
use fsn_inventory::Inventory;
use fsn_types::resources::bridge::BridgeResource;
use serde_json::Value;
use std::sync::Arc;
use tracing::instrument;

/// Routes a standardized role API call through the Inventory to the right bridge.
pub struct BridgeDispatcher {
    inventory: Arc<Inventory>,
}

impl BridgeDispatcher {
    /// Create a dispatcher backed by the given inventory.
    pub fn new(inventory: Arc<Inventory>) -> Self {
        Self { inventory }
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
                use fsn_inventory::models::BridgeStatus;
                b.status == BridgeStatus::Active
            })
            .ok_or_else(|| BridgeError::NoBridgeForRole { role: role.to_owned() })?;

        // 2. Load the bridge resource definition from the catalog (or disk).
        let resource = load_catalog_bridge(&instance.bridge_id).ok_or_else(|| {
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

// ── Catalog lookup ────────────────────────────────────────────────────────────

/// Return the built-in bridge definition for well-known bridge IDs.
///
/// Production nodes will extend this with custom bridges loaded from disk.
fn load_catalog_bridge(bridge_id: &str) -> Option<BridgeResource> {
    match bridge_id {
        "kanidm-iam-bridge"   => Some(kanidm_iam_bridge()),
        "outline-wiki-bridge" => Some(outline_wiki_bridge()),
        "forgejo-git-bridge"  => Some(forgejo_git_bridge()),
        _                     => None,
    }
}
