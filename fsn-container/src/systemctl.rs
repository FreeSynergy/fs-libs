// systemctl async wrapper for FreeSynergy container management.
//
// Wraps `systemctl` and exposes typed results.
// Always operates in `--user` mode (rootless Podman).

use std::fmt;

use serde::{Deserialize, Serialize};

use fsn_error::FsnError;

// ── UnitActiveState ───────────────────────────────────────────────────────────

/// Active state of a systemd unit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnitActiveState {
    /// Unit is running.
    Active,
    /// Unit is not running.
    Inactive,
    /// Unit is starting.
    Activating,
    /// Unit is stopping.
    Deactivating,
    /// Unit entered a failed state.
    Failed,
    /// State could not be determined.
    Unknown,
}

impl fmt::Display for UnitActiveState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Active       => "active",
            Self::Inactive     => "inactive",
            Self::Activating   => "activating",
            Self::Deactivating => "deactivating",
            Self::Failed       => "failed",
            Self::Unknown      => "unknown",
        };
        f.write_str(s)
    }
}

// ── ServiceStatus ─────────────────────────────────────────────────────────────

/// Runtime status snapshot of a managed service.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStatus {
    /// Unit file name, e.g. `"fsn-zentinel.service"`.
    pub name: String,
    /// High-level active state.
    pub active_state: UnitActiveState,
    /// Low-level sub-state (e.g. `"running"`, `"dead"`).
    pub sub_state: String,
    /// Human-readable description from the unit file.
    pub description: String,
}

impl ServiceStatus {
    /// `true` when the service is actively running.
    pub fn is_running(&self) -> bool {
        self.active_state == UnitActiveState::Active && self.sub_state == "running"
    }

    /// `true` when the service has failed.
    pub fn is_failed(&self) -> bool {
        self.active_state == UnitActiveState::Failed
    }
}

// ── SystemctlManager ─────────────────────────────────────────────────────────

/// Async wrapper around `systemctl --user`.
pub struct SystemctlManager {
    user_mode: bool,
}

impl SystemctlManager {
    /// Create a manager in user mode (`systemctl --user`).
    pub fn user() -> Self {
        Self { user_mode: true }
    }

    /// Create a manager in system mode (no `--user` flag).
    pub fn system() -> Self {
        Self { user_mode: false }
    }

    // ── Public API ────────────────────────────────────────────────────────────

    /// Query the runtime status of a unit.
    pub async fn service_status(&self, unit: &str) -> Result<ServiceStatus, FsnError> {
        let out = self
            .run(&["show", unit, "--property=ActiveState,SubState,Description", "--value"])
            .await?;
        let mut lines = out.lines();
        let active_raw  = lines.next().unwrap_or("").trim().to_string();
        let sub_raw     = lines.next().unwrap_or("").trim().to_string();
        let description = lines.next().unwrap_or("").trim().to_string();

        let active_state = match active_raw.as_str() {
            "active"       => UnitActiveState::Active,
            "inactive"     => UnitActiveState::Inactive,
            "activating"   => UnitActiveState::Activating,
            "deactivating" => UnitActiveState::Deactivating,
            "failed"       => UnitActiveState::Failed,
            _              => UnitActiveState::Unknown,
        };

        Ok(ServiceStatus { name: unit.to_string(), active_state, sub_state: sub_raw, description })
    }

    /// Start a unit.
    pub async fn start(&self, unit: &str) -> Result<(), FsnError> {
        self.run(&["start", unit]).await.map(|_| ())
    }

    /// Stop a unit.
    pub async fn stop(&self, unit: &str) -> Result<(), FsnError> {
        self.run(&["stop", unit]).await.map(|_| ())
    }

    /// Restart a unit.
    pub async fn restart(&self, unit: &str) -> Result<(), FsnError> {
        self.run(&["restart", unit]).await.map(|_| ())
    }

    /// Enable a unit.
    pub async fn enable(&self, unit: &str) -> Result<(), FsnError> {
        self.run(&["enable", unit]).await.map(|_| ())
    }

    /// Disable a unit.
    pub async fn disable(&self, unit: &str) -> Result<(), FsnError> {
        self.run(&["disable", unit]).await.map(|_| ())
    }

    /// Reload the systemd daemon (required after writing new unit files).
    pub async fn daemon_reload(&self) -> Result<(), FsnError> {
        self.run(&["daemon-reload"]).await.map(|_| ())
    }

    /// Return `true` when the unit is in the `active` state.
    pub async fn is_active(&self, unit: &str) -> Result<bool, FsnError> {
        let output = self.raw(&["is-active", unit]).await?;
        Ok(output.status.success())
    }

    // ── Private helpers ───────────────────────────────────────────────────────

    async fn run(&self, args: &[&str]) -> Result<String, FsnError> {
        let output = self.raw(args).await?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(FsnError::internal(format!(
                "systemctl {}: {stderr}",
                args.join(" ")
            )));
        }
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }

    async fn raw(&self, args: &[&str]) -> Result<std::process::Output, FsnError> {
        let mut full: Vec<&str> = Vec::new();
        if self.user_mode {
            full.push("--user");
        }
        full.extend_from_slice(args);

        tokio::process::Command::new("systemctl")
            .args(&full)
            .output()
            .await
            .map_err(|e| FsnError::internal(format!("systemctl subprocess: {e}")))
    }
}

impl Default for SystemctlManager {
    fn default() -> Self {
        Self::user()
    }
}
