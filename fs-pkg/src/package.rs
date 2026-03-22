// package.rs — Package: the unified domain object for one FreeSynergy package.
//
// A Package knows everything about itself:
//   - What it is   (manifest: name, version, type, author, ...)
//   - Whether it is installed   (checked against the InstalledRecord)
//   - Whether it is running     (status field)
//   - Whether it is healthy     (check_health())
//   - What it needs configured  (config_fields() via Manageable)
//   - What its sub-instances are (instances() via Manageable)
//
// No external component may ask a registry "is X installed?"
// The Package object itself carries the answer.
//
// Design:
//   Package         — the domain object
//   InstalledRecord — snapshot of the install state loaded from the inventory
//   PackageBuilder  — fluent constructor for tests / store registration

use serde::{Deserialize, Serialize};

use fs_error::FsError;

use crate::manageable::{
    ConfigField, ConfigFieldKind, ConfigValue, HealthCheck, Manageable, PackageHealth, RunStatus,
};
use crate::manifest::{ApiManifest, ManifestFieldType, PackageMeta, PackageType};

// ── InstalledRecord ───────────────────────────────────────────────────────────

/// Snapshot of the install state loaded from the inventory.
///
/// Stored inside [`Package`] so the object carries its own truth about whether
/// it is installed and in what state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InstalledRecord {
    /// Installed version string (e.g. `"1.5.0"`).
    pub version: String,
    /// ISO-8601 installation timestamp.
    pub installed_at: String,
    /// Path to the package's config file.
    pub config_path: String,
    /// Path to the package's data directory.
    pub data_path: String,
    /// Current runtime status.
    pub status: RunStatus,
}

impl InstalledRecord {
    pub fn new(version: impl Into<String>, installed_at: impl Into<String>) -> Self {
        Self {
            version:      version.into(),
            installed_at: installed_at.into(),
            config_path:  String::new(),
            data_path:    String::new(),
            status:       RunStatus::Stopped,
        }
    }

    pub fn with_paths(
        mut self,
        config_path: impl Into<String>,
        data_path:   impl Into<String>,
    ) -> Self {
        self.config_path = config_path.into();
        self.data_path   = data_path.into();
        self
    }

    pub fn with_status(mut self, status: RunStatus) -> Self {
        self.status = status;
        self
    }
}

// ── RunStatus serialisation ───────────────────────────────────────────────────

impl Serialize for RunStatus {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(self.label())
    }
}

impl<'de> Deserialize<'de> for RunStatus {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let label = String::deserialize(d)?;
        Ok(match label.as_str() {
            "Running"       => Self::Running,
            "Starting"      => Self::Starting,
            "Stopping"      => Self::Stopping,
            "Not installed" => Self::NotInstalled,
            s if s.starts_with("Error") => Self::Error(s.to_string()),
            _               => Self::Stopped,
        })
    }
}

// ── Package ───────────────────────────────────────────────────────────────────

/// The unified domain object for one FreeSynergy package.
///
/// A `Package` is the single source of truth for everything about one
/// installable unit — it combines the static manifest with the runtime state
/// loaded from the inventory.
///
/// # Self-responsibility
///
/// - `pkg.is_installed()` — no external registry needed
/// - `pkg.run_status()`   — package carries its own status
/// - `pkg.check_health()` — package runs its own checks
/// - `pkg.config_fields()` — package knows what to configure
///
/// The Manager receives a `&mut dyn Manageable` and renders the UI around it.
/// It never reaches inside the package — it always asks via the trait.
#[derive(Debug, Clone)]
pub struct Package {
    /// Static manifest (identity, files, hooks, requirements).
    pub manifest: ApiManifest,

    /// Install state — `None` if the package is not installed.
    installed: Option<InstalledRecord>,

    /// Mutable config values that have been edited (key → value).
    config_overrides: std::collections::HashMap<String, ConfigValue>,
}

impl Package {
    // ── Constructors ──────────────────────────────────────────────────────────

    /// Create a package from a manifest without an install record (not installed).
    pub fn from_manifest(manifest: ApiManifest) -> Self {
        Self {
            manifest,
            installed:        None,
            config_overrides: std::collections::HashMap::new(),
        }
    }

