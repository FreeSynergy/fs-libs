// install_paths.rs — Configurable installation paths for all ResourceTypes.
//
// InstallPaths stores base directories that can be reconfigured by the user.
// All resource-specific paths are derived from these bases.
// When the user changes a base, PathMigrator moves files with `rename`/copy-delete
// and the inventory records must be updated by the caller.
//
// Design:
//   InstallPaths  — holds configurable base dirs, computes per-resource paths
//   PathMigrator  — moves installed files when bases change
//
// Pattern: Value Object (InstallPaths is immutable config), Visitor (PathMigrator)

use std::path::{Path, PathBuf};

use fs_types::ResourceType;
use serde::{Deserialize, Serialize};

// ── InstallPaths ──────────────────────────────────────────────────────────────

/// Configurable base directories for all FreeSynergy resource installations.
///
/// All resource-specific paths are derived from these bases — changing a base
/// and calling [`PathMigrator::move_resource`] relocates all affected files.
///
/// # Defaults
///
/// | Field         | Default Path                         |
/// |---------------|--------------------------------------|
/// | `system_base` | `/opt/fsn`                           |
/// | `config_base` | `$HOME/.config/fsn`                  |
/// | `font_base`   | `$HOME/.local/share/fonts/fsn`       |
/// | `icon_base`   | `$HOME/.local/share/icons/fsn`       |
/// | `cursor_base` | `$HOME/.local/share/icons/fsn-cursors` |
///
/// # Persistence
///
/// Stored as TOML at `$HOME/.config/fsn/install-paths.toml`.
/// Load with [`InstallPaths::load`]; save with [`InstallPaths::save`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallPaths {
    /// Base directory for system-wide binaries: apps, bots, messenger adapters.
    pub system_base: PathBuf,
    /// Base directory for user configuration: widgets, tasks, themes, i18n, styles.
    pub config_base: PathBuf,
    /// Base directory for font files.
    pub font_base: PathBuf,
    /// Base directory for icon sets.
    pub icon_base: PathBuf,
    /// Base directory for cursor sets.
    pub cursor_base: PathBuf,
}

impl InstallPaths {
    /// Default paths for the current user (XDG-compliant).
    pub fn default_for_user() -> Self {
        let home = home_dir();
        Self {
            system_base: PathBuf::from("/opt/fsn"),
            config_base: home.join(".config/fsn"),
            font_base: home.join(".local/share/fonts/fsn"),
            icon_base: home.join(".local/share/icons/fsn"),
            cursor_base: home.join(".local/share/icons/fsn-cursors"),
        }
    }

    /// Load from the standard config file (`$HOME/.config/fsn/install-paths.toml`).
    /// Falls back to [`InstallPaths::default_for_user`] if the file does not exist.
    pub fn load() -> Self {
        let path = Self::config_file_path();
        match std::fs::read_to_string(&path) {
            Ok(content) => toml::from_str(&content).unwrap_or_else(|_| Self::default_for_user()),
            Err(_) => Self::default_for_user(),
        }
    }

