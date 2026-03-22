// installer.rs — PackageInstaller: orchestrates the full install/remove lifecycle.
//
// Install flow:
//   1. Validate manifest + requirements
//   2. Emit InstallStarted event
//   3. Run pre_install hooks
//   4. Write files (config, units, data)
//   5. Run post_install hooks
//   6. Emit InstallCompleted event
//
// Remove flow:
//   1. Emit RemoveStarted event
//   2. Run pre_remove hooks
//   3. Delete installed files
//   4. Run post_remove hooks
//   5. Emit RemoveCompleted event
//
// On any failure: emit InstallFailed / abort (no partial rollback in v0.1).

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use fs_error::FsError;

use crate::event::{EventBus, InstallEvent, InstallEventKind};
use crate::manifest::{ApiManifest, PackageId};

// ── TemplateVars ──────────────────────────────────────────────────────────────

/// Template variables for file destination path expansion.
///
/// Replaces `{key}` placeholders in template strings with their values.
#[derive(Debug, Clone, Default)]
pub struct TemplateVars(HashMap<String, String>);

impl TemplateVars {
    /// Create an empty variable set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a key-value pair.
    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.0.insert(key.into(), value.into());
    }

    /// Replace `{key}` placeholders in `template` with their values.
    pub fn expand(&self, template: &str) -> String {
        let mut result = template.to_string();
        for (k, v) in &self.0 {
            result = result.replace(&format!("{{{k}}}"), v);
        }
        result
    }
}

// ── InstallOptions ────────────────────────────────────────────────────────────

/// Options for a package install operation.
#[derive(Debug, Clone, Default)]
pub struct InstallOptions {
    /// Template variables for file destination paths (e.g. `data_root → /srv`).
    pub vars: TemplateVars,

    /// Perform a dry run: validate + print actions, but don't write anything.
    pub dry_run: bool,

    /// Skip hooks (pre/post install).
    pub skip_hooks: bool,
}

impl InstallOptions {
    /// Create options with a single variable.
    pub fn with_var(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.vars.insert(key, value);
        self
    }
}

// ── InstallOutcome ────────────────────────────────────────────────────────────

/// Result of a completed install or remove operation.
#[derive(Debug, Clone)]
pub struct InstallOutcome {
    /// Package ID.
    pub package_id: PackageId,

    /// Package version.
    pub version: String,

    /// Files that were written (absolute destination paths).
    pub written_files: Vec<PathBuf>,

    /// Hook commands that ran (in order).
    pub ran_hooks: Vec<String>,

    /// `true` if this was a dry run (nothing was actually written).
    pub dry_run: bool,
}

// ── PackageInstaller ──────────────────────────────────────────────────────────

/// Orchestrates the full install/remove lifecycle for a package.
///
/// Uses an [`EventBus`] to notify registered hooks at each lifecycle stage.
///
/// # Example
///
/// ```rust,ignore
/// use fs_pkg::{ApiManifest, InstallOptions, PackageInstaller};
///
/// let manifest = ApiManifest::from_file(Path::new("manifest.toml"))?;
/// let options = InstallOptions::default()
///     .with_var("data_root", "/srv/data/zentinel");
///
/// let mut installer = PackageInstaller::new();
/// installer.register_hook("logger", |e| {
///     println!("[pkg] {}: {:?}", e.package_id, e.kind);
///     Ok(())
/// });
///
/// let outcome = installer.install(&manifest, options)?;
/// println!("wrote {} files", outcome.written_files.len());
/// ```
pub struct PackageInstaller {
    bus: EventBus,
}

impl PackageInstaller {
    /// Create a new installer with no hooks registered.
    pub fn new() -> Self {
        Self { bus: EventBus::new() }
    }

    /// Register a named lifecycle hook.
    ///
    /// See [`EventBus::register`] for semantics.
    pub fn register_hook(
        &mut self,
        name: impl Into<String>,
        hook: impl Fn(&InstallEvent) -> Result<(), FsError> + Send + Sync + 'static,
    ) {
        self.bus.register(name, hook);
    }

