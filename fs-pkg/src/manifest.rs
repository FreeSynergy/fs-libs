// manifest.rs — ApiManifest: the TOML manifest format for FreeSynergy packages.
//
// Every installable package (module, plugin, theme, …) ships a `manifest.toml`
// that describes its identity, source, files, hooks, and runtime requirements.
//
// TOML structure:
//
//   [package]        — identity (id, name, version, …)
//   [source]         — where to get the package (OCI | store | local)
//   [files]          — files this package installs
//   [hooks]          — shell commands to run during install/remove lifecycle
//   [requires]       — other packages or capabilities this package needs

use std::path::Path;

use fs_error::FsError;
use fs_types::StrLabel;
use serde::{Deserialize, Serialize};

use crate::channel::ReleaseChannel;
use crate::oci::OciRef;

// ── PackageId ─────────────────────────────────────────────────────────────────

/// A unique package identifier (e.g. `"proxy/zentinel"`, `"iam/kanidm"`).
///
/// Wraps a `String` to ensure all package IDs are typed values rather than
/// plain strings.  Access the underlying text via `Display`, `Deref`, or
/// `as_str()`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PackageId(String);

impl PackageId {
    /// Construct a `PackageId` from any string-like value.
    pub fn new(s: impl Into<String>) -> Self { Self(s.into()) }

    /// Borrows the identifier as a `&str`.
    pub fn as_str(&self) -> &str { &self.0 }
}

impl std::fmt::Display for PackageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::ops::Deref for PackageId {
    type Target = str;
    fn deref(&self) -> &str { &self.0 }
}

impl AsRef<str> for PackageId {
    fn as_ref(&self) -> &str { &self.0 }
}

impl From<String> for PackageId {
    fn from(s: String) -> Self { Self(s) }
}

impl From<&str> for PackageId {
    fn from(s: &str) -> Self { Self(s.to_owned()) }
}

impl PartialEq<str> for PackageId {
    fn eq(&self, other: &str) -> bool { self.0 == other }
}

impl PartialEq<&str> for PackageId {
    fn eq(&self, other: &&str) -> bool { self.0 == *other }
}

impl<'a> From<&'a PackageId> for PackageId {
    fn from(id: &'a PackageId) -> Self { id.clone() }
}

// ── PackageType ───────────────────────────────────────────────────────────────

/// The category of a FreeSynergy package — determines how it is installed and managed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackageType {
    /// A FreeSynergy core application (Node, Desktop, Conductor, …).
    #[default]
    App,
    /// A containerized application running via Podman/Docker.
    Container,
    /// A meta-package that bundles several packages together.
    Bundle,
    /// A language pack (.ftl snippets).
    Language,
    /// A visual theme for the FreeSynergy desktop or TUI.
    Theme,
    /// A UI widget that can be embedded in the desktop.
    Widget,
    /// An autonomous bot that connects to the message bus.
    Bot,
    /// A bridge connector to an external protocol (Matrix, Telegram, …).
    Bridge,
    /// A scheduled or one-shot background task.
    Task,
}

impl StrLabel for PackageType {
    fn label(&self) -> &'static str {
        match self {
            Self::App       => "app",
            Self::Container => "container",
            Self::Bundle    => "bundle",
            Self::Language  => "language",
            Self::Theme     => "theme",
            Self::Widget    => "widget",
            Self::Bot       => "bot",
            Self::Bridge    => "bridge",
            Self::Task      => "task",
        }
    }
}

fs_types::impl_str_label_display!(PackageType);

// ── ApiManifest ───────────────────────────────────────────────────────────────

/// The top-level `manifest.toml` for a FreeSynergy package.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiManifest {
    /// Package identity.
    pub package: PackageMeta,

    /// Package source (where to fetch it from).
    #[serde(default)]
    pub source: Option<PackageSource>,

    /// Files this package installs.
    #[serde(default)]
    pub files: PackageFiles,

    /// Lifecycle hooks.
    #[serde(default)]
    pub hooks: PackageHooks,

    /// Runtime requirements.
    #[serde(default)]
    pub requires: PackageRequires,

    /// Bundle metadata — only present when `package.package_type == Bundle`.
    ///
    /// Declares which packages this bundle includes and which are optional.
    #[serde(default)]
    pub bundle: Option<BundleManifest>,
}

// ── BundleManifest ────────────────────────────────────────────────────────────