    /// Save to the standard config file.
    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_file_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("cannot create config dir: {e}"))?;
        }
        let content =
            toml::to_string_pretty(self).map_err(|e| format!("cannot serialize paths: {e}"))?;
        std::fs::write(&path, content).map_err(|e| format!("cannot write {}: {e}", path.display()))
    }

    /// Path to the install-paths config file.
    pub fn config_file_path() -> PathBuf {
        home_dir().join(".config/fsn/install-paths.toml")
    }

    // ── Path derivation ───────────────────────────────────────────────────────

    /// Compute the installation **directory** for a resource.
    ///
    /// Returns `None` for resource types that are in-process, inventory-only,
    /// or managed externally (Bridge, Bundle, Container).
    ///
    /// For flat-file resources (Task, ColorScheme, …) use [`Self::config_file_for`].
    pub fn dir_for(&self, rt: ResourceType, name: &str) -> Option<PathBuf> {
        match rt {
            ResourceType::App => Some(self.system_base.join("apps").join(name)),
            ResourceType::Bot => Some(self.system_base.join("bots").join(name)),
            ResourceType::MessengerAdapter => Some(self.system_base.join("adapters").join(name)),
            ResourceType::Widget => Some(self.config_base.join("widgets").join(name)),
            ResourceType::Language => Some(self.config_base.join("i18n").join(name)),
            ResourceType::FontSet => Some(self.font_base.join(name)),
            ResourceType::IconSet => Some(self.icon_base.join(name)),
            ResourceType::CursorSet => Some(self.cursor_base.join(name)),
            // Flat-file resources: no directory
            ResourceType::Task
            | ResourceType::ColorScheme
            | ResourceType::Style
            | ResourceType::ButtonStyle
            | ResourceType::WindowChrome
            | ResourceType::AnimationSet => None,
            // In-process or externally managed — no local directory
            ResourceType::Bridge
            | ResourceType::Bundle
            | ResourceType::Container
            | ResourceType::Bootstrap
            | ResourceType::Repo
            | ResourceType::Theme => None,
        }
    }

    /// Compute the **config file path** for flat-file resources (TOML themes, tasks).
    ///
    /// Returns `None` for directory-based or in-process resource types.
    pub fn config_file_for(&self, rt: ResourceType, name: &str) -> Option<PathBuf> {
        let (subdir, ext) = match rt {
            ResourceType::Task => ("tasks", "toml"),
            ResourceType::ColorScheme => ("themes", "toml"),
            ResourceType::Style => ("styles", "toml"),
            ResourceType::ButtonStyle => ("button-styles", "toml"),
            ResourceType::WindowChrome => ("window-chrome", "toml"),
            ResourceType::AnimationSet => ("animations", "toml"),
            _ => return None,
        };
        Some(self.config_base.join(subdir).join(format!("{name}.{ext}")))
    }

    /// Compute the canonical install path string for an `InstalledResource` record.
    ///
    /// - Directory-based types: the directory path.
    /// - Flat-file types: the file path.
    /// - In-process / inventory-only types: empty string.
    pub fn install_path_for(&self, rt: ResourceType, name: &str) -> String {
        self.dir_for(rt, name)
            .or_else(|| self.config_file_for(rt, name))
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default()
    }

    // ── Grouping helpers ──────────────────────────────────────────────────────

    /// Resource types whose paths are rooted under [`Self::system_base`].
    pub fn system_types() -> &'static [ResourceType] {
        &[
            ResourceType::App,
            ResourceType::Bot,
            ResourceType::MessengerAdapter,
        ]
    }

    /// Resource types whose paths are rooted under [`Self::config_base`].
    pub fn config_types() -> &'static [ResourceType] {
        &[
            ResourceType::Widget,
            ResourceType::Language,
            ResourceType::Task,
            ResourceType::ColorScheme,
            ResourceType::Style,
            ResourceType::ButtonStyle,
            ResourceType::WindowChrome,
            ResourceType::AnimationSet,
        ]
    }

    /// Resource types whose paths are rooted under [`Self::font_base`].
    pub fn font_types() -> &'static [ResourceType] {
        &[ResourceType::FontSet]
    }

    /// Resource types whose paths are rooted under [`Self::icon_base`] or [`Self::cursor_base`].
    pub fn icon_types() -> &'static [ResourceType] {
        &[ResourceType::IconSet, ResourceType::CursorSet]
    }
}

impl Default for InstallPaths {
    fn default() -> Self {
        Self::default_for_user()
    }
}

// ── MoveOutcome ───────────────────────────────────────────────────────────────

/// Result of moving a single resource to a new path.
#[derive(Debug)]
pub struct MoveOutcome {
    /// Absolute path where the resource now lives (empty for in-process types).
    pub new_path: String,
}

// ── PathMigrator ──────────────────────────────────────────────────────────────

