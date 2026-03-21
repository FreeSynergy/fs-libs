// Module manifest — the [plugin] block inside service module TOML files.
//
// Declares what commands the module supports, what cross-service data it
// needs from Core, and what files it produces.

use serde::{Deserialize, Serialize};

// ── ModuleManifest ────────────────────────────────────────────────────────────

/// The `[plugin]` block in a service module TOML.
///
/// Declares what commands the module supports, what cross-service data it
/// needs from Core, and what files it produces.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModuleManifest {
    /// Commands this module handles (e.g. `["deploy", "clean", "generate-config"]`).
    #[serde(default)]
    pub commands: Vec<String>,

    /// Cross-service data the module needs — Core collects and injects these.
    #[serde(default)]
    pub inputs: ManifestInputs,

    /// Files this module generates (Core writes them after plugin runs).
    #[serde(default, rename = "outputs")]
    pub output_files: Vec<ManifestOutputFile>,

    /// Passive/data-only plugin: no container, no deploy, just exposes config data.
    #[serde(default)]
    pub external: bool,

    /// JSON protocol version — must be 1.
    #[serde(default = "protocol_v1")]
    pub protocol: u32,
}

fn protocol_v1() -> u32 {
    1
}

/// Cross-service inputs a module may request.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ManifestInputs {
    /// Receive the full list of peer services (their domains, ports, types).
    ///
    /// Used by proxy, mail, wiki, IAM to produce per-service routing configs.
    #[serde(default)]
    pub services: bool,

    /// Receive IAM service vars (`IAM_URL`, `IAM_DOMAIN`, …) when an IAM service exists.
    #[serde(default)]
    pub iam_vars: bool,
}

/// A file the module plugin generates.
///
/// Core renders the template with Jinja2 and writes the result to `dest`.
/// The template path is relative to the module's Store directory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestOutputFile {
    /// Identifier used in logs and dependency tracking (e.g. `"proxy-config"`).
    pub name: String,

    /// Template file path, relative to the module's Store directory.
    ///
    /// E.g. `"templates/zentinel.kdl.j2"`.
    pub template: String,

    /// Absolute destination path — Jinja2 vars like `{{ data_root }}` are expanded.
    pub dest: String,
}
