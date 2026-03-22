// ServiceConfig — declarative container service description.
//
// A `ServiceConfig` describes everything needed to generate a Podman Quadlet
// `.container` unit file.  No bollard, no socket — just plain data.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use fs_types::StrLabel;

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
    #[serde(default = "PortBinding::default_protocol")]
    pub protocol: String,
}

impl PortBinding {
    const DEFAULT_PROTOCOL: &'static str = "tcp";

    fn default_protocol() -> String {
        Self::DEFAULT_PROTOCOL.to_string()
    }

    /// Create a TCP port binding `host_port:container_port`.
    pub fn tcp(host_port: u16, container_port: u16) -> Self {
        Self { host_port, container_port, protocol: Self::DEFAULT_PROTOCOL.to_string() }
    }

    /// Create a UDP port binding `host_port:container_port`.
    pub fn udp(host_port: u16, container_port: u16) -> Self {
        Self { host_port, container_port, protocol: "udp".to_string() }
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

impl StrLabel for RestartPolicy {
    fn label(&self) -> &'static str { self.as_str() }
}

fs_types::impl_str_label_display!(RestartPolicy);

// ── HealthCheck ───────────────────────────────────────────────────────────────

/// Container health check configuration.
///
/// Every FSN service module **must** declare a health check.
///
/// # Example
///
/// ```rust
/// use fs_container::HealthCheck;
///
/// let hc = HealthCheck::new(["curl", "-fs", "http://localhost/health"]);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    /// Command to run inside the container, e.g. `["curl", "-fs", "http://localhost/health"]`.
    pub test: Vec<String>,
    /// Interval between checks (systemd time span, e.g. `"30s"`).
    #[serde(default = "HealthCheck::default_interval")]
    pub interval: String,
    /// Time after which a check is considered failed (e.g. `"10s"`).
    #[serde(default = "HealthCheck::default_timeout")]
    pub timeout: String,
    /// Number of consecutive failures before declaring unhealthy.
    #[serde(default = "HealthCheck::default_retries")]
    pub retries: u32,
    /// Grace period at startup before health checks begin (e.g. `"5s"`).
    #[serde(default = "HealthCheck::default_start_period")]
    pub start_period: String,
}

impl HealthCheck {
    // Single source of truth for all timing defaults.
    const DEFAULT_INTERVAL:     &'static str = "30s";
    const DEFAULT_TIMEOUT:      &'static str = "10s";
    const DEFAULT_RETRIES:      u32          = 3;
    const DEFAULT_START_PERIOD: &'static str = "5s";

    // Serde `default = "…"` requires a function path, not a const.
    fn default_interval()     -> String { Self::DEFAULT_INTERVAL.to_string() }
    fn default_timeout()      -> String { Self::DEFAULT_TIMEOUT.to_string() }
    fn default_retries()      -> u32    { Self::DEFAULT_RETRIES }
    fn default_start_period() -> String { Self::DEFAULT_START_PERIOD.to_string() }

    /// Create a health check with the given command and default timings.
    ///
    /// ```rust
    /// use fs_container::HealthCheck;
    ///
    /// let hc = HealthCheck::new(["curl", "-fs", "http://localhost/health"]);
    /// ```
    pub fn new(test: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self { test: test.into_iter().map(Into::into).collect(), ..Self::default() }
    }

    /// Override the check interval (systemd time span, e.g. `"60s"`).
    pub fn with_interval(mut self, interval: impl Into<String>) -> Self {
        self.interval = interval.into();
        self
    }

    /// Override the check timeout (systemd time span, e.g. `"5s"`).
    pub fn with_timeout(mut self, timeout: impl Into<String>) -> Self {
        self.timeout = timeout.into();
        self
    }

    /// Override the number of retries before declaring unhealthy.
    pub fn with_retries(mut self, retries: u32) -> Self {
        self.retries = retries;
        self
    }

    /// Override the startup grace period (systemd time span, e.g. `"10s"`).
    pub fn with_start_period(mut self, start_period: impl Into<String>) -> Self {
        self.start_period = start_period.into();
        self
    }

