// installers/mod.rs — Installer and Uninstaller traits + shared types.
//
// Each ResourceType has a dedicated installer implementation (Strategy Pattern).
// The InstallerRegistry selects the right impl at runtime based on ResourceType.
//
// Install flow per impl:
//   check_prerequisites() → create_dir / create_parent_dirs → copy/write files
//   → post-install system commands (e.g. fc-cache) → return InstallReport
//
// Uninstall flow per impl:
//   compute_path → remove_dir_all / remove_file (unless keep_data)
//   → post-removal system commands

use std::path::{Path, PathBuf};

use fs_error::FsError;
use fs_types::{ResourceMeta, ResourceType};

use crate::install_paths::InstallPaths;

pub mod app;
pub mod bot;
pub mod bridge;
pub mod bundle;
pub mod container;
pub mod font;
pub mod icon;
pub mod theme;
pub mod widget;

// ── InstallReport ─────────────────────────────────────────────────────────────

/// What was written during an install operation.
///
/// The CLI layer uses this to create/update the `InstalledResource` inventory record.
#[derive(Debug, Clone)]
pub struct InstallReport {
    /// Absolute path where the resource was installed.
    /// Empty for in-process resources (Bridge, Bundle, Container).
    pub install_path: String,
    /// Human-readable summary of what was done (for progress output).
    pub summary: String,
    /// `true` if nothing was actually written (dry-run or no-op install).
    pub dry_run: bool,
}

// ── Installer trait ───────────────────────────────────────────────────────────

/// Install a resource of a specific type.
///
/// # Pattern
///
/// Strategy — one implementation per ResourceType or type group.
/// All implementations are registered in [`crate::installer_registry::InstallerRegistry`].
///
/// # Help texts
///
/// `check_prerequisites` error messages explain **what is missing** and
/// **how to fix it**, e.g.:
/// `"Podman not found — install with: dnf install podman"`
pub trait Installer: Send + Sync {
    /// Which ResourceType this installer handles.
    fn resource_type(&self) -> ResourceType;

    /// Check that all prerequisites for installation are satisfied.
    ///
    /// Returns `Ok(())` when ready.
    /// Returns `Err` with a human-readable message describing what is missing
    /// and how to resolve it.
    fn check_prerequisites(&self, meta: &ResourceMeta) -> Result<(), FsError>;

    /// Install the resource.
    ///
    /// Files are written to the path computed by
    /// `paths.install_path_for(meta.resource_type, &meta.id)`.
    ///
    /// With `dry_run = true`: validate, print what would happen, write nothing.
    fn install(
        &self,
        meta: &ResourceMeta,
        source: Option<&Path>,
        paths: &InstallPaths,
        dry_run: bool,
    ) -> Result<InstallReport, FsError>;
}

// ── Uninstaller trait ─────────────────────────────────────────────────────────

/// Uninstall a resource of a specific type.
///
/// Symmetric to [`Installer`] — the same struct typically implements both.
pub trait Uninstaller: Send + Sync {
    /// Which ResourceType this uninstaller handles.
    fn resource_type(&self) -> ResourceType;

    /// Remove all files for the given resource.
    ///
    /// `name` is the resource slug (matches `InstalledResource::id`).
    /// With `opts.keep_data = true`: preserve data directories, remove only binaries/config.
    fn uninstall(
        &self,
        name: &str,
        paths: &InstallPaths,
        opts: &UninstallOptions,
    ) -> Result<(), FsError>;
}

// ── UninstallOptions ──────────────────────────────────────────────────────────

/// Options for an uninstall operation.
#[derive(Debug, Clone, Default)]
pub struct UninstallOptions {
    /// Keep data directories; only remove binaries and config files.
    pub keep_data: bool,
    /// Perform a dry run: print what would be removed, write nothing.
    pub dry_run: bool,
}

// ── Shared helpers ────────────────────────────────────────────────────────────

/// Copy the contents of `source` into `dest_dir`.
///
/// If `source` is a file, copies that file into `dest_dir`.
/// If `source` is a directory, copies all its contents recursively.
pub(crate) fn copy_source_to_dir(source: &Path, dest_dir: &Path) -> Result<(), FsError> {
    std::fs::create_dir_all(dest_dir).map_err(|e| {
        FsError::internal(format!("cannot create {}: {e}", dest_dir.display()))
    })?;

    if source.is_file() {
        let dest = dest_dir.join(source.file_name().unwrap_or_default());
        std::fs::copy(source, &dest).map_err(|e| {
            FsError::internal(format!("copy {} → {}: {e}", source.display(), dest.display()))
        })?;
        return Ok(());
    }

    if source.is_dir() {
        copy_dir_recursive(source, dest_dir)?;
        return Ok(());
    }

    Err(FsError::internal(format!(
        "source is neither a file nor a directory: {}",
        source.display()
    )))
}

pub(crate) fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), FsError> {
    std::fs::create_dir_all(dst).map_err(|e| {
        FsError::internal(format!("cannot create {}: {e}", dst.display()))
    })?;
    for entry in std::fs::read_dir(src).map_err(|e| {
        FsError::internal(format!("cannot read dir {}: {e}", src.display()))
    })? {
        let entry = entry.map_err(|e| FsError::internal(format!("dir entry error: {e}")))?;
        let dest_path = dst.join(entry.file_name());
        if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            copy_dir_recursive(&entry.path(), &dest_path)?;
        } else {
            std::fs::copy(entry.path(), &dest_path).map_err(|e| {
                FsError::internal(format!(
                    "copy {} → {}: {e}",
                    entry.path().display(),
                    dest_path.display()
                ))
            })?;
        }
    }
    Ok(())
}

/// Run a system command for post-install/post-remove hooks (e.g. fc-cache, gtk-update-icon-cache).
/// Prints a warning but does not fail if the command is unavailable.
pub(crate) fn run_system_cmd(program: &str, args: &[&str]) {
    match std::process::Command::new(program).args(args).status() {
        Ok(s) if s.success() => {},
        Ok(s) => eprintln!("warning: {program} exited with {s}"),
        Err(e) => eprintln!("warning: {program} unavailable: {e}"),
    }
}

/// Check that a command-line tool is available in PATH.
/// Returns a user-facing error message if not found.
pub(crate) fn require_tool(name: &str, install_hint: &str) -> Result<(), FsError> {
    if which_available(name) {
        Ok(())
    } else {
        Err(FsError::internal(format!(
            "`{name}` not found in PATH — {install_hint}"
        )))
    }
}

fn which_available(name: &str) -> bool {
    std::process::Command::new("which")
        .arg(name)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Remove a path (file or directory) on uninstall, unless it doesn't exist.
pub(crate) fn remove_path(path: &PathBuf, dry_run: bool) -> Result<(), FsError> {
    if !path.exists() {
        return Ok(());
    }
    if dry_run {
        return Ok(());
    }
    if path.is_dir() {
        std::fs::remove_dir_all(path).map_err(|e| {
            FsError::internal(format!("cannot remove {}: {e}", path.display()))
        })
    } else {
        std::fs::remove_file(path).map_err(|e| {
            FsError::internal(format!("cannot remove {}: {e}", path.display()))
        })
    }
}