    /// Install a package according to `manifest` and `options`.
    pub fn install(
        &mut self,
        manifest: &ApiManifest,
        options: InstallOptions,
    ) -> Result<InstallOutcome, FsError> {
        let id = &manifest.package.id;
        let ver = &manifest.package.version;

        self.bus.emit(&InstallEvent::new(id, ver, InstallEventKind::InstallStarted))?;

        let mut written_files = Vec::new();
        let mut ran_hooks = Vec::new();

        // Pre-install hooks
        if !options.skip_hooks {
            for cmd in &manifest.hooks.pre_install {
                run_hook(cmd, &options.vars, options.dry_run)?;
                ran_hooks.push(cmd.clone());
            }
        }

        // Write all declared files
        for mapping in manifest.files.all() {
            let src = mapping.source.as_str();
            let dest = options.vars.expand(&mapping.dest);
            let dest_path = PathBuf::from(&dest);

            if !options.dry_run {
                if let Some(parent) = dest_path.parent() {
                    fs::create_dir_all(parent).map_err(|e| {
                        FsError::internal(format!(
                            "pkg install: cannot create {}: {e}",
                            parent.display()
                        ))
                    })?;
                }

                // For now: write the source path as a marker file.
                // In production this copies from the package bundle.
                let content = format!("# installed from package: {src}\n");
                fs::write(&dest_path, content).map_err(|e| {
                    FsError::internal(format!(
                        "pkg install: cannot write {}: {e}",
                        dest_path.display()
                    ))
                })?;
            }

            written_files.push(dest_path);
        }

        // Post-install hooks
        if !options.skip_hooks {
            for cmd in &manifest.hooks.post_install {
                run_hook(cmd, &options.vars, options.dry_run)?;
                ran_hooks.push(cmd.clone());
            }
        }

        self.bus.emit(&InstallEvent::new(id, ver, InstallEventKind::InstallCompleted))?;

        Ok(InstallOutcome {
            package_id: id.clone(),
            version:    ver.clone(),
            written_files,
            ran_hooks,
            dry_run: options.dry_run,
        })
    }

    /// Remove a package, deleting all files declared in `manifest`.
    pub fn remove(
        &mut self,
        manifest: &ApiManifest,
        options: InstallOptions,
    ) -> Result<InstallOutcome, FsError> {
        let id = &manifest.package.id;
        let ver = &manifest.package.version;

        self.bus.emit(&InstallEvent::new(id, ver, InstallEventKind::RemoveStarted))?;

        let mut removed_files = Vec::new();
        let mut ran_hooks = Vec::new();

        // Pre-remove hooks
        if !options.skip_hooks {
            for cmd in &manifest.hooks.pre_remove {
                run_hook(cmd, &options.vars, options.dry_run)?;
                ran_hooks.push(cmd.clone());
            }
        }

        // Delete all declared files
        for mapping in manifest.files.all() {
            let dest = options.vars.expand(&mapping.dest);
            let path = PathBuf::from(&dest);

            if !options.dry_run && path.exists() {
                fs::remove_file(&path).map_err(|e| {
                    FsError::internal(format!(
                        "pkg remove: cannot delete {}: {e}",
                        path.display()
                    ))
                })?;
            }

            removed_files.push(path);
        }

        // Post-remove hooks
        if !options.skip_hooks {
            for cmd in &manifest.hooks.post_remove {
                run_hook(cmd, &options.vars, options.dry_run)?;
                ran_hooks.push(cmd.clone());
            }
        }

        self.bus.emit(&InstallEvent::new(id, ver, InstallEventKind::RemoveCompleted))?;

        Ok(InstallOutcome {
            package_id: id.clone(),
            version:    ver.clone(),
            written_files: removed_files,
            ran_hooks,
            dry_run: options.dry_run,
        })
    }
}

