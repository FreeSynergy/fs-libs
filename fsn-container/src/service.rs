// ServiceConfig — declarative container service description.
//
// A `ServiceConfig` describes everything needed to generate a Podman Quadlet
// `.container` unit file.  No bollard, no socket — just plain data.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

// ── PortBinding ───────────────────────────────────────────────────────────────

/// A host ↔ container port mapping.
///
/// Only Zentinel (the proxy) should expose ports to the outside world.
/// All other services communicate on the internal Podman network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortBinding {
    /// Port bound on the host.
    pub host_port: u16,
    /// Port exposed inside the container.
    pub container_port: u16,
    /// Protocol: `"tcp"` or `"udp"`.
    #[serde(default = "default_protocol")]
    pub protocol: String,
}

fn default_protocol() -> String { "tcp".to_string() }

impl PortBinding {
    /// Create a TCP port binding `host_port:container_port`.
    pub fn tcp(host_port: u16, container_port: u16) -> Self {
        Self { host_port, container_port, protocol: "tcp".to_string() }
    }

    /// Render as `"host:container/proto"` (Quadlet `PublishPort=` format).
    pub fn to_quadlet_line(&self) -> String {
        format!("{}:{}/{}", self.host_port, self.container_port, self.protocol)
    }
}

// ── Volume ────────────────────────────────────────────────────────────────────

/// A host ↔ container volume mount.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Volume {
    /// Absolute path on the host (or a named volume identifier).
    pub host: String,
    /// Absolute path inside the container.
    pub container: String,
    /// Optional mount options, e.g. `"ro"`, `"z"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub options: Option<String>,
}

impl Volume {
    /// Create a read-write bind mount.
    pub fn bind(host: impl Into<String>, container: impl Into<String>) -> Self {
        Self { host: host.into(), container: container.into(), options: None }
    }

    /// Create a read-only bind mount.
    pub fn bind_ro(host: impl Into<String>, container: impl Into<String>) -> Self {
        Self { host: host.into(), container: container.into(), options: Some("ro".to_string()) }
    }

    /// Render as `"host:container[:opts]"` (Quadlet `Volume=` format).
    pub fn to_quadlet_line(&self) -> String {
        match &self.options {
            Some(opts) => format!("{}:{}:{}", self.host, self.container, opts),
            None       => format!("{}:{}", self.host, self.container),
        }
    }
}

// ── RestartPolicy ─────────────────────────────────────────────────────────────

/// Systemd restart policy for the service unit.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RestartPolicy {
    /// Always restart on failure (default).
    #[default]
    Always,
    /// Only restart on non-zero exit (not on clean exit).
    OnFailure,
    /// Never restart.
    No,
}

impl RestartPolicy {
    /// The string value used in the `[Service]` section.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Always    => "always",
            Self::OnFailure => "on-failure",
            Self::No        => "no",
        }
    }
}

// ── HealthCheck ───────────────────────────────────────────────────────────────

/// Container health check configuration.
///
/// Every FSN service module **must** declare a health check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    /// Command to run inside the container, e.g. `["curl", "-fs", "http://localhost/health"]`.
    pub test: Vec<String>,
    /// Interval between checks (systemd time span, e.g. `"30s"`).
    #[serde(default = "default_interval")]
    pub interval: String,
    /// Time after which a check is considered failed (e.g. `"10s"`).
    #[serde(default = "default_timeout")]
    pub timeout: String,
    /// Number of consecutive failures before declaring unhealthy.
    #[serde(default = "default_retries")]
    pub retries: u32,
    /// Grace period at startup before health checks begin (e.g. `"5s"`).
    #[serde(default = "default_start_period")]
    pub start_period: String,
}

fn default_interval()     -> String { "30s".to_string() }
fn default_timeout()      -> String { "10s".to_string() }
fn default_retries()      -> u32    { 3 }
fn default_start_period() -> String { "5s".to_string() }

impl HealthCheck {
    /// Render as the Quadlet `HealthCmd=` line (space-separated command).
    pub fn to_quadlet_cmd(&self) -> String {
        self.test.join(" ")
    }
}

// ── ServiceConfig ─────────────────────────────────────────────────────────────

/// Declarative description of a container service.
///
/// Passed to [`super::QuadletManager::create_quadlet`] to generate a
/// Podman Quadlet `.container` unit file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    /// Short service name, e.g. `"zentinel"` (used for unit file name).
    pub name: String,
    /// Container image reference, e.g. `"ghcr.io/freeSynergy/zentinel:latest"`.
    pub image: String,
    /// Human-readable description for the `[Unit]` section.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Environment variables injected into the container.
    #[serde(default)]
    pub environment: HashMap<String, String>,
    /// Volume mounts.
    #[serde(default)]
    pub volumes: Vec<Volume>,
    /// Published ports (only for the Zentinel proxy service).
    #[serde(default)]
    pub ports: Vec<PortBinding>,
    /// Container labels.
    #[serde(default)]
    pub labels: HashMap<String, String>,
    /// Health check configuration (required for every FSN service).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub healthcheck: Option<HealthCheck>,
    /// Restart policy for the `[Service]` section.
    #[serde(default)]
    pub restart_policy: RestartPolicy,
    /// Podman network name (default: `"fsn"`).
    #[serde(default = "default_network")]
    pub network: String,
    /// Optional container user override.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

fn default_network() -> String { "fsn".to_string() }

impl ServiceConfig {
    /// Create a minimal service config.
    pub fn new(name: impl Into<String>, image: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            image: image.into(),
            description: None,
            environment: HashMap::new(),
            volumes: Vec::new(),
            ports: Vec::new(),
            labels: HashMap::new(),
            healthcheck: None,
            restart_policy: RestartPolicy::Always,
            network: default_network(),
            user: None,
        }
    }

    /// The systemd unit name: `"fsn-{name}.service"`.
    pub fn unit_name(&self) -> String {
        format!("fsn-{}.service", self.name)
    }

    /// The Quadlet file name: `"fsn-{name}.container"`.
    pub fn quadlet_filename(&self) -> String {
        format!("fsn-{}.container", self.name)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn port_binding_quadlet_line() {
        let p = PortBinding::tcp(443, 443);
        assert_eq!(p.to_quadlet_line(), "443:443/tcp");
    }

    #[test]
    fn volume_quadlet_line_ro() {
        let v = Volume::bind_ro("/data/zentinel", "/data");
        assert_eq!(v.to_quadlet_line(), "/data/zentinel:/data:ro");
    }

    #[test]
    fn service_config_unit_name() {
        let svc = ServiceConfig::new("zentinel", "ghcr.io/freeSynergy/zentinel:latest");
        assert_eq!(svc.unit_name(), "fsn-zentinel.service");
        assert_eq!(svc.quadlet_filename(), "fsn-zentinel.container");
    }

    #[test]
    fn restart_policy_strings() {
        assert_eq!(RestartPolicy::Always.as_str(),    "always");
        assert_eq!(RestartPolicy::OnFailure.as_str(), "on-failure");
        assert_eq!(RestartPolicy::No.as_str(),        "no");
    }
}
