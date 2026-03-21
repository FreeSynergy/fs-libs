//! `ContainerResource` — a containerized application with Compose YAML.

use super::meta::{ResourceMeta, Role};
use serde::{Deserialize, Serialize};

// ── VarType ───────────────────────────────────────────────────────────────────

/// Semantic type of a container configuration variable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VarType {
    /// Sensitive value that must never appear in plain text logs.
    Secret,
    /// A full URL, e.g. `"https://kanidm.example.com"`.
    Url,
    /// A bare hostname or container name, e.g. `"kanidm"`.
    Hostname,
    /// A TCP/UDP port number.
    Port,
    /// An email address.
    Email,
    /// A file system path inside the container.
    Path,
    /// A boolean flag (`"true"` / `"false"`).
    Bool,
    /// An integer value.
    Int,
    /// A free-form string.
    String,
}

// ── AutoSource ────────────────────────────────────────────────────────────────

/// Where a variable value can be automatically derived from.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "source")]
pub enum AutoSource {
    /// Derive from a sibling sub-service by its compose service name.
    InternalService {
        /// The compose service name, e.g. `"ollama"`.
        service_name: String,
        /// URL template, e.g. `"http://ollama:11434"`.
        url_template: String,
    },
    /// Derive from the Inventory: find the service that provides `role` and read `field`.
    RoleVariable {
        /// The role to look up, e.g. `"iam"`.
        role: Role,
        /// Which field to read from the service's config, e.g. `"oidc_discovery_url"`.
        field: String,
    },
}

// ── ContainerVariable ─────────────────────────────────────────────────────────

/// A declared configuration variable of a container application.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerVariable {
    /// Variable name as it appears in the Compose YAML, e.g. `"WEBUI_SECRET_KEY"`.
    pub name: String,
    /// Semantic type of the value.
    pub var_type: VarType,
    /// The role this variable is sourced from (if any).
    pub role: Option<Role>,
    /// Whether this variable must be set before the container starts.
    pub required: bool,
    /// Default value used when the variable is not explicitly set.
    pub default: Option<String>,
    /// Source from which the value can be auto-populated.
    pub auto_from: Option<AutoSource>,
    /// Human-readable explanation shown in the Conductor UI.
    pub description: String,
    /// Confidence score (0.0–1.0) when this variable was auto-detected.
    pub confidence: f32,
}

// ── ContainerService ──────────────────────────────────────────────────────────

/// A service defined inside the Compose YAML (main service or sub-service).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerService {
    /// Compose service name, e.g. `"kanidm"` or `"postgres"`.
    pub name: String,
    /// Container image, e.g. `"kanidm/server:latest"`.
    pub image: String,
    /// `true` for the primary service; `false` for infrastructure sub-services.
    pub is_main: bool,
    /// `true` when this service is only reachable inside the compose network.
    pub internal: bool,
    /// Host port exposed (if any).
    pub port: Option<u16>,
    /// Healthcheck configuration embedded in the Compose YAML.
    pub healthcheck: Option<String>,
    /// Docker image tag used as the version identifier.
    pub version_tag: String,
}

// ── NetworkDef ────────────────────────────────────────────────────────────────

/// A Docker network declared in the Compose YAML.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkDef {
    /// Network name, e.g. `"kanidm-backend"`.
    pub name: String,
    /// `true` when the network is shared with other containers.
    pub external: bool,
}

// ── VolumeDef ─────────────────────────────────────────────────────────────────

/// A Docker volume declared in the Compose YAML.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeDef {
    /// Volume name in the Compose YAML.
    pub name: String,
    /// Suggested S3 path for backup, e.g. `"backups/kanidm/data"`.
    pub s3_path: Option<String>,
}

// ── RoleApiRef ────────────────────────────────────────────────────────────────

/// A reference to a standardized role API exposed via a Bridge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleApiRef {
    /// The role this API covers, e.g. `"iam"`.
    pub role: Role,
    /// The bridge resource id that maps this service to the role API.
    pub bridge_id: String,
}

// ── RoleDep ───────────────────────────────────────────────────────────────────

/// A role dependency declared by this container app.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleDep {
    /// The role that must be available, e.g. `"smtp"`.
    pub role: Role,
    /// When `true`, the app runs in degraded mode without this role.
    pub optional: bool,
}

// ── ContainerResource ─────────────────────────────────────────────────────

/// A containerized application shipped with its Compose YAML (Kanidm, Forgejo, …).
///
/// The Conductor converts the `compose_yaml` into Quadlet unit files.
/// Container apps have **no bundled translations** — that is handled separately.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerResource {
    /// Shared metadata present on every resource.
    pub meta: ResourceMeta,
    /// Original Compose YAML content (stored verbatim, downloadable).
    pub compose_yaml: String,
    /// All services declared in the Compose YAML (main + sub-services).
    pub services: Vec<ContainerService>,
    /// Roles this container app provides when deployed.
    pub roles_provided: Vec<Role>,
    /// Roles this container app requires from the environment.
    pub roles_required: Vec<RoleDep>,
    /// Standardized role APIs exposed via Bridges.
    pub apis: Vec<RoleApiRef>,
    /// Configuration variables (sourced from Compose YAML + auto-detection).
    pub variables: Vec<ContainerVariable>,
    /// Docker networks declared in the Compose YAML.
    pub networks: Vec<NetworkDef>,
    /// Docker volumes declared in the Compose YAML.
    pub volumes: Vec<VolumeDef>,
}