/// Bundle metadata (`[bundle]`) — only for `PackageType::Bundle`.
///
/// A bundle is a meta-package that groups other packages together, similar to
/// `dnf group install`. The store resolves all `packages` automatically;
/// `optional` packages are presented to the user as choices.
///
/// # Example (TOML)
///
/// ```toml
/// [package]
/// id   = "server-minimal"
/// name = "Server Minimal"
/// type = "bundle"
///
/// [bundle]
/// packages = ["node", "conductor", "zentinel", "kanidm"]
/// optional  = ["forgejo", "outline", "stalwart"]
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BundleManifest {
    /// Package IDs that are always installed as part of this bundle.
    #[serde(default)]
    pub packages: Vec<String>,

    /// Package IDs that are offered to the user as optional additions.
    #[serde(default)]
    pub optional: Vec<String>,
}

impl ApiManifest {
    /// Parse a manifest from a TOML string.
    pub fn from_toml(s: &str) -> Result<Self, FsError> {
        toml::from_str(s)
            .map_err(|e| FsError::parse(format!("manifest parse error: {e}")))
    }

    /// Parse a manifest from a file.
    pub fn from_file(path: &Path) -> Result<Self, FsError> {
        let s = std::fs::read_to_string(path)
            .map_err(|e| FsError::internal(format!("cannot read {}: {e}", path.display())))?;
        Self::from_toml(&s)
    }

    /// Serialize to TOML.
    pub fn to_toml(&self) -> Result<String, FsError> {
        toml::to_string_pretty(self)
            .map_err(|e| FsError::internal(format!("manifest serialize error: {e}")))
    }
}

// ── PackageMeta ───────────────────────────────────────────────────────────────

/// Identity block (`[package]`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageMeta {
    /// Unique identifier (e.g. `"proxy/zentinel"`).
    pub id: PackageId,

    /// Display name.
    pub name: String,

    /// Semantic version string.
    pub version: String,

    /// Short description.
    #[serde(default)]
    pub description: String,

    /// Package category (e.g. `"deploy.proxy"`).
    #[serde(default)]
    pub category: String,

    /// SPDX license identifier.
    #[serde(default)]
    pub license: String,

    /// Author name or email.
    #[serde(default)]
    pub author: String,

    /// Tags for search/filtering.
    #[serde(default)]
    pub tags: Vec<String>,

    /// Package icon (emoji or icon name). Empty string if not set.
    #[serde(default)]
    pub icon: String,

    /// Package type — controls install behaviour and display.
    #[serde(default)]
    pub package_type: PackageType,

    /// Release channel this manifest targets.
    #[serde(default)]
    pub channel: ReleaseChannel,
}

// ── PackageSource ─────────────────────────────────────────────────────────────

/// Where to fetch this package from.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum PackageSource {
    /// Pull a container image from an OCI registry.
    Oci {
        /// OCI image reference (e.g. `"ghcr.io/freesynergy/zentinel:0.1.0"`).
        image: String,
    },

    /// Download from the FreeSynergy Store (namespace + package ID).
    Store {
        /// Store namespace (e.g. `"Node"`).
        namespace: String,
    },

    /// Use a local directory (development / offline).
    Local {
        /// Absolute or workspace-relative path.
        path: String,
    },
}

impl PackageSource {
    /// Parse the OCI image reference for `Oci` sources.
    pub fn oci_ref(&self) -> Option<Result<OciRef, FsError>> {
        match self {
            Self::Oci { image } => Some(OciRef::parse(image)),
            _ => None,
        }
    }
}

// ── FileMapping ───────────────────────────────────────────────────────────────

/// One source → destination file entry in a package's `[files.*]` block.
///
/// `source` is relative to the package root.
/// `dest` is the absolute target path on the system; may contain
/// `{data_root}` / `{config_root}` placeholders expanded at install time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMapping {
    /// Source path relative to the package root (e.g. `"zentinel.kdl.j2"`).
    pub source: String,
    /// Destination path on the target system (e.g. `"{config_root}/zentinel.kdl"`).
    pub dest: String,
}

// ── PackageFiles ──────────────────────────────────────────────────────────────

/// Files this package installs (`[files]`).
///
/// Each subsection holds typed [`FileMapping`] entries instead of raw
/// `HashMap<String, String>` so callers receive objects, not plain data.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PackageFiles {
    /// Config files to write.
    #[serde(default)]
    pub config: Vec<FileMapping>,

    /// Systemd unit files (Quadlets) to install (typically to `/etc/containers/systemd/`).
    #[serde(default)]
    pub units: Vec<FileMapping>,

    /// Arbitrary data files.
    #[serde(default)]
    pub data: Vec<FileMapping>,
}

impl PackageFiles {
    /// Iterator over **all** file mappings across config, units, and data (in that order).
    pub fn all(&self) -> impl Iterator<Item = &FileMapping> {
        self.config.iter().chain(self.units.iter()).chain(self.data.iter())
    }