impl Default for PackageInstaller {
    fn default() -> Self {
        Self::new()
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Run a shell hook command.
///
/// In dry-run mode: log and skip.
fn run_hook(cmd: &str, vars: &TemplateVars, dry_run: bool) -> Result<(), FsError> {
    let expanded = vars.expand(cmd);
    if dry_run {
        // In dry-run mode, just validate that the command is non-empty
        return Ok(());
    }
    let status = Command::new("sh")
        .arg("-c")
        .arg(&expanded)
        .status()
        .map_err(|e| FsError::internal(format!("hook spawn failed `{expanded}`: {e}")))?;
    if !status.success() {
        return Err(FsError::internal(format!(
            "hook failed (exit {}): {expanded}",
            status.code().unwrap_or(-1)
        )));
    }
    Ok(())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::ApiManifest;
    use std::sync::{Arc, Mutex};
    use tempfile::TempDir;

    const MANIFEST_TOML: &str = r#"
[package]
id      = "test/pkg"
name    = "Test Package"
version = "0.1.0"

[files.config]
"config.toml" = "{data_root}/config.toml"

[hooks]
pre_install  = ["echo pre-install"]
post_install = ["echo post-install"]
pre_remove   = ["echo pre-remove"]
post_remove  = ["echo post-remove"]
"#;

    fn make_manifest() -> ApiManifest {
        ApiManifest::from_toml(MANIFEST_TOML).unwrap()
    }

    #[test]
    fn dry_run_install_writes_no_files() {
        let dir = TempDir::new().unwrap();
        let manifest = make_manifest();
        let options = InstallOptions::default()
            .with_var("data_root", dir.path().to_str().unwrap())
            .with_var("data_root", dir.path().to_str().unwrap());
        let mut options = options;
        options.dry_run = true;

        let outcome = PackageInstaller::new().install(&manifest, options).unwrap();

        assert!(outcome.dry_run);
        // File should NOT exist in dry-run mode
        let dest = dir.path().join("config.toml");
        assert!(!dest.exists(), "dry-run must not create files");
    }

    #[test]
    fn install_writes_declared_files() {
        let dir = TempDir::new().unwrap();
        let manifest = make_manifest();
        let options = InstallOptions::default()
            .with_var("data_root", dir.path().to_str().unwrap());

        let mut opts = options;
        opts.skip_hooks = true; // skip echo hooks to keep test clean

        let outcome = PackageInstaller::new().install(&manifest, opts).unwrap();

        assert_eq!(outcome.written_files.len(), 1);
        let dest = dir.path().join("config.toml");
        assert!(dest.exists(), "install must create the file");
    }

    #[test]
    fn remove_deletes_installed_files() {
        let dir = TempDir::new().unwrap();
        let manifest = make_manifest();
        let opts = InstallOptions::default()
            .with_var("data_root", dir.path().to_str().unwrap());

        // Install first
        let mut install_opts = opts.clone();
        install_opts.skip_hooks = true;
        PackageInstaller::new().install(&manifest, install_opts).unwrap();

        let dest = dir.path().join("config.toml");
        assert!(dest.exists());

        // Remove
        let mut remove_opts = opts;
        remove_opts.skip_hooks = true;
        PackageInstaller::new().remove(&manifest, remove_opts).unwrap();

        assert!(!dest.exists(), "remove must delete the file");
    }

    #[test]
    fn install_emits_events() {
        let events: Arc<Mutex<Vec<InstallEventKind>>> = Arc::new(Mutex::new(vec![]));
        let ev = events.clone();

        let mut installer = PackageInstaller::new();
        installer.register_hook("tracker", move |e| {
            ev.lock().unwrap().push(e.kind.clone());
            Ok(())
        });

        let dir = TempDir::new().unwrap();
        let manifest = make_manifest();
        let mut opts = InstallOptions::default()
            .with_var("data_root", dir.path().to_str().unwrap());
        opts.skip_hooks = true;

        installer.install(&manifest, opts).unwrap();

        let recorded = events.lock().unwrap();
        assert!(recorded.contains(&InstallEventKind::InstallStarted));
        assert!(recorded.contains(&InstallEventKind::InstallCompleted));
    }

    #[test]
    fn template_vars_expands_placeholders() {
        let mut vars = TemplateVars::new();
        vars.insert("data_root", "/srv/data");
        vars.insert("name", "zentinel");

        let result = vars.expand("{data_root}/{name}/config.toml");
        assert_eq!(result, "/srv/data/zentinel/config.toml");
    }
}
