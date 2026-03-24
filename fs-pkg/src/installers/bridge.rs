// installers/bridge.rs — Installer + Uninstaller for Bridge resources.
//
// Bridge = in-process Rust crate that maps a role API to a service API.
// No files are written — bridges are compiled into the binary.
// Install = inventory-only registration.

use std::path::Path;

use fs_error::FsError;
use fs_types::{ResourceMeta, ResourceType};

use crate::install_paths::InstallPaths;
use crate::installers::{InstallReport, Installer, UninstallOptions, Uninstaller};

pub struct BridgeInstaller;

impl Installer for BridgeInstaller {
    fn resource_type(&self) -> ResourceType {
        ResourceType::Bridge
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
                "{}registered bridge '{}' (in-process, no files written)",
                if dry_run {
                    "[dry-run] would register"
                } else {
                    ""
                },
                meta.id
            ),
            dry_run,
        })
    }
}

impl Uninstaller for BridgeInstaller {
    fn resource_type(&self) -> ResourceType {
        ResourceType::Bridge
    }

    fn uninstall(
        &self,
        name: &str,
        _paths: &InstallPaths,
        opts: &UninstallOptions,
    ) -> Result<(), FsError> {
        if opts.dry_run {
            println!("[dry-run] would unregister bridge '{name}' (in-process, no files to remove)");
        } else {
            println!("unregistered bridge '{name}' (in-process, no files to remove)");
        }
        Ok(())
    }
}
