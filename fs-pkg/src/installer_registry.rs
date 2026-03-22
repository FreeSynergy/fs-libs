// installer_registry.rs — InstallerRegistry: maps ResourceType → Installer/Uninstaller.
//
// The registry instantiates the correct installer on demand based on ResourceType.
// All installers are stateless (except for InstallPaths), so we can create them
// on every call — no storage needed.
//
// Pattern: Registry (Factory variant) — single dispatch point for all resource types.

use std::path::Path;

use fs_error::FsError;
use fs_types::{ResourceMeta, ResourceType};

use crate::install_paths::InstallPaths;
use crate::installers::{
    app::AppInstaller,
    bot::BotInstaller,
    bridge::BridgeInstaller,
    bundle::BundleInstaller,
    container::ContainerInstaller,
    font::FontInstaller,
    icon::IconInstaller,
    theme::{LanguageInstaller, ThemeInstaller},
    widget::WidgetInstaller,
    InstallReport, Installer, UninstallOptions, Uninstaller,
};

// ── InstallerRegistry ─────────────────────────────────────────────────────────

/// Central registry for all installer/uninstaller implementations.
///
/// Instantiates the correct [`Installer`] or [`Uninstaller`] for a given
/// [`ResourceType`] on demand (stateless strategy dispatch).
///
/// # Example
///
/// ```rust,ignore
/// let paths    = InstallPaths::load();
/// let registry = InstallerRegistry::new();
///
/// // Check prerequisites before installing.
/// registry.check_prerequisites(ResourceType::App, &meta)?;
///
/// // Install.
/// let report = registry.install(&meta, source, &paths, false)?;
/// println!("{}", report.summary);
///
/// // Uninstall.
/// registry.uninstall(ResourceType::App, "kanidm", &paths, &UninstallOptions::default())?;
/// ```
pub struct InstallerRegistry;

impl InstallerRegistry {
    pub fn new() -> Self {
        Self
    }

    /// Check prerequisites for installing a resource of the given type.
    pub fn check_prerequisites(
        &self,
        rt: ResourceType,
        meta: &ResourceMeta,
    ) -> Result<(), FsError> {
        self.installer_for(rt).check_prerequisites(meta)
    }

    /// Install a resource.
    pub fn install(
        &self,
        meta: &ResourceMeta,
        source: Option<&Path>,
        paths: &InstallPaths,
        dry_run: bool,
    ) -> Result<InstallReport, FsError> {
        self.installer_for(meta.resource_type)
            .install(meta, source, paths, dry_run)
    }

    /// Uninstall a resource.
    pub fn uninstall(
        &self,
        rt: ResourceType,
        name: &str,
        paths: &InstallPaths,
        opts: &UninstallOptions,
    ) -> Result<(), FsError> {
        self.uninstaller_for(rt).uninstall(name, paths, opts)
    }

    // ── Private factories ──────────────────────────────────────────────────────

    fn installer_for(&self, rt: ResourceType) -> Box<dyn Installer> {
        match rt {
            ResourceType::App              => Box::new(AppInstaller),
            ResourceType::Bot              => Box::new(BotInstaller),
            ResourceType::MessengerAdapter => Box::new(BotInstaller), // same logic as Bot
            ResourceType::Widget           => Box::new(WidgetInstaller),
            ResourceType::Language         => Box::new(LanguageInstaller),
            ResourceType::FontSet          => Box::new(FontInstaller),
            ResourceType::IconSet          => Box::new(IconInstaller::for_icons()),
            ResourceType::CursorSet        => Box::new(IconInstaller::for_cursors()),
            ResourceType::Bridge           => Box::new(BridgeInstaller),
            ResourceType::Bundle           => Box::new(BundleInstaller),
            ResourceType::Container        => Box::new(ContainerInstaller),
            // All flat-file theme types share ThemeInstaller, parameterized by ResourceType.
            rt @ (ResourceType::Task
                | ResourceType::ColorScheme
                | ResourceType::Style
                | ResourceType::ButtonStyle
                | ResourceType::WindowChrome
                | ResourceType::AnimationSet) => Box::new(ThemeInstaller { rt }),
        }
    }

    fn uninstaller_for(&self, rt: ResourceType) -> Box<dyn Uninstaller> {
        match rt {
            ResourceType::App              => Box::new(AppInstaller),
            ResourceType::Bot              => Box::new(BotInstaller),
            ResourceType::MessengerAdapter => Box::new(BotInstaller),
            ResourceType::Widget           => Box::new(WidgetInstaller),
            ResourceType::Language         => Box::new(LanguageInstaller),
            ResourceType::FontSet          => Box::new(FontInstaller),
            ResourceType::IconSet          => Box::new(IconInstaller::for_icons()),
            ResourceType::CursorSet        => Box::new(IconInstaller::for_cursors()),
            ResourceType::Bridge           => Box::new(BridgeInstaller),
            ResourceType::Bundle           => Box::new(BundleInstaller),
            ResourceType::Container        => Box::new(ContainerInstaller),
            rt @ (ResourceType::Task
                | ResourceType::ColorScheme
                | ResourceType::Style
                | ResourceType::ButtonStyle
                | ResourceType::WindowChrome
                | ResourceType::AnimationSet) => Box::new(ThemeInstaller { rt }),
        }
    }
}

impl Default for InstallerRegistry {
    fn default() -> Self {
        Self::new()
    }
}