/// Moves installed resource files from old base paths to new ones.
///
/// Call this after the user changes a base directory (e.g. via `fsn config install-root`).
/// Uses `std::fs::rename` (equivalent to `mv`) — falls back to recursive copy+delete
/// when source and destination are on different filesystems.
///
/// # Responsibility
///
/// `PathMigrator` only moves files on disk. The caller must update
/// `config_path`/`data_path` in the inventory records using [`MoveOutcome::new_path`].
///
/// # Example
///
/// ```rust,ignore
/// let migrator = PathMigrator { old: &old_paths, new: &new_paths };
/// let outcome  = migrator.move_resource(ResourceType::App, "kanidm")?;
/// // update inventory record: record.config_path = outcome.new_path;
/// ```
pub struct PathMigrator<'a> {
    /// Paths before the change.
    pub old: &'a InstallPaths,
    /// Paths after the change.
    pub new: &'a InstallPaths,
}

impl<'a> PathMigrator<'a> {
    /// Move a single resource's files from the old path to the new path.
    ///
    /// Returns `Ok(MoveOutcome)` with the new path on success.
    /// Returns `Err(message)` with a human-readable error on failure.
    /// Does nothing (returns new path) when:
    ///   - The resource type has no path (Bridge, Bundle, Container).
    ///   - The old and new paths are identical.
    ///   - The old path does not exist on disk.
    pub fn move_resource(&self, rt: ResourceType, name: &str) -> Result<MoveOutcome, String> {
        let old_path = self.old.install_path_for(rt, name);
        let new_path = self.new.install_path_for(rt, name);

        if old_path.is_empty() {
            return Ok(MoveOutcome { new_path });
        }

        if old_path == new_path {
            return Ok(MoveOutcome { new_path });
        }

        let old_p = Path::new(&old_path);
        if !old_p.exists() {
            // Nothing on disk yet — just return the new path for future installs.
            return Ok(MoveOutcome { new_path });
        }

        let new_p = Path::new(&new_path);
        if let Some(parent) = new_p.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("cannot create {}: {e}", parent.display()))?;
        }

        // Prefer atomic rename (same filesystem = instant mv).
        if std::fs::rename(old_p, new_p).is_err() {
            // Cross-filesystem fallback: recursive copy then remove source.
            if old_p.is_dir() {
                copy_dir_all(old_p, new_p).map_err(|e| format!("copy {}: {e}", old_p.display()))?;
                std::fs::remove_dir_all(old_p)
                    .map_err(|e| format!("cleanup {}: {e}", old_p.display()))?;
            } else {
                std::fs::copy(old_p, new_p)
                    .map_err(|e| format!("copy {}: {e}", old_p.display()))?;
                std::fs::remove_file(old_p)
                    .map_err(|e| format!("cleanup {}: {e}", old_p.display()))?;
            }
        }

        Ok(MoveOutcome { new_path })
    }
}

// ── Private helpers ───────────────────────────────────────────────────────────

/// Recursively copy a directory tree from `src` to `dst`.
fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let dest_path = dst.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_all(&entry.path(), &dest_path)?;
        } else {
            std::fs::copy(entry.path(), &dest_path)?;
        }
    }
    Ok(())
}