    /// Create a package from a manifest + install record.
    pub fn from_installed(manifest: ApiManifest, record: InstalledRecord) -> Self {
        Self {
            manifest,
            installed:        Some(record),
            config_overrides: std::collections::HashMap::new(),
        }
    }

    // ── Accessors ─────────────────────────────────────────────────────────────

    /// The install record, if present.
    pub fn install_record(&self) -> Option<&InstalledRecord> {
        self.installed.as_ref()
    }

    /// Update the install record (called after install / upgrade / uninstall).
    pub fn set_installed(&mut self, record: InstalledRecord) {
        self.installed = Some(record);
    }

    /// Remove the install record (called after uninstall).
    pub fn set_uninstalled(&mut self) {
        self.installed = None;
    }

    /// Directly update the runtime status in the install record.
    pub fn set_status(&mut self, status: RunStatus) {
        if let Some(rec) = &mut self.installed {
            rec.status = status;
        }
    }

    /// Returns the config path from the install record.
    pub fn config_path(&self) -> Option<&str> {
        self.installed.as_ref().map(|r| r.config_path.as_str())
    }

    /// Returns the data path from the install record.
    pub fn data_path(&self) -> Option<&str> {
        self.installed.as_ref().map(|r| r.data_path.as_str())
    }
}

// ── Manageable impl ───────────────────────────────────────────────────────────

/// Default `Manageable` implementation for `Package`.
///
/// Packages that need richer behavior (e.g. a Container package that can list
/// its running instances from Podman) should use a newtype wrapper or a
/// concrete struct that holds a `Package` internally and overrides the
/// relevant methods.
impl Manageable for Package {
    fn meta(&self) -> &PackageMeta {
        &self.manifest.package
    }

    fn package_type(&self) -> PackageType {
        self.manifest.package.package_type
    }

    fn is_installed(&self) -> bool {
        self.installed.is_some()
    }

    fn run_status(&self) -> RunStatus {
        match &self.installed {
            Some(rec) => rec.status.clone(),
            None      => RunStatus::NotInstalled,
        }
    }

    fn config_fields(&self) -> Vec<ConfigField> {
        // Build ConfigFields from the manifest's [[variables]] entries.
        // Each ManifestVariable maps to one ConfigField rendered in the Config tab.
        self.manifest.variables.iter().map(|v| {
            let kind = match v.field_type {
                ManifestFieldType::Bool     => ConfigFieldKind::Bool,
                ManifestFieldType::Port     => ConfigFieldKind::Port,
                ManifestFieldType::Password |
                ManifestFieldType::Secret   => ConfigFieldKind::Password,
                ManifestFieldType::Path     => ConfigFieldKind::Path,
                ManifestFieldType::Textarea => ConfigFieldKind::Textarea,
                ManifestFieldType::Text |
                ManifestFieldType::String   => ConfigFieldKind::Text,
            };

            let default_value = v.default.as_deref().map(|d| {
                if v.field_type == ManifestFieldType::Port {
                    d.parse::<u16>().map(ConfigValue::Port).unwrap_or_else(|_| ConfigValue::Text(d.to_string()))
                } else if v.field_type == ManifestFieldType::Bool {
                    ConfigValue::Bool(d == "true" || d == "1" || d == "yes")
                } else {
                    ConfigValue::Text(d.to_string())
                }
            }).unwrap_or(ConfigValue::Empty);

            let mut field = ConfigField::new(
                &v.name,
                v.display_label(),
                &v.description,
                kind,
            )
            .with_value(default_value);

            if v.required {
                field = field.required();
            }
            if v.needs_restart {
                field = field.needs_restart();
            }
            field
        }).collect()
    }

    fn apply_config(&mut self, key: &str, value: ConfigValue) -> Result<(), FsError> {
        self.config_overrides.insert(key.to_string(), value);
        Ok(())
    }

