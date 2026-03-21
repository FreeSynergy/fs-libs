// QuadletManager — Podman Quadlet file generation + systemctl lifecycle.
//
// Manages `.container` unit files in `~/.config/containers/systemd/`.
// Never touches a Podman socket — all operations go through file generation
// and `systemctl --user` subprocess calls.
//
// Rollback strategy:
//   - Before writing a new Quadlet file, the existing file is backed up as `<name>.bak`.
//   - `rollback(name)` swaps the backup back and restarts the service.

use std::path::PathBuf;

use fs_error::FsError;

use crate::service::ServiceConfig;
use crate::systemctl::{ServiceStatus, SystemctlManager};

// ── QuadletManager ────────────────────────────────────────────────────────────

/// Manages Podman Quadlet `.container` unit files and their lifecycle.
///
/// # Usage
/// ```rust,ignore
/// use fs_container::{QuadletManager, ServiceConfig};
///
/// let mgr = QuadletManager::user_default();
/// let svc = ServiceConfig::new("zentinel", "ghcr.io/freeSynergy/zentinel:latest");
/// let path = mgr.create_quadlet(&svc).await?;
/// mgr.reload_daemon().await?;
/// mgr.start_service("zentinel").await?;
/// ```
pub struct QuadletManager {
    /// Directory where Quadlet `.container` files are stored.
    quadlet_dir: PathBuf,
    systemctl: SystemctlManager,
}

