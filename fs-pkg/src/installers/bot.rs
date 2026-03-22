// installers/bot.rs — Installer + Uninstaller for Bot resources.
//
// Bot = automation agent that connects to messaging channels.
// Install path: {system_base}/bots/{name}/
//
// Prerequisites: none.
// Post-install: set executable bit on the binary.

use std::path::Path;

use fs_error::FsError;
use fs_types::{ResourceMeta, ResourceType};

use crate::install_paths::InstallPaths;
use crate::installers::{
    copy_source_to_dir, remove_path, InstallReport, Installer, UninstallOptions, Uninstaller,
};

// ── BotInstaller ──────────────────────────────────────────────────────────────

/// Installs and removes Bot resources.
pub struct BotInstaller;

impl Installer for BotInstaller {
    fn resource_type(&self) -> ResourceType {
        ResourceType::Bot
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
            .dir_for(ResourceType::Bot, &meta.id)
            .expect("BotInstaller: dir_for returned None");

        if dry_run {
            return Ok(InstallReport {
                install_path: dest.to_string_lossy().into_owned(),
                summary: format!(
                    "[dry-run] would install bot '{}' to {}",
                    meta.id,
                    dest.display()
                ),
                dry_run: true,
            });
        }

        if let Some(src) = source {
            copy_source_to_dir(src, &dest)?;
            set_executables_in_dir(&dest);
        } else {
            std::fs::create_dir_all(&dest).map_err(|e| {
                FsError::internal(format!("cannot create {}: {e}", dest.display()))
            })?;
        }

        Ok(InstallReport {
            install_path: dest.to_string_lossy().into_owned(),
            summary:      format!("installed bot '{}' to {}", meta.id, dest.display()),
            dry_run:      false,
        })
    }
}

impl Uninstaller for BotInstaller {
    fn resource_type(&self) -> ResourceType {
        ResourceType::Bot
    }

    fn uninstall(
        &self,
        name: &str,
        paths: &InstallPaths,
        opts: &UninstallOptions,
    ) -> Result<(), FsError> {
        let dest = paths
            .dir_for(ResourceType::Bot, name)
            .expect("BotInstaller: dir_for returned None");

        if opts.dry_run {
            println!("[dry-run] would remove {}", dest.display());
            return Ok(());
        }
        remove_path(&dest, opts.dry_run)
    }
}

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