    fn check_health(&self) -> PackageHealth {
        if !self.is_installed() {
            return PackageHealth::not_installed();
        }

        let mut checks = Vec::new();

        // Check 1: config file readable (if path is set)
        if let Some(path) = self.config_path() {
            if path.is_empty() {
                checks.push(HealthCheck::ok_with("config file", "No config path set"));
            } else if std::path::Path::new(path).exists() {
                checks.push(HealthCheck::ok("config file readable"));
            } else {
                checks.push(HealthCheck::fail(
                    "config file readable",
                    format!("Config file not found: {path}"),
                ));
            }
        }

        // Check 2: data directory exists (if path is set)
        if let Some(path) = self.data_path() {
            if !path.is_empty() {
                if std::path::Path::new(path).is_dir() {
                    checks.push(HealthCheck::ok("data directory exists"));
                } else {
                    checks.push(HealthCheck::fail(
                        "data directory exists",
                        format!("Data directory not found: {path}"),
                    ));
                }
            }
        }

        if checks.is_empty() {
            checks.push(HealthCheck::ok("installed"));
        }

        PackageHealth::new(checks)
    }

    fn can_persist(&self) -> bool {
        // A package can be persisted via systemd when it declares a service unit.
        // App packages: check for [app.service]. Container packages: always yes.
        // Bot/Bridge: always yes (they run as daemons).
        // This is manifest-driven — no match on PackageType.
        match &self.manifest.package.package_type {
            PackageType::App => self.manifest.app
                .as_ref()
                .map(|a| a.service.is_some())
                .unwrap_or(false),
            PackageType::Container => true,
            PackageType::Bot | PackageType::Bridge => true,
            _ => false,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::PackageMeta;

    fn make_manifest(id: &str, pkg_type: PackageType) -> ApiManifest {
        ApiManifest {
            package: PackageMeta {
                id:           id.into(),
                name:         id.to_string(),
                version:      "0.1.0".into(),
                description:  String::new(),
                category:     String::new(),
                license:      String::new(),
                author:       String::new(),
                tags:         vec![],
                icon:         String::new(),
                package_type: pkg_type,
                channel:      Default::default(),
                origin:       Default::default(),
            },
            source:    None,
            app:       None,
            container: None,
            files:     Default::default(),
            hooks:     Default::default(),
            requires:  Default::default(),
            bundle:    None,
            variables: vec![],
            setup:     None,
            contract:  None,
        }
    }

    #[test]
    fn not_installed_by_default() {
        let p = Package::from_manifest(make_manifest("test", PackageType::App));
        assert!(!p.is_installed());
        assert_eq!(p.run_status(), RunStatus::NotInstalled);
    }

    #[test]
    fn installed_after_set_installed() {
        let mut p = Package::from_manifest(make_manifest("test", PackageType::App));
        p.set_installed(InstalledRecord::new("0.1.0", "2026-03-22"));
        assert!(p.is_installed());
        assert_eq!(p.run_status(), RunStatus::Stopped);
    }

    #[test]
    fn health_not_installed() {
        let p = Package::from_manifest(make_manifest("test", PackageType::App));
        assert!(!p.check_health().is_healthy());
    }

    #[test]
    fn health_installed_no_paths() {
        let mut p = Package::from_manifest(make_manifest("test", PackageType::App));
        p.set_installed(InstalledRecord::new("0.1.0", "2026-03-22"));
        // No paths set — base check passes.
        assert!(p.check_health().is_healthy());
    }

    #[test]
    fn can_start_and_stop() {
        let mut p = Package::from_manifest(make_manifest("svc", PackageType::Container));
        p.set_installed(InstalledRecord::new("0.1.0", "2026-03-22"));
        assert!(p.can_start());
        assert!(!p.can_stop());

        p.set_status(RunStatus::Running);
        assert!(!p.can_start());
        assert!(p.can_stop());
    }

    #[test]
    fn can_persist_for_container_types() {
        let mut p = Package::from_manifest(make_manifest("c", PackageType::Container));
        p.set_installed(InstalledRecord::new("0.1.0", "2026-03-22"));
        assert!(p.can_persist());

        let app = Package::from_manifest(make_manifest("a", PackageType::App));
        assert!(!app.can_persist());
    }

    #[test]
    fn apply_config_stores_override() {
        let mut p = Package::from_manifest(make_manifest("test", PackageType::App));
        p.apply_config("port", ConfigValue::Port(8080)).unwrap();
        assert_eq!(
            p.config_overrides.get("port"),
            Some(&ConfigValue::Port(8080))
        );
    }
}