    /// Iterator over all **destination** paths (may contain placeholders).
    pub fn all_dests(&self) -> impl Iterator<Item = &str> {
        self.all().map(|f| f.dest.as_str())
    }
}

// ── PackageHooks ──────────────────────────────────────────────────────────────

/// Lifecycle hooks (`[hooks]`).
///
/// All hooks are shell command strings executed in order.
/// A non-zero exit code aborts the install/remove operation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PackageHooks {
    /// Run before install begins.
    #[serde(default)]
    pub pre_install: Vec<String>,

    /// Run after all files are written.
    #[serde(default)]
    pub post_install: Vec<String>,

    /// Run before files are removed.
    #[serde(default)]
    pub pre_remove: Vec<String>,

    /// Run after files are removed.
    #[serde(default)]
    pub post_remove: Vec<String>,

    /// Run before an upgrade (old version still active).
    #[serde(default)]
    pub pre_upgrade: Vec<String>,

    /// Run after an upgrade (new version active).
    #[serde(default)]
    pub post_upgrade: Vec<String>,
}

// ── PackageRequires ───────────────────────────────────────────────────────────

/// Runtime requirements (`[requires]`).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PackageRequires {
    /// Other package IDs that must be installed first.
    #[serde(default)]
    pub packages: Vec<String>,

    /// System capabilities required (e.g. `"podman"`, `"systemd"`).
    #[serde(default)]
    pub capabilities: Vec<String>,

    /// Minimum FreeSynergy version.
    #[serde(default)]
    pub min_fs_version: Option<String>,
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const MINIMAL_TOML: &str = r#"
[package]
id          = "proxy/zentinel"
name        = "Zentinel"
version     = "0.1.0"
description = "Reverse proxy module"
category    = "deploy.proxy"
license     = "MIT"
author      = "FreeSynergy.Net"
"#;

    const FULL_TOML: &str = r#"
[package]
id          = "proxy/zentinel"
name        = "Zentinel"
version     = "0.1.0"
description = "Reverse proxy module"
category    = "deploy.proxy"
license     = "MIT"
author      = "FreeSynergy.Net"
tags        = ["proxy", "tls"]

[source]
type      = "oci"
image     = "ghcr.io/freesynergy/zentinel:0.1.0"

[[files.config]]
source = "zentinel.kdl.j2"
dest   = "{config_root}/zentinel.kdl"

[[files.units]]
source = "zentinel.container"
dest   = "/etc/containers/systemd/zentinel.container"

[hooks]
pre_install   = ["mkdir -p /srv/data/zentinel"]
post_install  = ["systemctl daemon-reload"]
pre_remove    = ["systemctl stop zentinel"]
post_remove   = ["systemctl daemon-reload"]

[requires]
packages      = ["iam/kanidm"]
capabilities  = ["podman", "systemd"]
"#;

    #[test]
    fn parse_minimal() {
        let m = ApiManifest::from_toml(MINIMAL_TOML).unwrap();
        assert_eq!(m.package.id, "proxy/zentinel");
        assert_eq!(m.package.version, "0.1.0");
    }

    #[test]
    fn parse_full() {
        let m = ApiManifest::from_toml(FULL_TOML).unwrap();
        assert_eq!(m.package.id, "proxy/zentinel");
        assert_eq!(m.package.tags, vec!["proxy", "tls"]);

        // Source
        let src = m.source.unwrap();
        let oci = src.oci_ref().unwrap().unwrap();
        assert_eq!(oci.tag(), Some("0.1.0"));

        // Files
        assert!(m.files.config.iter().any(|f| f.source == "zentinel.kdl.j2"));
        assert!(m.files.units.iter().any(|f| f.source == "zentinel.container"));

        // Hooks
        assert_eq!(m.hooks.pre_install, vec!["mkdir -p /srv/data/zentinel"]);
        assert_eq!(m.hooks.post_install, vec!["systemctl daemon-reload"]);

        // Requires
        assert_eq!(m.requires.packages, vec!["iam/kanidm"]);
        assert!(m.requires.capabilities.contains(&"podman".to_string()));
    }

    #[test]
    fn toml_roundtrip() {
        let m = ApiManifest::from_toml(FULL_TOML).unwrap();
        let serialized = m.to_toml().unwrap();
        let back = ApiManifest::from_toml(&serialized).unwrap();
        assert_eq!(back.package.id, m.package.id);
        assert_eq!(back.package.version, m.package.version);
    }

    #[test]
    fn invalid_toml_returns_error() {
        assert!(ApiManifest::from_toml("not valid toml ===").is_err());
    }
}
