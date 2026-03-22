// installers/bundle.rs — Installer + Uninstaller for Bundle resources.
//
// Bundle = meta-package referencing other resources.
// No own files — only an inventory entry.
// The CLI resolves and installs all bundled packages individually.

use std::path::Path;

use fs_error::FsError;
use fs_types::{ResourceMeta, ResourceType};

use crate::install_paths::InstallPaths;
use crate::installers::{InstallReport, Installer, UninstallOptions, Uninstaller};

pub struct BundleInstaller;

impl Installer for BundleInstaller {
    fn resource_type(&self) -> ResourceType {
        ResourceType::Bundle
    }

    fn check_prerequisites(&self, _meta: &ResourceMeta) -> Result<(), FsError> {
        Ok(())
    }

    fn install(
        &self,
        meta: &ResourceMeta,
        _source: Option<&Path>,
        _paths: &InstallPaths,
        dry_run: bool,
    ) -> Result<InstallReport, FsError> {
        Ok(InstallReport {
            install_path: String::new(),
            summary: format!(
                "{}registered bundle '{}' ({} dependencies — install individually)",
                if dry_run { "[dry-run] would register" } else { "" },
                meta.id,
                meta.dependencies.len()
            ),
            dry_run,
        })
    }
}

impl Uninstaller for BundleInstaller {
    fn resource_type(&self) -> ResourceType {
        ResourceType::Bundle
    }

    fn uninstall(&self, name: &str, _paths: &InstallPaths, opts: &UninstallOptions) -> Result<(), FsError> {
        if opts.dry_run {
            println!("[dry-run] would unregister bundle '{name}' (no files to remove)");
        } else {
            println!("unregistered bundle '{name}' (constituent packages not removed automatically)");
        }
        Ok(())
    }
}
