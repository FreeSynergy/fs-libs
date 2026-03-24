// installers/widget.rs — Installer + Uninstaller for Widget resources.
//
// Widget = desktop widget with data-source requirements.
// Install path: {config_base}/widgets/{name}/
//
// Prerequisites: none.

use std::path::Path;

use fs_error::FsError;
use fs_types::{ResourceMeta, ResourceType};

use crate::install_paths::InstallPaths;
use crate::installers::{
    copy_source_to_dir, remove_path, InstallReport, Installer, UninstallOptions, Uninstaller,
};

pub struct WidgetInstaller;

impl Installer for WidgetInstaller {
    fn resource_type(&self) -> ResourceType {
        ResourceType::Widget
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
            .dir_for(ResourceType::Widget, &meta.id)
            .expect("WidgetInstaller: dir_for returned None");

        if dry_run {
            return Ok(InstallReport {
                install_path: dest.to_string_lossy().into_owned(),
                summary: format!(
                    "[dry-run] would install widget '{}' to {}",
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

        Ok(InstallReport {
            install_path: dest.to_string_lossy().into_owned(),
            summary: format!("installed widget '{}' to {}", meta.id, dest.display()),
            dry_run: false,
        })
    }
}

impl Uninstaller for WidgetInstaller {
    fn resource_type(&self) -> ResourceType {
        ResourceType::Widget
    }

    fn uninstall(
        &self,
        name: &str,
        paths: &InstallPaths,
        opts: &UninstallOptions,
    ) -> Result<(), FsError> {
        let dest = paths
            .dir_for(ResourceType::Widget, name)
            .expect("WidgetInstaller: dir_for returned None");
        if opts.dry_run {
            println!("[dry-run] would remove {}", dest.display());
            return Ok(());
        }
        remove_path(&dest, opts.dry_run)
    }
}
