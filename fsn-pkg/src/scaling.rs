// scaling.rs — Worker and horizontal scaling manifest for FreeSynergy packages.
//
// Some services can run as workers (sharing load with other instances) or be
// scaled horizontally. This module defines the scaling manifest block and the
// multi-instance dialog logic.
//
// Design:
//   WorkerMode      — stateless | stateful | leader-follower
//   ScalingManifest — declares scaling capabilities of a package
//   InstanceRole    — role chosen for a new instance when one already exists
//   ScalingDialog   — decides what options the installer UI should offer
//
// TOML structure (in package manifest):
//   [package.scaling]
//   supports_workers             = true
//   supports_horizontal_scaling  = true
//   min_instances                = 1
//   max_instances                = 0    # 0 = unlimited
//   worker_mode                  = "stateless"

use serde::{Deserialize, Serialize};

// ── WorkerMode ────────────────────────────────────────────────────────────────

/// How a service handles distributed worker instances.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum WorkerMode {
    /// Workers are independent; any request can go to any instance.
    #[default]
    Stateless,
    /// Workers share state; session affinity may be required.
    Stateful,
    /// One instance is the leader; others are followers.
    LeaderFollower,
}

impl WorkerMode {
    /// Human-readable description.
    pub fn description(&self) -> &'static str {
        match self {
            Self::Stateless      => "Workers are stateless — any request can go to any instance",
            Self::Stateful       => "Workers share state — session affinity may be required",
            Self::LeaderFollower => "One leader, multiple followers — the leader coordinates writes",
        }
    }

    /// Short label for UI display.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Stateless      => "Stateless",
            Self::Stateful       => "Stateful",
            Self::LeaderFollower => "Leader-Follower",
        }
    }
}

// ── ScalingManifest ───────────────────────────────────────────────────────────

/// Scaling capabilities declared in `[package.scaling]`.
///
/// ```toml
/// [package.scaling]
/// supports_workers            = true
/// supports_horizontal_scaling = true
/// min_instances               = 1
/// max_instances               = 0
/// worker_mode                 = "stateless"
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingManifest {
    /// Can this service run as a worker alongside existing instances?
    #[serde(default)]
    pub supports_workers: bool,

    /// Can multiple instances run in parallel on different hosts?
    #[serde(default)]
    pub supports_horizontal_scaling: bool,

    /// Minimum number of instances required (usually 1).
    #[serde(default = "default_min_instances")]
    pub min_instances: u32,

    /// Maximum number of instances allowed (0 = unlimited).
    #[serde(default)]
    pub max_instances: u32,

    /// How worker instances share work and state.
    #[serde(default)]
    pub worker_mode: WorkerMode,
}

fn default_min_instances() -> u32 { 1 }

impl Default for ScalingManifest {
    fn default() -> Self {
        Self {
            supports_workers:            false,
            supports_horizontal_scaling: false,
            min_instances:               1,
            max_instances:               0,
            worker_mode:                 WorkerMode::Stateless,
        }
    }
}

impl ScalingManifest {
    /// Returns `true` if more than `current_instances` may be added.
    pub fn can_add_instance(&self, current_instances: u32) -> bool {
        if !self.supports_horizontal_scaling {
            return false;
        }
        if self.max_instances == 0 {
            return true;
        }
        current_instances < self.max_instances
    }

    /// Returns `true` if the instance count is within the declared bounds.
    pub fn is_count_valid(&self, count: u32) -> bool {
        if count < self.min_instances {
            return false;
        }
        if self.max_instances > 0 && count > self.max_instances {
            return false;
        }
        true
    }
}

// ── InstanceRole ──────────────────────────────────────────────────────────────

/// The role a new instance takes when the service already has a running instance.
///
/// This corresponds to the three choices shown in the multi-instance dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InstanceRole {
    /// Join as a load-sharing worker (requires `supports_workers = true`).
    Worker,
    /// Run as a completely independent instance.
    Standalone,
    /// Read-only mirror / backup replica.
    Mirror,
}

impl InstanceRole {
    /// Human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Worker     => "Worker (load sharing)",
            Self::Standalone => "Standalone (independent)",
            Self::Mirror     => "Mirror (read-only replica)",
        }
    }

    /// Short description shown in the installer UI.
    pub fn description(&self) -> &'static str {
        match self {
            Self::Worker     => "Shares the workload with existing instances",
            Self::Standalone => "Runs independently from other instances",
            Self::Mirror     => "Read-only backup; follows the primary instance",
        }
    }
}

// ── ScalingDialog ─────────────────────────────────────────────────────────────

