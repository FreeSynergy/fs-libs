//! `AppResource` — a native FreeSynergy binary application.

use super::meta::{ResourceMeta, Role};
use serde::{Deserialize, Serialize};

// ── Platform ──────────────────────────────────────────────────────────────────

/// Supported target platforms for binary distribution.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Platform {
    LinuxX86_64,
    LinuxAarch64,
    MacosX86_64,
    MacosAarch64,
    WindowsX86_64,
}

impl Platform {
    /// The canonical target triple string.
    pub fn target_triple(&self) -> &'static str {
        match self {
            Platform::LinuxX86_64    => "x86_64-unknown-linux-gnu",
            Platform::LinuxAarch64   => "aarch64-unknown-linux-gnu",
            Platform::MacosX86_64    => "x86_64-apple-darwin",
            Platform::MacosAarch64   => "aarch64-apple-darwin",
            Platform::WindowsX86_64  => "x86_64-pc-windows-msvc",
        }
    }
}

// ── CliCommand ────────────────────────────────────────────────────────────────

/// A CLI sub-command exposed by this app.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliCommand {
    /// Command name, e.g. `"serve"`.
    pub name: String,
    /// Short description for `--help` output.
    pub description: String,
}

// ── ApiEndpoint ───────────────────────────────────────────────────────────────

/// A REST API endpoint exposed by this app.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiEndpoint {
    /// HTTP method, e.g. `"GET"` or `"POST"`.
    pub method: String,
    /// Path pattern, e.g. `"/api/store/packages"`.
    pub path: String,
    /// Short description.
    pub description: String,
}

// ── RoleDep ───────────────────────────────────────────────────────────────────

/// A role dependency declared by this app.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleDep {
    /// The role id that must be fulfilled, e.g. `"iam"`.
    pub role: Role,
    /// When `true`, the app starts in degraded mode without this role.
    pub optional: bool,
}

// ── AppResource ───────────────────────────────────────────────────────────────

/// A native FreeSynergy binary application (Node, Desktop, Conductor, …).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppResource {
    /// Shared metadata present on every resource.
    pub meta: ResourceMeta,
    /// Target platforms for which binaries are provided.
    pub platforms: Vec<Platform>,
    /// Name of the main binary file, e.g. `"fs-node"`.
    pub binary_name: String,
    /// Locale codes bundled with this app, e.g. `["en", "de"]`.
    pub locales: Vec<String>,
    /// JSON Schema string describing the app's configuration file.
    pub config_schema: Option<String>,
    /// CLI sub-commands exposed by this app.
    pub cli_commands: Vec<CliCommand>,
    /// REST API endpoints exposed by this app.
    pub api_endpoints: Vec<ApiEndpoint>,
    /// Roles this app provides when running.
    pub roles_provided: Vec<Role>,
    /// Roles this app requires from other services.
    pub roles_required: Vec<RoleDep>,
}