impl QuadletManager {
    /// Create a manager using the default user Quadlet directory:
    /// `$HOME/.config/containers/systemd/`.
    pub fn user_default() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
        let quadlet_dir = PathBuf::from(home).join(".config/containers/systemd");
        Self { quadlet_dir, systemctl: SystemctlManager::user() }
    }

    /// Create a manager with a custom Quadlet directory (useful for testing).
    pub fn with_dir(quadlet_dir: impl Into<PathBuf>) -> Self {
        Self {
            quadlet_dir: quadlet_dir.into(),
            systemctl: SystemctlManager::user(),
        }
    }

    // ── Quadlet file management ───────────────────────────────────────────────

    /// Generate a Quadlet `.container` file for `service`.
    ///
    /// If a file for this service already exists, it is backed up as `<name>.bak`
    /// before being overwritten (enables rollback).
    ///
    /// Returns the path to the written file.
    pub async fn create_quadlet(&self, service: &ServiceConfig) -> Result<PathBuf, FsError> {
        self.ensure_quadlet_dir().await?;

        let file_path = self.quadlet_dir.join(service.quadlet_filename());
        let bak_path  = self.quadlet_dir.join(format!("{}.bak", service.quadlet_filename()));

        // Backup existing file if present
        if file_path.exists() {
            tokio::fs::copy(&file_path, &bak_path)
                .await
                .map_err(|e| FsError::internal(format!("backup quadlet: {e}")))?;
        }

        let content = render_quadlet(service);
        tokio::fs::write(&file_path, content)
            .await
            .map_err(|e| FsError::internal(format!("write quadlet {}: {e}", file_path.display())))?;

        tracing::info!(service = %service.name, path = %file_path.display(), "Quadlet written");
        Ok(file_path)
    }

    /// Remove the Quadlet file for `name` (does not stop the service first).
    pub async fn remove_quadlet(&self, name: &str) -> Result<(), FsError> {
        let filename = format!("fs-{}.container", name);
        let file_path = self.quadlet_dir.join(&filename);
        if file_path.exists() {
            tokio::fs::remove_file(&file_path)
                .await
                .map_err(|e| FsError::internal(format!("remove quadlet {filename}: {e}")))?;
        }
        Ok(())
    }

    /// Restore the backup Quadlet file and restart the service.
    ///
    /// Call this when a newly deployed service is unhealthy.
    pub async fn rollback(&self, name: &str) -> Result<(), FsError> {
        let filename     = format!("fs-{}.container", name);
        let file_path    = self.quadlet_dir.join(&filename);
        let bak_path     = self.quadlet_dir.join(format!("{}.bak", filename));

        if !bak_path.exists() {
            return Err(FsError::not_found(format!("no backup found for service {name}")));
        }

        tokio::fs::copy(&bak_path, &file_path)
            .await
            .map_err(|e| FsError::internal(format!("restore quadlet from backup: {e}")))?;

        tracing::warn!(service = %name, "Rolling back to previous Quadlet");

        self.reload_daemon().await?;
        self.restart_service(name).await?;
        Ok(())
    }

    // ── systemctl wrappers ────────────────────────────────────────────────────

    /// Run `systemctl --user daemon-reload`.
    ///
    /// Must be called after creating or modifying Quadlet files.
    pub async fn reload_daemon(&self) -> Result<(), FsError> {
        self.systemctl.daemon_reload().await
    }

    /// Run `systemctl --user start fs-{name}.service`.
    pub async fn start_service(&self, name: &str) -> Result<(), FsError> {
        self.systemctl.start(&unit(name)).await
    }

    /// Run `systemctl --user stop fs-{name}.service`.
    pub async fn stop_service(&self, name: &str) -> Result<(), FsError> {
        self.systemctl.stop(&unit(name)).await
    }

    /// Run `systemctl --user restart fs-{name}.service`.
    pub async fn restart_service(&self, name: &str) -> Result<(), FsError> {
        self.systemctl.restart(&unit(name)).await
    }

    /// Query the runtime status of `fs-{name}.service`.
    pub async fn service_status(&self, name: &str) -> Result<ServiceStatus, FsError> {
        self.systemctl.service_status(&unit(name)).await
    }

    /// Retrieve the last `lines` log lines for `fs-{name}.service`.
    ///
    /// Runs `journalctl --user -u fs-{name}.service -n {lines} --no-pager`.
    pub async fn service_logs(&self, name: &str, lines: usize) -> Result<Vec<String>, FsError> {
        let unit_name = unit(name);
        let args = vec![
            "--user".to_string(),
            "-u".to_string(),
            unit_name,
            "-n".to_string(),
            lines.to_string(),
            "--no-pager".to_string(),
        ];

        let output = tokio::process::Command::new("journalctl")
            .args(&args)
            .output()
            .await
            .map_err(|e| FsError::internal(format!("journalctl subprocess: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(FsError::internal(format!("journalctl: {stderr}")));
        }

        let lines_out = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(str::to_string)
            .collect();
        Ok(lines_out)
    }

    // ── private helpers ───────────────────────────────────────────────────────

    async fn ensure_quadlet_dir(&self) -> Result<(), FsError> {
        tokio::fs::create_dir_all(&self.quadlet_dir)
            .await
            .map_err(|e| FsError::internal(format!("create quadlet dir: {e}")))
    }
}

// ── Quadlet file rendering ────────────────────────────────────────────────────

/// Render a complete Podman Quadlet `.container` file for `service`.
fn render_quadlet(svc: &ServiceConfig) -> String {
    let default_desc = format!("FSN Service: {}", svc.name);
    let desc = svc.description.as_deref().unwrap_or(&default_desc);

    let mut lines: Vec<String> = Vec::new();

    // [Unit]
    lines.push("[Unit]".into());
    lines.push(format!("Description={desc}"));
    lines.push("After=network-online.target".into());
    lines.push("Wants=network-online.target".into());
    lines.push(String::new());

    // [Container]
    lines.push("[Container]".into());
    lines.push(format!("Image={}", svc.image));
    lines.push(format!("ContainerName=fs-{}", svc.name));
    lines.push(format!("Network={}", svc.network));

    // Labels
    lines.push("Label=managed-by=freeSynergy".into());
    for (k, v) in &svc.labels {
        lines.push(format!("Label={k}={v}"));
    }

    // Environment (sorted for deterministic output)
    let mut env: Vec<(&String, &String)> = svc.environment.iter().collect();
    env.sort_by_key(|(k, _)| k.as_str());
    for (k, v) in env {
        lines.push(format!("Environment={k}={v}"));
    }

    // Volumes
    for vol in &svc.volumes {
        lines.push(format!("Volume={}", vol.to_quadlet_line()));
    }

    // Ports (Zentinel only)
    for port in &svc.ports {
        lines.push(format!("PublishPort={}", port.to_quadlet_line()));
    }

    // User
    if let Some(user) = &svc.user {
        lines.push(format!("User={user}"));
    }

    // Health check
    if let Some(hc) = &svc.healthcheck {
        lines.push(format!("HealthCmd={}", hc.to_quadlet_cmd()));
        lines.push(format!("HealthInterval={}", hc.interval));
        lines.push(format!("HealthTimeout={}", hc.timeout));
        lines.push(format!("HealthRetries={}", hc.retries));
        lines.push(format!("HealthStartPeriod={}", hc.start_period));
    }

    lines.push(String::new());

    // [Service]
    lines.push("[Service]".into());
    lines.push(format!("Restart={}", svc.restart_policy.as_str()));
    lines.push("TimeoutStartSec=300".into());
    lines.push(String::new());

    // [Install]
    lines.push("[Install]".into());
    lines.push("WantedBy=default.target".into());
    lines.push(String::new());

    lines.join("\n")
}

/// Format `"fs-{name}.service"`.
fn unit(name: &str) -> String {
    format!("fs-{}.service", name)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::{HealthCheck, PortBinding, ServiceConfig, Volume};

    fn zentinel_config() -> ServiceConfig {
        let mut svc = ServiceConfig::new("zentinel", "ghcr.io/freeSynergy/zentinel:latest");
        svc.description = Some("Zentinel Reverse Proxy".into());
        svc.ports = vec![PortBinding::tcp(443, 443), PortBinding::tcp(80, 80)];
        svc.volumes = vec![Volume::bind("/data/zentinel", "/data")];
        svc.environment.insert("LOG_LEVEL".into(), "info".into());
        svc.healthcheck = Some(HealthCheck {
            test: vec!["curl".into(), "-fs".into(), "http://localhost/health".into()],
            interval: "30s".into(),
            timeout: "10s".into(),
            retries: 3,
            start_period: "5s".into(),
        });
        svc
    }

    #[test]
    fn quadlet_contains_image() {
        let svc = zentinel_config();
        let out = render_quadlet(&svc);
        assert!(out.contains("Image=ghcr.io/freeSynergy/zentinel:latest"));
    }

    #[test]
    fn quadlet_contains_ports() {
        let svc = zentinel_config();
        let out = render_quadlet(&svc);
        assert!(out.contains("PublishPort=443:443/tcp"));
        assert!(out.contains("PublishPort=80:80/tcp"));
    }

    #[test]
    fn quadlet_contains_volume() {
        let svc = zentinel_config();
        let out = render_quadlet(&svc);
        assert!(out.contains("Volume=/data/zentinel:/data"));
    }

    #[test]
    fn quadlet_contains_env() {
        let svc = zentinel_config();
        let out = render_quadlet(&svc);
        assert!(out.contains("Environment=LOG_LEVEL=info"));
    }

    #[test]
    fn quadlet_contains_healthcheck() {
        let svc = zentinel_config();
        let out = render_quadlet(&svc);
        assert!(out.contains("HealthCmd=curl -fs http://localhost/health"));
    }

    #[test]
    fn quadlet_sections_present() {
        let svc = ServiceConfig::new("test", "image:latest");
        let out = render_quadlet(&svc);
        assert!(out.contains("[Unit]"));
        assert!(out.contains("[Container]"));
        assert!(out.contains("[Service]"));
        assert!(out.contains("[Install]"));
    }

    #[test]
    fn unit_name_format() {
        assert_eq!(unit("zentinel"), "fs-zentinel.service");
    }
}
