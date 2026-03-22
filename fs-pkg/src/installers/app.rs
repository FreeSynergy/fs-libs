// installers/app.rs — Installer + Uninstaller for App resources.
//
// App = native FreeSynergy binary (fsn, fsd, …).
// Install path: {system_base}/apps/{name}/
//
// Prerequisites: none (no container runtime needed).
// Post-install: set executable bit on the binary.

use std::path::Path;

use fs_error::FsError;
use fs_types::{ResourceMeta, ResourceType};

use crate::install_paths::InstallPaths;
use crate::installers::{
    copy_source_to_dir, remove_path, InstallReport, Installer, UninstallOptions, Uninstaller,
};

// ── AppInstaller ──────────────────────────────────────────────────────────────

/// Installs and removes native FreeSynergy App resources.
pub struct AppInstaller;

impl Installer for AppInstaller {
    fn resource_type(&self) -> ResourceType {
        ResourceType::App
    }

    fn check_prerequisites(&self, _meta: &ResourceMeta) -> Result<(), FsError> {
        // Native apps have no runtime prerequisites.
        Ok(())
    }

    fn install(
        &self,
        meta: &ResourceMeta,
        source: Option<&Path>,
        paths: &InstallPaths,
        dry_run: bool,
    ) -> Result<InstallReport, FsError> {
        let dest = paths
            .dir_for(ResourceType::App, &meta.id)
            .expect("AppInstaller: dir_for returned None");

        if dry_run {
            return Ok(InstallReport {
                install_path: dest.to_string_lossy().into_owned(),
                summary: format!(
                    "[dry-run] would install app '{}' to {}",
                    meta.id,
                    dest.display()
                ),
                dry_run: true,
            });
        }

        if let Some(src) = source {
            copy_source_to_dir(src, &dest)?;
            // Set executable bit on anything that looks like a binary.
            set_executables_in_dir(&dest);
        } else {
            // No source yet — create the directory as a placeholder.
            std::fs::create_dir_all(&dest).map_err(|e| {
                FsError::internal(format!("cannot create {}: {e}", dest.display()))
            })?;
        }

        Ok(InstallReport {
            install_path: dest.to_string_lossy().into_owned(),
            summary:      format!("installed app '{}' to {}", meta.id, dest.display()),
            dry_run:      false,
        })
    }
}

impl Uninstaller for AppInstaller {
    fn resource_type(&self) -> ResourceType {
        ResourceType::App
    }

    fn uninstall(
        &self,
        name: &str,
        paths: &InstallPaths,
        opts: &UninstallOptions,
    ) -> Result<(), FsError> {
        let dest = paths
            .dir_for(ResourceType::App, name)
            .expect("AppInstaller: dir_for returned None");

        if opts.dry_run {
            println!("[dry-run] would remove {}", dest.display());
            return Ok(());
        }
        if !opts.keep_data {
            remove_path(&dest, opts.dry_run)?;
        } else {
            println!("keep-data: skipping removal of {}", dest.display());
        }
        Ok(())
    }
}

// ── helpers ───────────────────────────────────────────────────────────────────

/// Set the executable bit on all files in `dir` that have no extension
/// (heuristic: executables are typically extension-free on Linux).
#[cfg(unix)]
fn set_executables_in_dir(dir: &std::path::PathBuf) {
    use std::os::unix::fs::PermissionsExt;
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() && path.extension().is_none() {
            if let Ok(meta) = path.metadata() {
                let mut perm = meta.permissions();
                perm.set_mode(perm.mode() | 0o111);
                let _ = std::fs::set_permissions(&path, perm);
            }
        }
    }
}

#[cfg(not(unix))]
fn set_executables_in_dir(_dir: &std::path::PathBuf) {}
