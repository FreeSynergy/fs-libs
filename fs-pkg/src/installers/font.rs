// installers/font.rs — Installer + Uninstaller for FontSet resources.
//
// FontSet = font face files + CSS @font-face declarations.
// Install path: {font_base}/{name}/
//
// Prerequisites: fc-cache (optional — warns if missing).
// Post-install: fc-cache -f to refresh the font cache.
// Post-remove:  fc-cache -f to update cache after removal.

use std::path::Path;

use fs_error::FsError;
use fs_types::{ResourceMeta, ResourceType};

use crate::install_paths::InstallPaths;
use crate::installers::{
    copy_source_to_dir, remove_path, run_system_cmd, InstallReport, Installer, UninstallOptions,
    Uninstaller,
};

pub struct FontInstaller;

impl Installer for FontInstaller {
    fn resource_type(&self) -> ResourceType {
        ResourceType::FontSet
    }

    fn check_prerequisites(&self, _meta: &ResourceMeta) -> Result<(), FsError> {
        // fc-cache is optional — warn at install time, not here.
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
            .dir_for(ResourceType::FontSet, &meta.id)
            .expect("FontInstaller: dir_for returned None");

        if dry_run {
            return Ok(InstallReport {
                install_path: dest.to_string_lossy().into_owned(),
                summary: format!(
                    "[dry-run] would install font set '{}' to {} (then run fc-cache -f)",
                    meta.id,
                    dest.display()
                ),
                dry_run: true,
            });
        }

        if let Some(src) = source {
            copy_source_to_dir(src, &dest)?;
        } else {
            std::fs::create_dir_all(&dest)
                .map_err(|e| FsError::internal(format!("cannot create {}: {e}", dest.display())))?;
        }

        // Refresh the system font cache — non-fatal if fc-cache is unavailable.
        run_system_cmd("fc-cache", &["-f"]);

        Ok(InstallReport {
            install_path: dest.to_string_lossy().into_owned(),
            summary: format!(
                "installed font set '{}' to {} (fc-cache refreshed)",
                meta.id,
                dest.display()
            ),
            dry_run: false,
        })
    }
}

impl Uninstaller for FontInstaller {
    fn resource_type(&self) -> ResourceType {
        ResourceType::FontSet
    }

    fn uninstall(
        &self,
        name: &str,
        paths: &InstallPaths,
        opts: &UninstallOptions,
    ) -> Result<(), FsError> {
        let dest = paths
            .dir_for(ResourceType::FontSet, name)
            .expect("FontInstaller: dir_for returned None");

        if opts.dry_run {
            println!(
                "[dry-run] would remove {} (then run fc-cache -f)",
                dest.display()
            );
            return Ok(());
        }
        remove_path(&dest, opts.dry_run)?;
        run_system_cmd("fc-cache", &["-f"]);
        Ok(())
    }
}
