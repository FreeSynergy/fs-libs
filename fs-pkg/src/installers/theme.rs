// installers/theme.rs — Installer + Uninstaller for flat-file theme resources.
//
// Handles: ColorScheme, Style, ButtonStyle, WindowChrome, AnimationSet, Task, Language.
// Install path: {config_base}/{subdir}/{name}.toml  (or /{name}/ for Language)
//
// Language is special: it installs into a directory, not a flat file.
// All other types here write a single TOML file.

use std::path::Path;

use fs_error::FsError;
use fs_types::{ResourceMeta, ResourceType};

use crate::install_paths::InstallPaths;
use crate::installers::{
    copy_source_to_dir, remove_path, InstallReport, Installer, UninstallOptions, Uninstaller,
};

// ── ThemeInstaller ────────────────────────────────────────────────────────────

/// Installs and removes flat-file theme/config resources (TOML files).
///
/// Handles: ColorScheme, Style, ButtonStyle, WindowChrome, AnimationSet, Task.
pub struct ThemeInstaller {
    pub rt: ResourceType,
}

impl Installer for ThemeInstaller {
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
        let dest_file = paths
            .config_file_for(self.rt, &meta.id)
            .ok_or_else(|| {
                FsError::internal(format!(
                    "ThemeInstaller: config_file_for returned None for {:?}",
                    self.rt
                ))
            })?;

        if dry_run {
            return Ok(InstallReport {
                install_path: dest_file.to_string_lossy().into_owned(),
                summary: format!(
                    "[dry-run] would install {} '{}' to {}",
                    self.rt.label(), meta.id, dest_file.display()
                ),
                dry_run: true,
            });
        }

        if let Some(parent) = dest_file.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                FsError::internal(format!("cannot create {}: {e}", parent.display()))
            })?;
        }

        if let Some(src) = source {
            std::fs::copy(src, &dest_file).map_err(|e| {
                FsError::internal(format!(
                    "copy {} → {}: {e}",
                    src.display(),
                    dest_file.display()
                ))
            })?;
        } else {
            // Write an empty placeholder so the resource is "installed".
            std::fs::write(&dest_file, b"# FreeSynergy resource placeholder\n").map_err(|e| {
                FsError::internal(format!("cannot write {}: {e}", dest_file.display()))
            })?;
        }

        Ok(InstallReport {
            install_path: dest_file.to_string_lossy().into_owned(),
            summary: format!(
                "installed {} '{}' to {}",
                self.rt.label(), meta.id, dest_file.display()
            ),
            dry_run: false,
        })
    }
}

impl Uninstaller for ThemeInstaller {
    fn resource_type(&self) -> ResourceType {
        self.rt
    }

    fn uninstall(&self, name: &str, paths: &InstallPaths, opts: &UninstallOptions) -> Result<(), FsError> {
        let dest_file = paths
            .config_file_for(self.rt, name)
            .ok_or_else(|| FsError::internal("ThemeInstaller: config_file_for returned None"))?;

        if opts.dry_run {
            println!("[dry-run] would remove {}", dest_file.display());
            return Ok(());
        }
        if !opts.keep_data {
            remove_path(&dest_file, opts.dry_run)?;
        }
        Ok(())
    }
}

// ── LanguageInstaller ─────────────────────────────────────────────────────────

/// Installs and removes Language pack resources (directory-based).
///
/// Language packs install into `{config_base}/i18n/{locale}/`.
pub struct LanguageInstaller;

impl Installer for LanguageInstaller {
    fn resource_type(&self) -> ResourceType {
        ResourceType::Language
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
            .dir_for(ResourceType::Language, &meta.id)
            .expect("LanguageInstaller: dir_for returned None");

        if dry_run {
            return Ok(InstallReport {
                install_path: dest.to_string_lossy().into_owned(),
                summary: format!("[dry-run] would install language '{}' to {}", meta.id, dest.display()),
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

        Ok(InstallReport {
            install_path: dest.to_string_lossy().into_owned(),
            summary:      format!("installed language '{}' to {}", meta.id, dest.display()),
            dry_run:      false,
        })
    }
}

impl Uninstaller for LanguageInstaller {
    fn resource_type(&self) -> ResourceType {
        ResourceType::Language
    }

    fn uninstall(&self, name: &str, paths: &InstallPaths, opts: &UninstallOptions) -> Result<(), FsError> {
        let dest = paths
            .dir_for(ResourceType::Language, name)
            .expect("LanguageInstaller: dir_for returned None");
        if opts.dry_run { println!("[dry-run] would remove {}", dest.display()); return Ok(()); }
        remove_path(&dest, opts.dry_run)
    }
}
