// PluginContext — JSON payload Core writes to a plugin's stdin.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

// ── PluginContext ─────────────────────────────────────────────────────────────

/// JSON payload Core writes to a plugin's stdin.
///
/// The plugin reads this, processes the requested command, and writes a
/// [`PluginResponse`][crate::PluginResponse] to stdout.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginContext {
    /// Protocol version — plugin must reject if it doesn't support this version.
    ///
    /// Must equal `1` for the current protocol.
    pub protocol: u32,

    /// The command to execute (must be in `ModuleManifest::commands`).
    pub command: String,

    /// The service instance being operated on.
    pub instance: InstanceInfo,

    /// Peer services in the same project (provided when `ManifestInputs::services = true`).
    #[serde(default)]
    pub peers: Vec<PeerService>,

    /// Cross-service environment variables collected from all peer services.
    ///
    /// E.g. `IAM_URL`, `MAIL_DOMAIN`, `GIT_HOST`, …
    #[serde(default)]
    pub env: HashMap<String, String>,
}

// ── InstanceInfo ──────────────────────────────────────────────────────────────

/// Information about the service instance being operated on.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceInfo {
    /// Instance name (e.g. `"zentinel"`).
    pub name: String,

    /// Service class key (e.g. `"proxy/zentinel"`).
    pub class_key: String,

    /// Fully qualified service domain (e.g. `"zentinel.example.com"`).
    pub domain: String,

    /// Project slug (e.g. `"example"`).
    pub project: String,

    /// Project domain (e.g. `"example.com"`).
    pub project_domain: String,

    /// Data root directory for this instance.
    pub data_root: String,

    /// Resolved environment variables for this instance (from `[environment]` + vault).
    #[serde(default)]
    pub env: HashMap<String, String>,
}

// ── PeerService ───────────────────────────────────────────────────────────────

/// A peer service — one of the other services running in the same project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerService {
    /// Instance name (e.g. `"forgejo"`).
    pub name: String,

    /// Service class key (e.g. `"git/forgejo"`).
    pub class_key: String,

    /// Functional types (e.g. `["git"]`).
    pub types: Vec<String>,

    /// Fully qualified domain (e.g. `"forgejo.example.com"`).
    pub domain: String,

    /// Primary port.
    pub port: u16,

    /// Whether the upstream speaks TLS internally.
    pub upstream_tls: bool,

    /// HTTP routes declared in the module's `[contract]`.
    #[serde(default)]
    pub routes: Vec<PeerRoute>,

    /// Resolved environment vars exported by this peer (for cross-service injection).
    #[serde(default)]
    pub exported_vars: HashMap<String, String>,
}

// ── PeerRoute ─────────────────────────────────────────────────────────────────

/// An HTTP route declared by a peer service in its module contract.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerRoute {
    /// Route identifier (e.g. `"api"`).
    pub id: String,

    /// URL path prefix (e.g. `"/api"`).
    pub path: String,

    /// Whether the proxy strips this prefix before forwarding the request.
    pub strip: bool,
}
