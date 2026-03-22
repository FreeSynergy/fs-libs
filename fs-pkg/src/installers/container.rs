// installers/container.rs — Installer + Uninstaller for Container resources.
//
// Container = containerized application deployed via Podman Compose.
// The actual deployment is handled by fs-deploy (Quadlet generation).
// This installer only handles inventory registration.
//
// Prerequisites: Podman must be installed.

use std::path::Path;

use fs_error::FsError;
use fs_types::{ResourceMeta, ResourceType};

use crate::install_paths::InstallPaths;
use crate::installers::{require_tool, InstallReport, Installer, UninstallOptions, Uninstaller};

pub struct ContainerInstaller;

impl Installer for ContainerInstaller {
    fn resource_type(&self) -> ResourceType {
        ResourceType::Container
    }

    fn check_prerequisites(&self, _meta: &ResourceMeta) -> Result<(), FsError> {
        require_tool(
            "podman",
            "install with: dnf install podman  (or: apt install podman)",
        )
    }

    fn install(
        &self,
        meta: &ResourceMeta,
        _source: Option<&Path>,
        _paths: &InstallPaths,
        dry_run: bool,
    ) -> Result<InstallReport, FsError> {
        // Container lifecycle (pull image, create Quadlet files, start) is handled
        // by `fsn deploy` via fs-deploy. This installer only records the intent.
        Ok(InstallReport {
            install_path: String::new(),
            summary: format!(
                "{}registered container '{}' — run `fsn deploy` to start",
                if dry_run { "[dry-run] would register" } else { "" },
                meta.id
            ),
            dry_run,
        })
    }
}

impl Uninstaller for ContainerInstaller {
    fn resource_type(&self) -> ResourceType {
        ResourceType::Container
    }

    fn uninstall(&self, name: &str, _paths: &InstallPaths, opts: &UninstallOptions) -> Result<(), FsError> {
        // Actual container removal is via `fsn undeploy` / `fsn remove --service`.
        if opts.dry_run {
            println!("[dry-run] would unregister container '{name}' — run `fsn undeploy` to stop first");
        } else {
            println!("unregistered container '{name}' — run `fsn undeploy --service {name}` to stop");
        }
        Ok(())
    }
}
