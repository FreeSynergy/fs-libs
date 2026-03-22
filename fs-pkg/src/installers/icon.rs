// installers/icon.rs — Installer + Uninstaller for IconSet and CursorSet resources.
//
// IconSet   install path: {icon_base}/{name}/
// CursorSet install path: {cursor_base}/{name}/
//
// Post-install IconSet: gtk-update-icon-cache (optional, warns if missing).
// Post-install CursorSet: no system command needed.

use std::path::Path;

use fs_error::FsError;
use fs_types::{ResourceMeta, ResourceType};

use crate::install_paths::InstallPaths;
use crate::installers::{
    copy_source_to_dir, remove_path, run_system_cmd, InstallReport, Installer, UninstallOptions,
    Uninstaller,
};

// ── IconInstaller (IconSet + CursorSet) ───────────────────────────────────────

/// Installs and removes IconSet and CursorSet resources.
pub struct IconInstaller {
    pub rt: ResourceType,
}

impl IconInstaller {
    pub fn for_icons()   -> Self { Self { rt: ResourceType::IconSet } }
    pub fn for_cursors() -> Self { Self { rt: ResourceType::CursorSet } }
}

impl Installer for IconInstaller {
    fn resource_type(&self) -> ResourceType {
        self.rt
    }

    fn check_prerequisites(&self, _meta: &ResourceMeta) -> Result<(), FsError> {
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
            .dir_for(self.rt, &meta.id)
            .ok_or_else(|| FsError::internal("IconInstaller: dir_for returned None"))?;

        if dry_run {
            return Ok(InstallReport {
                install_path: dest.to_string_lossy().into_owned(),
                summary: format!(
                    "[dry-run] would install {} '{}' to {}",
                    self.rt.label(), meta.id, dest.display()
                ),
                dry_run: true,
            });
        }

        if let Some(src) = source {
            copy_source_to_dir(src, &dest)?;
        } else {
            std::fs::create_dir_all(&dest).map_err(|e| {
                FsError::internal(format!("cannot create {}: {e}", dest.display()))
            })?;
        }

        // Only IconSet needs cache refresh.
        if self.rt == ResourceType::IconSet {
            run_system_cmd("gtk-update-icon-cache", &["-f", "-t", dest.to_str().unwrap_or("")]);
        }

        Ok(InstallReport {
            install_path: dest.to_string_lossy().into_owned(),
            summary: format!("installed {} '{}' to {}", self.rt.label(), meta.id, dest.display()),
            dry_run: false,
        })
    }
}

impl Uninstaller for IconInstaller {
    fn resource_type(&self) -> ResourceType {
        self.rt
    }

    fn uninstall(&self, name: &str, paths: &InstallPaths, opts: &UninstallOptions) -> Result<(), FsError> {
        let dest = paths
            .dir_for(self.rt, name)
            .ok_or_else(|| FsError::internal("IconInstaller: dir_for returned None"))?;

        if opts.dry_run {
            println!("[dry-run] would remove {}", dest.display());
            return Ok(());
        }
        remove_path(&dest, opts.dry_run)?;
        if self.rt == ResourceType::IconSet {
            // Parent directory is the icons dir — refresh its cache.
            if let Some(parent) = dest.parent() {
                run_system_cmd("gtk-update-icon-cache", &["-f", "-t", parent.to_str().unwrap_or("")]);
            }
        }
        Ok(())
    }
}
