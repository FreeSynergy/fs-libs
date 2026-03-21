// versioning.rs — VersionManager: tracks installed versions, enables rollback.
//
// Multiple versions of a package can coexist on disk; exactly one is "active".
// Rolling back switches the active version to a previously installed one.
//
// Design:
//   VersionRecord   — one installed snapshot (package + version + channel)
//   VersionManager  — list / activate / rollback / prune old versions

use crate::channel::ReleaseChannel;
use serde::{Deserialize, Serialize};

/// A single installed version snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionRecord {
    /// Package identifier.
    pub package_id: String,
    /// Semver version string.
    pub version: String,
    /// Release channel this version came from.
    pub channel: ReleaseChannel,
    /// Whether this is the currently active version.
    pub active: bool,
    /// Unix timestamp of installation.
    pub installed_at: i64,
}

/// Manages version history for installed packages (in-memory; persisted via fsn-db).
///
/// The caller is responsible for persisting changes using [`fsn_db::InstalledPackageRepo`].
#[derive(Debug, Default)]
pub struct VersionManager {
    records: Vec<VersionRecord>,
}

impl VersionManager {
    /// Create an empty manager.
    pub fn new() -> Self {
        Self::default()
    }

    /// Load from a list of records (e.g. read from DB).
    pub fn from_records(records: Vec<VersionRecord>) -> Self {
        Self { records }
    }

    /// All recorded versions for a given package, newest first.
    pub fn versions_of(&self, package_id: &str) -> Vec<&VersionRecord> {
        let mut v: Vec<&VersionRecord> = self
            .records
            .iter()
            .filter(|r| r.package_id == package_id)
            .collect();
        v.sort_by(|a, b| b.installed_at.cmp(&a.installed_at));
        v
    }

    /// Currently active version for a package.
    pub fn active_version(&self, package_id: &str) -> Option<&VersionRecord> {
        self.records
            .iter()
            .find(|r| r.package_id == package_id && r.active)
    }

    /// Register a new installation. Marks it active, deactivates any previous active version.
    pub fn register(&mut self, record: VersionRecord) {
        // Deactivate previous active
        for r in &mut self.records {
            if r.package_id == record.package_id && r.active {
                r.active = false;
            }
        }
        self.records.push(record);
    }

    /// Roll back to a specific version string. Returns `Err` if the version isn't found.
    pub fn rollback(&mut self, package_id: &str, target_version: &str) -> Result<(), RollbackError> {
        let target_exists = self.records.iter().any(|r| {
            r.package_id == package_id && r.version == target_version
        });
        if !target_exists {
            return Err(RollbackError::VersionNotFound {
                package_id: package_id.to_string(),
                version:    target_version.to_string(),
            });
        }

        for r in &mut self.records {
            if r.package_id == package_id {
                r.active = r.version == target_version;
            }
        }
        Ok(())
    }

    /// Roll back to the previous version (the one before the current active).
    pub fn rollback_one(&mut self, package_id: &str) -> Result<(), RollbackError> {
        let versions: Vec<String> = {
            let mut v: Vec<&VersionRecord> = self
                .records
                .iter()
                .filter(|r| r.package_id == package_id)
                .collect();
            v.sort_by(|a, b| b.installed_at.cmp(&a.installed_at));
            v.iter().map(|r| r.version.clone()).collect()
        };

        match versions.as_slice() {
            [] | [_] => Err(RollbackError::NoPreviousVersion { package_id: package_id.to_string() }),
            [_current, prev, ..] => self.rollback(package_id, prev),
        }
    }

    /// Remove all non-active versions beyond `keep` count (oldest first).
    pub fn prune(&mut self, package_id: &str, keep: usize) {
        let mut inactive: Vec<usize> = self
            .records
            .iter()
            .enumerate()
            .filter(|(_, r)| r.package_id == package_id && !r.active)
            .map(|(i, _)| i)
            .collect();

        // Sort by installed_at ascending (oldest first)
        inactive.sort_by_key(|&i| self.records[i].installed_at);

        let to_remove = inactive.len().saturating_sub(keep);
        let mut removed = 0;
        self.records.retain(|r| {
            if r.package_id == package_id && !r.active && removed < to_remove {
                removed += 1;
                false
            } else {
                true
            }
        });
    }
}

/// Why a rollback operation failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RollbackError {
    /// The requested version is not in the version history.
    VersionNotFound { package_id: String, version: String },
    /// Only one version is installed — cannot roll back.
    NoPreviousVersion { package_id: String },
}

impl std::fmt::Display for RollbackError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::VersionNotFound { package_id, version } => {
                write!(f, "version '{version}' of '{package_id}' not found in history")
            }
            Self::NoPreviousVersion { package_id } => {
                write!(f, "no previous version for '{package_id}' to roll back to")
            }
        }
    }
}
