//! Domain models for the inventory — what is installed and running.

use fsn_types::{ResourceType, Role, ValidationStatus};
use serde::{Deserialize, Serialize};

// ── ReleaseChannel ────────────────────────────────────────────────────────────

/// Which store release channel a resource was installed from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReleaseChannel {
    Stable,
    Testing,
    Nightly,
}

impl ReleaseChannel {
    pub fn label(self) -> &'static str {
        match self {
            ReleaseChannel::Stable  => "Stable",
            ReleaseChannel::Testing => "Testing",
            ReleaseChannel::Nightly => "Nightly",
        }
    }
}

impl Default for ReleaseChannel {
    fn default() -> Self {
        Self::Stable
    }
}

// ── ResourceStatus ────────────────────────────────────────────────────────────

/// Runtime state of an installed resource.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "state", content = "detail")]
pub enum ResourceStatus {
    Active,
    Stopped,
    Error(String),
    Updating,
    Installing,
}

impl ResourceStatus {
    pub fn label(&self) -> &str {
        match self {
            ResourceStatus::Active      => "Active",
            ResourceStatus::Stopped     => "Stopped",
            ResourceStatus::Error(_)    => "Error",
            ResourceStatus::Updating    => "Updating",
            ResourceStatus::Installing  => "Installing",
        }
    }

    pub fn needs_attention(&self) -> bool {
        matches!(self, ResourceStatus::Error(_))
    }
}

// ── InstalledResource ─────────────────────────────────────────────────────────

/// A resource that has been downloaded and installed on this node.
///
/// One row per installed resource — all resource types share this table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledResource {
    /// Resource slug, e.g. `"kanidm"`.  Primary key.
    pub id: String,
    /// What kind of resource this is.
    pub resource_type: ResourceType,
    /// Installed version string, e.g. `"1.5.0"`.
    pub version: String,
    /// Which release channel was used.
    pub channel: ReleaseChannel,
    /// ISO-8601 installation timestamp.
    pub installed_at: String,
    /// Runtime status.
    pub status: ResourceStatus,
    /// Path to the resource's configuration file.
    pub config_path: String,
    /// Path to the resource's data directory.
    pub data_path: String,
    /// Structural completeness and signature status.
    pub validation: ValidationStatus,
}

// ── ServiceStatus ─────────────────────────────────────────────────────────────

/// Runtime state of a container service instance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "state", content = "detail")]
pub enum ServiceStatus {
    Running,
    Stopped,
    Starting,
    Error(String),
}

impl ServiceStatus {
    pub fn is_running(&self) -> bool {
        matches!(self, ServiceStatus::Running)
    }
}

// ── ConfiguredVar ─────────────────────────────────────────────────────────────

/// A configuration variable with its resolved value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfiguredVar {
    /// Variable name, e.g. `"WEBUI_SECRET_KEY"`.
    pub name: String,
    /// Resolved value (secrets stored encrypted separately).
    pub value: Option<String>,
}

// ── BridgeRef ─────────────────────────────────────────────────────────────────

/// A reference to an active bridge instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeRef {
    /// Bridge resource id, e.g. `"kanidm-iam-bridge"`.
    pub bridge_id: String,
    /// Role served by this bridge, e.g. `"iam"`.
    pub role: Role,
}

// ── ServiceInstance ───────────────────────────────────────────────────────────

/// A running (or stopped) container service instance.
///
/// Multiple instances of the same resource can exist (e.g. two Kanidm instances).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInstance {
    /// Unique instance identifier (UUID).
    pub id: String,
    /// The installed resource this instance is derived from.
    pub resource_id: String,
    /// User-assigned name, e.g. `"main-iam"`.
    pub instance_name: String,
    /// Roles this instance provides, e.g. `["iam"]`.
    pub roles_provided: Vec<Role>,
    /// Roles this instance requires from other services.
    pub roles_required: Vec<Role>,
    /// Active bridge instances attached to this service.
    pub bridges: Vec<BridgeRef>,
    /// Configured environment variables.
    pub variables: Vec<ConfiguredVar>,
    /// Docker network name.
    pub network: String,
    /// Runtime state.
    pub status: ServiceStatus,
    /// Host port exposed (if any).
    pub port: Option<u16>,
    /// S3 paths used by this instance for data storage.
    pub s3_paths: Vec<String>,
}

// ── BridgeStatus ──────────────────────────────────────────────────────────────

/// Runtime state of a bridge instance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BridgeStatus {
    Active,
    Unreachable,
    Misconfigured,
}

// ── BridgeInstance ────────────────────────────────────────────────────────────

/// An active bridge connecting a standard role API to a concrete service.
///
/// The Bus queries the Inventory for bridge instances when it needs to route
/// a role-based request to the correct service endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeInstance {
    /// Unique instance identifier (UUID).
    pub id: String,
    /// Bridge resource id, e.g. `"kanidm-iam-bridge"`.
    pub bridge_id: String,
    /// The standardized role this bridge serves.
    pub role: Role,
    /// The service instance id this bridge is attached to.
    pub service_instance: String,
    /// Base URL of the service API, e.g. `"http://kanidm:8443"`.
    pub api_base_url: String,
    /// Runtime reachability status.
    pub status: BridgeStatus,
}
