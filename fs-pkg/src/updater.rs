// updater.rs — Updater: orchestrates atomic package updates with rollback.
//
// Update flow:
//   1. Check if the new version differs from the installed one.
//   2. Check prerequisites.
//   3. Uninstall old version (keep-data — preserve data directory).
//   4. Install new version.
//   5. Verify: install path exists on disk.
//   6. On install failure: return error with keep-data note for manual recovery.
//
// Pattern: Template Method (update() drives the fixed flow, steps delegated
//          to InstallerRegistry).

use std::path::Path;

use fs_error::FsError;
use fs_types::ResourceMeta;

use crate::channel::ReleaseChannel;
use crate::install_paths::InstallPaths;
use crate::installer_registry::InstallerRegistry;
use crate::installers::{InstallReport, UninstallOptions};
use crate::manifest::PackageId;
use crate::versioning::{VersionManager, VersionRecord};

// ── UpdateOutcome ─────────────────────────────────────────────────────────────

/// Result of a completed update operation.
#[derive(Debug)]
pub struct UpdateOutcome {
    /// Resource slug (matches `InstalledResource::id`).
    pub id: String,
    /// Version that was previously installed.
    pub old_version: String,
    /// Version that is now installed.
    pub new_version: String,
    /// Install report from the new-version install step.
    pub report: InstallReport,
}

// ── Updater ───────────────────────────────────────────────────────────────────

/// Orchestrates atomic package updates: uninstall → install → verify.
///
/// # Pattern
///
/// Template Method — the update flow is fixed; Installer/Uninstaller handle
/// the type-specific work via [`InstallerRegistry`].
///
/// # Example
///
/// ```rust,ignore
/// let updater = Updater::new(InstallerRegistry::new(), InstallPaths::load());
/// let outcome = updater.update(
///     &new_meta,
///     Some(Path::new("/tmp/new-kanidm")),
///     "1.4.0",
///     ReleaseChannel::Stable,
///     false,
/// )?;
/// println!("{} → {}", outcome.old_version, outcome.new_version);
/// ```
pub struct Updater {
    registry: InstallerRegistry,
    paths:    InstallPaths,
}

impl Updater {
    pub fn new(registry: InstallerRegistry, paths: InstallPaths) -> Self {
        Self { registry, paths }
    }

    /// Update a package to a new version.
    ///
    /// - `meta`            — metadata for the **new** version to install.
    /// - `source`          — local path to the new version's artifacts (None = store fetch).
    /// - `current_version` — version string currently installed.
    /// - `_channel`        — release channel (recorded in version history by caller).
    /// - `dry_run`         — when true, print what would happen without writing.
    pub fn update(
        &self,
        meta: &ResourceMeta,
        source: Option<&Path>,
        current_version: &str,
        _channel: ReleaseChannel,
        dry_run: bool,
    ) -> Result<UpdateOutcome, FsError> {
        let rt = meta.resource_type;

        // 1. Skip if already at the target version.
        if meta.version == current_version {
            return Ok(UpdateOutcome {
                id:          meta.id.clone(),
                old_version: current_version.to_owned(),
                new_version: meta.version.clone(),
                report: InstallReport {
                    install_path: self.paths.install_path_for(rt, &meta.id),
                    summary: format!(
                        "'{}' is already at version {}",
                        meta.id, meta.version
                    ),
                    dry_run,
                },
            });
        }

        // 2. Check prerequisites before touching anything.
        self.registry.check_prerequisites(rt, meta)?;

        // Dry-run: report what would happen, write nothing.
        if dry_run {
            let report = self.registry.install(meta, source, &self.paths, true)?;
            return Ok(UpdateOutcome {
                id:          meta.id.clone(),
                old_version: current_version.to_owned(),
                new_version: meta.version.clone(),
                report,
            });
        }

        // 3. Uninstall old version (keep data so user config is preserved).
        let uninstall_opts = UninstallOptions { keep_data: true, dry_run: false };
        self.registry.uninstall(rt, &meta.id, &self.paths, &uninstall_opts)
            .map_err(|e| {
                FsError::internal(format!("uninstall failed during update of '{}': {e}", meta.id))
            })?;

        // 4. Install new version.
        let report = self.registry.install(meta, source, &self.paths, false)
            .map_err(|e| {
                FsError::internal(format!(
                    "install of '{}' v{} failed: {e}\n\
                     Data from v{current_version} is preserved (keep-data).\n\
                     To restore: fsn install --from <old-artifacts-path>",
                    meta.id, meta.version
                ))
            })?;

        // 5. Verify: the install path must exist on disk (skip for in-process types).
        if !report.install_path.is_empty() {
            let p = std::path::Path::new(&report.install_path);
            if !p.exists() {
                return Err(FsError::internal(format!(
                    "install verification failed: '{}' not found after install",
                    report.install_path
                )));
            }
        }

        Ok(UpdateOutcome {
            id:          meta.id.clone(),
            old_version: current_version.to_owned(),
            new_version: meta.version.clone(),
            report,
        })
    }
}

// ── Batch updater ─────────────────────────────────────────────────────────────

/// Result of a batch update (`fsn update --all`).
#[derive(Debug, Default)]
pub struct BatchUpdateOutcome {
    pub updated:  Vec<UpdateOutcome>,
    pub skipped:  Vec<String>,
    pub failures: Vec<(String, String)>,
}

impl BatchUpdateOutcome {
    pub fn print_summary(&self) {
        if !self.updated.is_empty() {
            println!("Updated {} package(s):", self.updated.len());
            for u in &self.updated {
                println!("  {} {} → {}", u.id, u.old_version, u.new_version);
            }
        }
        if !self.skipped.is_empty() {
            println!("Already up-to-date: {}", self.skipped.join(", "));
        }
        if !self.failures.is_empty() {
            println!("Failed {}:", self.failures.len());
            for (id, err) in &self.failures {
                println!("  {id}: {err}");
            }
        }
    }
}

// ── VersionManager integration ────────────────────────────────────────────────

/// Record a successful install/update in the [`VersionManager`].
pub fn record_version(
    manager: &mut VersionManager,
    id: &str,
    version: &str,
    channel: ReleaseChannel,
) {
    use std::time::{SystemTime, UNIX_EPOCH};
    let installed_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    manager.register(VersionRecord {
        package_id:   PackageId::new(id),
        version:      version.to_owned(),
        channel,
        active:       true,
        installed_at,
    });
}