    /// Render as the Quadlet `HealthCmd=` line (space-separated command).
    pub fn to_quadlet_cmd(&self) -> String {
        self.test.join(" ")
    }
}

impl Default for HealthCheck {
    fn default() -> Self {
        Self {
            test: Vec::new(),
            interval: Self::DEFAULT_INTERVAL.to_string(),
            timeout: Self::DEFAULT_TIMEOUT.to_string(),
            retries: Self::DEFAULT_RETRIES,
            start_period: Self::DEFAULT_START_PERIOD.to_string(),
        }
    }
}

// ── ServiceConfig ─────────────────────────────────────────────────────────────

/// Declarative description of a container service.
///
/// Built via the fluent builder API starting from [`ServiceConfig::new`]:
///
/// ```rust,ignore
/// let svc = ServiceConfig::new("zentinel", "ghcr.io/freeSynergy/zentinel:latest")
///     .with_description("Zentinel reverse proxy")
///     .with_port(PortBinding::tcp(443, 443))
///     .with_volume(Volume::bind("/data/zentinel", "/data"))
///     .with_healthcheck(HealthCheck::new(["curl", "-fs", "http://localhost/health"]))
///     .with_env("LOG_LEVEL", "info");
/// ```
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
    #[serde(default = "ServiceConfig::default_network")]
    pub network: String,
    /// Optional container user override.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

impl ServiceConfig {
    const DEFAULT_NETWORK: &'static str = "fsn";

    fn default_network() -> String {
        Self::DEFAULT_NETWORK.to_string()
    }

    /// Create a minimal service config with required fields.
    ///
    /// All optional fields are populated with safe defaults. Use the
    /// `with_*` builder methods to configure additional settings.
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
            network: Self::DEFAULT_NETWORK.to_string(),
            user: None,
        }
    }

    /// Set the human-readable description for the `[Unit]` section.
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Add or overwrite an environment variable.
    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.environment.insert(key.into(), value.into());
        self
    }

    /// Add a volume mount.
    pub fn with_volume(mut self, volume: Volume) -> Self {
        self.volumes.push(volume);
        self
    }

    /// Add a published port.
    pub fn with_port(mut self, port: PortBinding) -> Self {
        self.ports.push(port);
        self
    }

    /// Add a container label.
    pub fn with_label(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.labels.insert(key.into(), value.into());
        self
    }

    /// Set the health check configuration.
    pub fn with_healthcheck(mut self, healthcheck: HealthCheck) -> Self {
        self.healthcheck = Some(healthcheck);
        self
    }

    /// Set the systemd restart policy.
    pub fn with_restart(mut self, policy: RestartPolicy) -> Self {
        self.restart_policy = policy;
        self
    }

    /// Set the Podman network name (default: `"fsn"`).
    pub fn with_network(mut self, network: impl Into<String>) -> Self {
        self.network = network.into();
        self
    }

    /// Set the container user override.
    pub fn with_user(mut self, user: impl Into<String>) -> Self {
        self.user = Some(user.into());
        self
    }

    /// The systemd unit name: `"fs-{name}.service"`.
    pub fn unit_name(&self) -> String {
        format!("fs-{}.service", self.name)
    }

    /// The Quadlet file name: `"fs-{name}.container"`.
    pub fn quadlet_filename(&self) -> String {
        format!("fs-{}.container", self.name)
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
        assert_eq!(svc.unit_name(), "fs-zentinel.service");
        assert_eq!(svc.quadlet_filename(), "fs-zentinel.container");
    }

    #[test]
    fn restart_policy_strings() {
        assert_eq!(RestartPolicy::Always.as_str(),    "always");
        assert_eq!(RestartPolicy::OnFailure.as_str(), "on-failure");
        assert_eq!(RestartPolicy::No.as_str(),        "no");
    }

    #[test]
    fn healthcheck_defaults() {
        let hc = HealthCheck::new(["curl", "-fs", "http://localhost/health"]);
        assert_eq!(hc.interval, "30s");
        assert_eq!(hc.timeout, "10s");
        assert_eq!(hc.retries, 3);
        assert_eq!(hc.start_period, "5s");
        assert_eq!(hc.to_quadlet_cmd(), "curl -fs http://localhost/health");
    }

    #[test]
    fn healthcheck_builder() {
        let hc = HealthCheck::new(["nc", "-z", "localhost", "5432"])
            .with_interval("60s")
            .with_retries(5);
        assert_eq!(hc.interval, "60s");
        assert_eq!(hc.retries, 5);
        assert_eq!(hc.timeout, "10s"); // unchanged default
    }

    #[test]
    fn service_config_builder() {
        let svc = ServiceConfig::new("myapp", "example.com/myapp:1.0")
            .with_description("My app")
            .with_env("LOG_LEVEL", "debug")
            .with_volume(Volume::bind("/data", "/data"))
            .with_port(PortBinding::tcp(8080, 8080))
            .with_restart(RestartPolicy::OnFailure);

        assert_eq!(svc.description.as_deref(), Some("My app"));
        assert_eq!(svc.environment.get("LOG_LEVEL").map(String::as_str), Some("debug"));
        assert_eq!(svc.volumes.len(), 1);
        assert_eq!(svc.ports.len(), 1);
        assert_eq!(svc.restart_policy.as_str(), "on-failure");
        assert_eq!(svc.network, "fsn"); // default unchanged
    }
}