/// Determines which instance roles are available in the multi-instance dialog.
///
/// The installer calls `available_roles()` when a service is added to a host
/// where it already runs on another host in the same project.
///
/// # Example
///
/// ```rust
/// use fsn_pkg::scaling::{ScalingManifest, ScalingDialog, WorkerMode};
///
/// let manifest = ScalingManifest {
///     supports_workers:            true,
///     supports_horizontal_scaling: true,
///     min_instances:               1,
///     max_instances:               0,
///     worker_mode:                 WorkerMode::Stateless,
/// };
///
/// let dialog = ScalingDialog::new(&manifest);
/// let roles = dialog.available_roles();
/// assert!(roles.contains(&fsn_pkg::scaling::InstanceRole::Worker));
/// assert!(roles.contains(&fsn_pkg::scaling::InstanceRole::Standalone));
/// ```
pub struct ScalingDialog<'a> {
    manifest: &'a ScalingManifest,
}

impl<'a> ScalingDialog<'a> {
    /// Create a new dialog for the given manifest.
    pub fn new(manifest: &'a ScalingManifest) -> Self {
        Self { manifest }
    }

    /// Returns the instance roles the user may choose from.
    ///
    /// `Standalone` is always available.
    /// `Worker` requires `supports_workers = true`.
    /// `Mirror` requires `supports_horizontal_scaling = true`.
    pub fn available_roles(&self) -> Vec<InstanceRole> {
        let mut roles = vec![InstanceRole::Standalone];
        if self.manifest.supports_workers {
            roles.insert(0, InstanceRole::Worker);
        }
        if self.manifest.supports_horizontal_scaling {
            roles.push(InstanceRole::Mirror);
        }
        roles
    }

    /// Returns a summary line describing what the package supports.
    ///
    /// Used in the installer UI hint line (e.g. "Forgejo supports: Worker ✓ Standalone ✓ Mirror ✗")
    pub fn support_summary(&self) -> String {
        format!(
            "Worker {} Standalone ✓ Mirror {}",
            if self.manifest.supports_workers { "✓" } else { "✗" },
            if self.manifest.supports_horizontal_scaling { "✓" } else { "✗" },
        )
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_scaling_no_support() {
        let m = ScalingManifest::default();
        assert!(!m.supports_workers);
        assert!(!m.supports_horizontal_scaling);
        assert_eq!(m.min_instances, 1);
        assert_eq!(m.max_instances, 0);
    }

    #[test]
    fn can_add_instance_unlimited() {
        let m = ScalingManifest {
            supports_horizontal_scaling: true,
            max_instances: 0,
            ..ScalingManifest::default()
        };
        assert!(m.can_add_instance(100));
    }

    #[test]
    fn can_add_instance_bounded() {
        let m = ScalingManifest {
            supports_horizontal_scaling: true,
            max_instances: 3,
            ..ScalingManifest::default()
        };
        assert!(m.can_add_instance(2));
        assert!(!m.can_add_instance(3));
    }

    #[test]
    fn can_add_instance_not_supported() {
        let m = ScalingManifest {
            supports_horizontal_scaling: false,
            ..ScalingManifest::default()
        };
        assert!(!m.can_add_instance(0));
    }

    #[test]
    fn is_count_valid() {
        let m = ScalingManifest {
            min_instances: 1,
            max_instances: 5,
            ..ScalingManifest::default()
        };
        assert!(!m.is_count_valid(0));
        assert!(m.is_count_valid(1));
        assert!(m.is_count_valid(5));
        assert!(!m.is_count_valid(6));
    }

    #[test]
    fn dialog_worker_and_standalone() {
        let m = ScalingManifest {
            supports_workers:            true,
            supports_horizontal_scaling: false,
            ..ScalingManifest::default()
        };
        let roles = ScalingDialog::new(&m).available_roles();
        assert!(roles.contains(&InstanceRole::Worker));
        assert!(roles.contains(&InstanceRole::Standalone));
        assert!(!roles.contains(&InstanceRole::Mirror));
    }

    #[test]
    fn dialog_all_roles() {
        let m = ScalingManifest {
            supports_workers:            true,
            supports_horizontal_scaling: true,
            ..ScalingManifest::default()
        };
        let roles = ScalingDialog::new(&m).available_roles();
        assert_eq!(roles.len(), 3);
    }

    #[test]
    fn dialog_standalone_only() {
        let m = ScalingManifest::default();
        let roles = ScalingDialog::new(&m).available_roles();
        assert_eq!(roles, vec![InstanceRole::Standalone]);
    }

    #[test]
    fn serde_roundtrip() {
        let m = ScalingManifest {
            supports_workers:            true,
            supports_horizontal_scaling: true,
            min_instances:               1,
            max_instances:               5,
            worker_mode:                 WorkerMode::LeaderFollower,
        };
        let toml_str = toml::to_string(&m).unwrap();
        let back: ScalingManifest = toml::from_str(&toml_str).unwrap();
        assert_eq!(back.worker_mode, WorkerMode::LeaderFollower);
        assert_eq!(back.max_instances, 5);
    }

    #[test]
    fn worker_mode_descriptions() {
        assert!(!WorkerMode::Stateless.description().is_empty());
        assert!(!WorkerMode::Stateful.description().is_empty());
        assert!(!WorkerMode::LeaderFollower.description().is_empty());
    }
}
