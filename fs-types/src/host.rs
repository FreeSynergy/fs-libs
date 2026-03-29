//! Host-related types: operating mode and runtime status.

use serde::{Deserialize, Serialize};

// ── HostMode ──────────────────────────────────────────────────────────────────

/// How a host is managed by this `FreeSynergy` node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum HostMode {
    /// The local machine on which this FSN binary is running.
    #[default]
    Local,
    /// A remote machine managed via SSH.
    Remote,
    /// A machine fully managed and provisioned by FSN (Ansible + Quadlets).
    Managed,
}

impl HostMode {
    /// Human-readable label for UI display.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            HostMode::Local => "Local",
            HostMode::Remote => "Remote",
            HostMode::Managed => "Managed",
        }
    }

    /// i18n key.
    #[must_use]
    pub fn i18n_key(self) -> &'static str {
        match self {
            HostMode::Local => "host.mode.local",
            HostMode::Remote => "host.mode.remote",
            HostMode::Managed => "host.mode.managed",
        }
    }

    /// `true` when SSH connectivity is required to reach this host.
    #[must_use]
    pub fn requires_ssh(self) -> bool {
        matches!(self, HostMode::Remote | HostMode::Managed)
    }
}

// ── HostStatus ────────────────────────────────────────────────────────────────

/// Runtime reachability and health status of a host.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum HostStatus {
    /// Host is reachable and all health checks pass.
    Online,
    /// Host is not reachable (network or SSH failure).
    Offline,
    /// Host is reachable but some services are degraded.
    Degraded,
    /// Status has not been checked yet or the last check timed out.
    #[default]
    Unknown,
}

impl HostStatus {
    /// Human-readable label for UI display.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            HostStatus::Online => "Online",
            HostStatus::Offline => "Offline",
            HostStatus::Degraded => "Degraded",
            HostStatus::Unknown => "Unknown",
        }
    }

    /// i18n key.
    #[must_use]
    pub fn i18n_key(self) -> &'static str {
        match self {
            HostStatus::Online => "host.status.online",
            HostStatus::Offline => "host.status.offline",
            HostStatus::Degraded => "host.status.degraded",
            HostStatus::Unknown => "host.status.unknown",
        }
    }

    /// `true` when the host can accept new deployments.
    #[must_use]
    pub fn is_available(self) -> bool {
        matches!(self, HostStatus::Online | HostStatus::Degraded)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_mode_requires_ssh() {
        assert!(!HostMode::Local.requires_ssh());
        assert!(HostMode::Remote.requires_ssh());
        assert!(HostMode::Managed.requires_ssh());
    }

    #[test]
    fn host_status_is_available() {
        assert!(HostStatus::Online.is_available());
        assert!(HostStatus::Degraded.is_available());
        assert!(!HostStatus::Offline.is_available());
        assert!(!HostStatus::Unknown.is_available());
    }

    #[test]
    fn host_mode_default_is_local() {
        assert_eq!(HostMode::default(), HostMode::Local);
    }

    #[test]
    fn host_status_default_is_unknown() {
        assert_eq!(HostStatus::default(), HostStatus::Unknown);
    }

    #[test]
    fn host_mode_labels() {
        assert_eq!(HostMode::Managed.label(), "Managed");
    }

    #[test]
    fn host_status_labels() {
        assert_eq!(HostStatus::Offline.label(), "Offline");
    }
}