/// Resolve `$HOME`, falling back to `/root`.
fn home_dir() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/root"))
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn fixed_paths() -> InstallPaths {
        InstallPaths {
            system_base: PathBuf::from("/opt/fsn"),
            config_base: PathBuf::from("/home/user/.config/fsn"),
            font_base: PathBuf::from("/home/user/.local/share/fonts/fsn"),
            icon_base: PathBuf::from("/home/user/.local/share/icons/fsn"),
            cursor_base: PathBuf::from("/home/user/.local/share/icons/fsn-cursors"),
        }
    }

    #[test]
    fn app_goes_to_system_base() {
        let p = fixed_paths();
        assert_eq!(
            p.dir_for(ResourceType::App, "kanidm"),
            Some(PathBuf::from("/opt/fsn/apps/kanidm"))
        );
    }

    #[test]
    fn bot_goes_to_system_base() {
        let p = fixed_paths();
        assert_eq!(
            p.dir_for(ResourceType::Bot, "mybot"),
            Some(PathBuf::from("/opt/fsn/bots/mybot"))
        );
    }

    #[test]
    fn messenger_adapter_goes_to_system_base() {
        let p = fixed_paths();
        assert_eq!(
            p.dir_for(ResourceType::MessengerAdapter, "telegram"),
            Some(PathBuf::from("/opt/fsn/adapters/telegram"))
        );
    }

    #[test]
    fn widget_goes_to_config_base() {
        let p = fixed_paths();
        assert_eq!(
            p.dir_for(ResourceType::Widget, "clock"),
            Some(PathBuf::from("/home/user/.config/fsn/widgets/clock"))
        );
    }

    #[test]
    fn language_goes_to_config_base() {
        let p = fixed_paths();
        assert_eq!(
            p.dir_for(ResourceType::Language, "de"),
            Some(PathBuf::from("/home/user/.config/fsn/i18n/de"))
        );
    }

    #[test]
    fn font_set_goes_to_font_base() {
        let p = fixed_paths();
        assert_eq!(
            p.dir_for(ResourceType::FontSet, "inter"),
            Some(PathBuf::from("/home/user/.local/share/fonts/fsn/inter"))
        );
    }

    #[test]
    fn icon_set_goes_to_icon_base() {
        let p = fixed_paths();
        assert_eq!(
            p.dir_for(ResourceType::IconSet, "papirus"),
            Some(PathBuf::from("/home/user/.local/share/icons/fsn/papirus"))
        );
    }

    #[test]
    fn cursor_set_goes_to_cursor_base() {
        let p = fixed_paths();
        assert_eq!(
            p.dir_for(ResourceType::CursorSet, "breeze"),
            Some(PathBuf::from(
                "/home/user/.local/share/icons/fsn-cursors/breeze"
            ))
        );
    }

    #[test]
    fn task_is_flat_file() {
        let p = fixed_paths();
        assert_eq!(p.dir_for(ResourceType::Task, "backup"), None);
        assert_eq!(
            p.config_file_for(ResourceType::Task, "backup"),
            Some(PathBuf::from("/home/user/.config/fsn/tasks/backup.toml"))
        );
    }

    #[test]
    fn color_scheme_is_flat_file() {
        let p = fixed_paths();
        assert_eq!(p.dir_for(ResourceType::ColorScheme, "dracula"), None);
        assert_eq!(
            p.config_file_for(ResourceType::ColorScheme, "dracula"),
            Some(PathBuf::from("/home/user/.config/fsn/themes/dracula.toml"))
        );
    }

    #[test]
    fn style_is_flat_file() {
        let p = fixed_paths();
        assert_eq!(
            p.config_file_for(ResourceType::Style, "compact"),
            Some(PathBuf::from("/home/user/.config/fsn/styles/compact.toml"))
        );
    }

    #[test]
    fn button_style_is_flat_file() {
        let p = fixed_paths();
        assert_eq!(
            p.config_file_for(ResourceType::ButtonStyle, "rounded"),
            Some(PathBuf::from(
                "/home/user/.config/fsn/button-styles/rounded.toml"
            ))
        );
    }

    #[test]
    fn window_chrome_is_flat_file() {
        let p = fixed_paths();
        assert_eq!(
            p.config_file_for(ResourceType::WindowChrome, "minimal"),
            Some(PathBuf::from(
                "/home/user/.config/fsn/window-chrome/minimal.toml"
            ))
        );
    }

    #[test]
    fn animation_set_is_flat_file() {
        let p = fixed_paths();
        assert_eq!(
            p.config_file_for(ResourceType::AnimationSet, "fast"),
            Some(PathBuf::from("/home/user/.config/fsn/animations/fast.toml"))
        );
    }

    #[test]
    fn bridge_has_no_path() {
        let p = fixed_paths();
        assert_eq!(p.dir_for(ResourceType::Bridge, "kanidm-iam"), None);
        assert_eq!(p.config_file_for(ResourceType::Bridge, "kanidm-iam"), None);
        assert!(p
            .install_path_for(ResourceType::Bridge, "kanidm-iam")
            .is_empty());
    }

    #[test]
    fn bundle_has_no_path() {
        let p = fixed_paths();
        assert_eq!(p.dir_for(ResourceType::Bundle, "starter"), None);
        assert!(p
            .install_path_for(ResourceType::Bundle, "starter")
            .is_empty());
    }

    #[test]
    fn container_has_no_path() {
        let p = fixed_paths();
        assert_eq!(p.dir_for(ResourceType::Container, "forgejo"), None);
        assert!(p
            .install_path_for(ResourceType::Container, "forgejo")
            .is_empty());
    }

    #[test]
    fn install_path_for_combines_dir_and_file_types() {
        let p = fixed_paths();
        assert_eq!(
            p.install_path_for(ResourceType::App, "forgejo"),
            "/opt/fsn/apps/forgejo"
        );
        assert_eq!(
            p.install_path_for(ResourceType::Task, "backup"),
            "/home/user/.config/fsn/tasks/backup.toml"
        );
    }

    #[test]
    fn migrator_noop_when_paths_identical() {
        let paths = fixed_paths();
        let migrator = PathMigrator {
            old: &paths,
            new: &paths,
        };
        let outcome = migrator.move_resource(ResourceType::App, "kanidm").unwrap();
        assert_eq!(outcome.new_path, "/opt/fsn/apps/kanidm");
    }

    #[test]
    fn migrator_noop_for_in_process_types() {
        let paths = fixed_paths();
        let migrator = PathMigrator {
            old: &paths,
            new: &paths,
        };
        let outcome = migrator.move_resource(ResourceType::Bridge, "any").unwrap();
        assert!(outcome.new_path.is_empty());
    }

    #[test]
    fn migrator_moves_files_across_bases() {
        use tempfile::TempDir;

        let src_tmp = TempDir::new().unwrap();
        let dst_tmp = TempDir::new().unwrap();

        let old_paths = InstallPaths {
            system_base: src_tmp.path().join("opt/fsn"),
            config_base: src_tmp.path().join("config/fsn"),
            font_base: src_tmp.path().join("fonts/fsn"),
            icon_base: src_tmp.path().join("icons/fsn"),
            cursor_base: src_tmp.path().join("cursors/fsn"),
        };
        let new_paths = InstallPaths {
            system_base: dst_tmp.path().join("opt/fsn"),
            config_base: dst_tmp.path().join("config/fsn"),
            font_base: dst_tmp.path().join("fonts/fsn"),
            icon_base: dst_tmp.path().join("icons/fsn"),
            cursor_base: dst_tmp.path().join("cursors/fsn"),
        };

        // Create a fake app directory at the old location.
        let app_src = old_paths.dir_for(ResourceType::App, "testapp").unwrap();
        std::fs::create_dir_all(&app_src).unwrap();
        std::fs::write(app_src.join("testapp"), b"binary").unwrap();

        let migrator = PathMigrator {
            old: &old_paths,
            new: &new_paths,
        };
        let outcome = migrator
            .move_resource(ResourceType::App, "testapp")
            .unwrap();

        let expected = new_paths.dir_for(ResourceType::App, "testapp").unwrap();
        assert_eq!(outcome.new_path, expected.to_string_lossy());
        assert!(
            expected.join("testapp").exists(),
            "file must exist at new location"
        );
        assert!(!app_src.exists(), "old directory must be gone");
    }

    #[test]
    fn save_and_load_roundtrip() {
        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();
        // Override HOME for test isolation.
        std::env::set_var("HOME", tmp.path());

        let paths = InstallPaths {
            system_base: PathBuf::from("/custom/fsn"),
            config_base: PathBuf::from("/custom/config"),
            font_base: PathBuf::from("/custom/fonts"),
            icon_base: PathBuf::from("/custom/icons"),
            cursor_base: PathBuf::from("/custom/cursors"),
        };
        paths.save().unwrap();

        let loaded = InstallPaths::load();
        assert_eq!(loaded.system_base, paths.system_base);
        assert_eq!(loaded.config_base, paths.config_base);
    }
}
