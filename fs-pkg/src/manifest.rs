// manifest.rs — ApiManifest: the TOML manifest format for FreeSynergy packages.
//
// Every installable package (module, plugin, theme, …) ships a `manifest.toml`
// that describes its identity, source, files, hooks, and runtime requirements.
//
// TOML structure:
//
//   [package]        — identity (id, name, version, …)
//   [source]         — where to get the package (OCI | GithubRelease | local)
//   [app]            — native binary config (only for PackageType::App)
//   [container]      — container config (only for PackageType::Container)
//   [files]          — files this package installs
//   [hooks]          — shell commands to run during install/remove lifecycle
//   [requires]       — other packages or capabilities this package needs
//   [[variables]]    — user-configurable variables (drives Config tab)
//   [setup]          — setup wizard fields (shown before first install)
//   [contract]       — reverse proxy integration (routes via Zentinel)

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
    /// A native binary application (downloaded from GitHub Releases, runs as a systemd service).
    #[default]
    App,
    /// A containerized application running via Podman/Docker (Quadlet).
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
///
/// Type-specific sections (`app`, `container`) are optional and only present
/// for the matching `PackageType`. The Manager reads `variables` to populate
/// the Config tab; `setup` drives the first-install wizard; `contract` tells
/// Zentinel how to proxy traffic for this package.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiManifest {
    /// Package identity.
    pub package: PackageMeta,

    /// Package source (where to fetch it from).
    #[serde(default)]
    pub source: Option<PackageSource>,

    /// Native binary section — present when `package.package_type == App`.
    #[serde(default)]
    pub app: Option<AppManifest>,

    /// Container section — present when `package.package_type == Container`.
    #[serde(default)]
    pub container: Option<ContainerManifest>,

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
    #[serde(default)]
    pub bundle: Option<BundleManifest>,

    /// User-configurable variables — drives the Config tab in the Manager.
    ///
    /// Each entry maps to one `ConfigField`. The Manager renders these fields;
    /// the package defines them via `[[variables]]` entries in the manifest.
    #[serde(default)]
    pub variables: Vec<ManifestVariable>,

    /// Setup wizard configuration — shown before first-time installation.
    #[serde(default)]
    pub setup: Option<SetupManifest>,

    /// Reverse proxy contract — how Zentinel routes traffic to this package.
    #[serde(default)]
    pub contract: Option<ContractManifest>,
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

    /// Returns the package type.
    pub fn package_type(&self) -> PackageType {
        self.package.package_type
    }

    /// Returns `true` when this manifest has a `[app]` section.
    pub fn is_app(&self) -> bool {
        self.app.is_some() || self.package.package_type == PackageType::App
    }

    /// Returns `true` when this manifest has a `[container]` section.
    pub fn is_container(&self) -> bool {
        self.container.is_some() || self.package.package_type == PackageType::Container
    }

    /// Whether this package can be registered as a persistent systemd service.
    ///
    /// - `App`: only when `[app.service]` is declared
    /// - `Container`, `Bot`, `Bridge`: always (they run as long-lived daemons)
    /// - All other types: no
    pub fn can_persist(&self) -> bool {
        match self.package.package_type {
            PackageType::App => self.app.as_ref().map(|a| a.service.is_some()).unwrap_or(false),
            PackageType::Container | PackageType::Bot | PackageType::Bridge => true,
            _ => false,
        }
    }
}

// ── PackageSource ─────────────────────────────────────────────────────────────

/// Where to fetch this package from.
///
/// The [`FetchStrategy`] in `installer.rs` selects the download method based
/// on which variant is present in `[source]`.
///
/// [`FetchStrategy`]: crate::installer::FetchStrategy
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PackageSource {
    /// Pull a container image from an OCI registry.
    Oci {
        /// OCI image reference (e.g. `"ghcr.io/freesynergy/zentinel:0.1.0"`).
        image: String,
    },

    /// Download a pre-built binary from a GitHub Releases page.
    ///
    /// Used for native App packages built from FreeSynergy forks
    /// (kanidm, tuwunel, stalwart, mistral, …).
    ///
    /// The installer resolves the download URL as:
    /// `https://github.com/{repo}/releases/download/v{version}/{artifact}`
    /// where `{version}` is replaced with `package.version`.
    GithubRelease {
        /// GitHub repository (e.g. `"FreeSynergy/fs-kanidm"`).
        repo: String,

        /// Artifact file name (e.g. `"kanidmd-x86_64-linux.tar.gz"`).
        ///
        /// May contain `{version}` which is substituted at download time.
        artifact: String,

        /// Optional SHA-256 hex digest for integrity verification.
        #[serde(default)]
        checksum: Option<String>,
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

    /// Returns the GitHub repo for `GithubRelease` sources.
    pub fn github_repo(&self) -> Option<&str> {
        match self {
            Self::GithubRelease { repo, .. } => Some(repo),
            _ => None,
        }
    }

    /// Returns the artifact name for `GithubRelease` sources.
    pub fn github_artifact(&self) -> Option<&str> {
        match self {
            Self::GithubRelease { artifact, .. } => Some(artifact),
            _ => None,
        }
    }
}

// ── AppManifest ───────────────────────────────────────────────────────────────

/// Native binary section (`[app]`).
///
/// Present when `package.package_type == App`. Describes how the binary is
/// installed and managed on the host system.
///
/// # Example (TOML)
///
/// ```toml
/// [app]
/// binary     = "kanidmd"
/// service    = "kanidm.service"
/// data_dir   = "{data_root}/kanidm"
/// config_dir = "{config_root}/kanidm"
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppManifest {
    /// Primary binary name (e.g. `"kanidmd"`).
    pub binary: String,

    /// Systemd service unit name (e.g. `"kanidm.service"`).
    /// `None` for user-managed or manually started processes.
    #[serde(default)]
    pub service: Option<String>,

    /// Default data directory. May contain `{data_root}` placeholder.
    #[serde(default)]
    pub data_dir: String,

    /// Default config directory. May contain `{config_root}` placeholder.
    #[serde(default)]
    pub config_dir: String,
}

// ── ContainerManifest ─────────────────────────────────────────────────────────

/// Container section (`[container]`).
///
/// Present when `package.package_type == Container`. Describes the OCI image,
/// volumes, environment variables, and health check for Podman/Quadlet deployment.
///
/// # Example (TOML)
///
/// ```toml
/// [container]
/// image     = "ghcr.io/matrix-construct/tuwunel"
/// image_tag = "latest"
/// volumes   = ["{data_root}/tuwunel:/var/lib/tuwunel:Z"]
///
/// [container.healthcheck]
/// cmd          = "curl -fsS http://localhost:8008/_matrix/client/versions"
/// interval     = "30s"
/// timeout      = "5s"
/// retries      = 3
/// start_period = "30s"
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerManifest {
    /// OCI image name (e.g. `"docker.io/kanidm/server"`).
    pub image: String,

    /// Image tag (e.g. `"latest"` or `"1.4.2"`).
    #[serde(default = "default_image_tag")]
    pub image_tag: String,

    /// Volume mounts — may contain template vars like `{data_root}`.
    #[serde(default)]
    pub volumes: Vec<String>,

    /// Published port mappings (e.g. `"8008:8008"`).
    #[serde(default)]
    pub ports: Vec<String>,

    /// Container health check.
    #[serde(default)]
    pub healthcheck: Option<ContainerHealthCheck>,

    /// Environment variables passed to the container.
    /// Values may contain template vars (e.g. `{service_domain}`).
    #[serde(default)]
    pub environment: std::collections::HashMap<String, String>,
}

fn default_image_tag() -> String { "latest".into() }

/// Container health check configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerHealthCheck {
    /// Shell command to run (exit 0 = healthy).
    pub cmd: String,

    /// Interval between checks (e.g. `"30s"`).
    #[serde(default = "default_interval")]
    pub interval: String,

    /// Maximum time to wait for a check (e.g. `"5s"`).
    #[serde(default = "default_timeout")]
    pub timeout: String,

    /// Number of consecutive failures before marking unhealthy.
    #[serde(default = "default_retries")]
    pub retries: u32,

    /// Wait time before first check after container start.
    #[serde(default = "default_start_period")]
    pub start_period: String,
}

fn default_interval()     -> String { "30s".into() }
fn default_timeout()      -> String { "5s".into() }
fn default_retries()      -> u32    { 3 }
fn default_start_period() -> String { "30s".into() }

// ── ManifestVariable ──────────────────────────────────────────────────────────

/// A user-configurable variable declared in the manifest (`[[variables]]`).
///
/// Maps directly to a [`ConfigField`] rendered in the Manager's Config tab.
/// Every variable MUST have a non-empty `description` — the Manager will
/// show a warning for variables without help text.
///
/// [`ConfigField`]: crate::manageable::ConfigField
///
/// # Example (TOML)
///
/// ```toml
/// [[variables]]
/// name        = "TUWUNEL_SERVER_NAME"
/// description = "Matrix server name — the domain part of every user MXID."
/// required    = true
///
/// [[variables]]
/// name        = "TUWUNEL_ALLOW_REGISTRATION"
/// description = "Allow new account registrations."
/// field_type  = "bool"
/// default     = "false"
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ManifestVariable {
    /// Variable name — becomes the `ConfigField::key` (e.g. `"TUWUNEL_PORT"`).
    pub name: String,

    /// Human-readable description shown as help text in the Config tab.
    ///
    /// MANDATORY — every variable must explain what it controls.
    #[serde(default)]
    pub description: String,

    /// Whether this variable must be set before the package can start.
    #[serde(default)]
    pub required: bool,

    /// Default value (stored as string; converted to the appropriate `ConfigValue`).
    #[serde(default)]
    pub default: Option<String>,

    /// Input widget type. Defaults to `text`.
    #[serde(default)]
    pub field_type: ManifestFieldType,

    /// Optional label override — falls back to `name` if empty.
    #[serde(default)]
    pub label: String,

    /// Whether a restart is required when this variable changes.
    #[serde(default)]
    pub needs_restart: bool,
}

impl ManifestVariable {
    /// Returns the display label — `label` if set, otherwise `name`.
    pub fn display_label(&self) -> &str {
        if self.label.is_empty() { &self.name } else { &self.label }
    }

    /// Returns `true` when the description is non-empty.
    pub fn has_description(&self) -> bool {
        !self.description.is_empty()
    }
}

/// Input field type for manifest variables — determines the Config tab widget.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ManifestFieldType {
    /// Single-line text input (default).
    #[default]
    Text,
    /// Boolean toggle.
    Bool,
    /// Port number (validated 1–65535).
    Port,
    /// Password input (value masked).
    Password,
    /// Secret — like password; can be auto-generated.
    Secret,
    /// File system path with optional picker.
    Path,
    /// Multi-line text area.
    Textarea,
    /// Plain string (alias for text — used in existing manifests).
    String,
}

// ── SetupManifest ─────────────────────────────────────────────────────────────

/// Setup wizard configuration (`[setup]`).
///
/// Describes the fields shown to the user in the first-install wizard.
/// These are higher-level than `variables` — they represent choices the
/// user makes before the package is started for the first time.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SetupManifest {
    /// Setup wizard fields shown before first install.
    #[serde(default, rename = "fields")]
    pub fields: Vec<SetupField>,
}

/// One field in the setup wizard (`[[setup.fields]]`).
///
/// Every field MUST have a non-empty `help` string — this is shown in the
/// wizard's right-side help panel. Fields without help are marked with a
/// warning in the Manager so package authors are notified of the gap.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupField {
    /// Variable key this field writes to (matches `[[variables]] name`).
    pub key: String,

    /// Human-readable label shown next to the input widget.
    pub label: String,

    /// Short inline description shown below the input (1–2 sentences).
    #[serde(default)]
    pub description: String,

    /// Full help text shown in the right-side help panel. MANDATORY.
    ///
    /// Explain: what is this for, what happens if it is wrong, and what
    /// a sensible value looks like. Non-technical users must understand this.
    #[serde(default)]
    pub help: String,

    /// Input widget type.
    #[serde(default)]
    pub field_type: ManifestFieldType,

    /// Default value shown pre-filled in the wizard.
    ///
    /// Non-technical users can just click OK if the default is sensible.
    /// Mandatory fields MUST have a default or `auto_generate = true`.
    #[serde(default)]
    pub default: Option<String>,

    /// When this field should be shown / re-applied.
    ///
    /// Serialized as a list of trigger names:
    /// `["first_install", "on_config_save", "on_start"]`
    ///
    /// Defaults to `["first_install"]` when empty.
    #[serde(default)]
    pub triggers: Vec<String>,

    /// Skip showing this field if the variable is already set in the context.
    ///
    /// Useful for auto-generated secrets: once generated, don't show again.
    #[serde(default)]
    pub skip_if_set: bool,

    /// Auto-generate a random value for this field (passwords/secrets).
    ///
    /// The generated value is shown to the user (highlighted) so they can
    /// copy it. They may replace it with their own value if desired.
    #[serde(default)]
    pub auto_generate: bool,
}

impl SetupField {
    /// Returns true when the help text is present (non-empty).
    ///
    /// The Manager logs a warning for fields where this returns false.
    pub fn has_help(&self) -> bool {
        !self.help.is_empty()
    }

    /// Returns the effective triggers for this field.
    ///
    /// Falls back to `["first_install"]` when none are declared.
    pub fn effective_triggers(&self) -> Vec<&str> {
        if self.triggers.is_empty() {
            vec!["first_install"]
        } else {
            self.triggers.iter().map(|s| s.as_str()).collect()
        }
    }
}

// ── ContractManifest ──────────────────────────────────────────────────────────

/// Reverse proxy contract (`[contract]`).
///
/// Describes how Zentinel (the reverse proxy) routes traffic to this package.
/// Each route entry declares a path prefix and whether it should be stripped.
///
/// # Example (TOML)
///
/// ```toml
/// [contract]
/// upstream_tls = true
///
/// [[contract.routes]]
/// id          = "main"
/// path        = "/"
/// strip       = false
/// description = "Kanidm web UI + REST API"
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContractManifest {
    /// Whether the upstream (this package) uses TLS.
    #[serde(default)]
    pub upstream_tls: bool,

    /// Routes this package handles.
    #[serde(default)]
    pub routes: Vec<ContractRoute>,
}

/// One route in the proxy contract (`[[contract.routes]]`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractRoute {
    /// Unique route identifier (e.g. `"main"`, `"api"`).
    pub id: String,

    /// Path prefix to match (e.g. `"/"`).
    pub path: String,

    /// Whether to strip the prefix before forwarding.
    #[serde(default)]
    pub strip: bool,

    /// Human-readable description of what this route serves.
    #[serde(default)]
    pub description: String,
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

// ── PackageOrigin ─────────────────────────────────────────────────────────────

/// Provenance metadata — where a package *comes from*.
///
/// Separate from [`PackageSource`] which describes *how to install* it.
/// `PackageOrigin` is displayed in the help panel and store detail view so
/// users can trace a package back to its upstream project.
///
/// # Example (TOML)
///
/// ```toml
/// [package.origin]
/// website   = "https://kanidm.com"
/// git       = "https://github.com/kanidm/kanidm"
/// docs      = "https://kanidm.com/documentation/stable/"
/// search    = "kanidm identity provider tutorial"
/// publisher = "Kanidm Project"
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PackageOrigin {
    /// Project website URL.
    #[serde(default)]
    pub website: Option<String>,

    /// Git repository URL.
    #[serde(default)]
    pub git: Option<String>,

    /// Documentation URL.
    #[serde(default)]
    pub docs: Option<String>,

    /// Engine-agnostic tutorial search query
    /// (e.g. `"kanidm identity provider tutorial"`).
    /// Used by the help panel to open a search in the browser.
    #[serde(default)]
    pub search: Option<String>,

    /// Publisher or vendor name (e.g. `"Kanidm Project"`).
    #[serde(default)]
    pub publisher: Option<String>,
}

impl PackageOrigin {
    /// `true` if at least one link is set.
    pub fn has_links(&self) -> bool {
        self.website.is_some() || self.git.is_some() || self.docs.is_some()
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

    /// Package category (e.g. `"iam"`).
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
    #[serde(default, rename = "type")]
    pub package_type: PackageType,

    /// Release channel this manifest targets.
    #[serde(default)]
    pub channel: ReleaseChannel,

    /// Provenance — where this package comes from (website, git, docs, search).
    /// Used by the help panel and store detail view.
    #[serde(default)]
    pub origin: PackageOrigin,
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
category    = "proxy"
license     = "MIT"
author      = "FreeSynergy.Net"
"#;

    const APP_TOML: &str = r#"
[package]
id          = "iam/kanidm"
name        = "Kanidm"
version     = "1.4.2"
description = "Identity and access management server"
category    = "iam"
type        = "app"
license     = "MPL-2.0"
author      = "Kal El"
tags        = ["iam", "identity", "oidc", "ldap", "webauthn"]

[package.origin]
website = "https://kanidm.com"
git     = "https://github.com/FreeSynergy/fs-kanidm"

[source]
type     = "github_release"
repo     = "FreeSynergy/fs-kanidm"
artifact = "kanidmd-x86_64-linux.tar.gz"

[app]
binary     = "kanidmd"
service    = "kanidm.service"
data_dir   = "{data_root}/kanidm"
config_dir = "{config_root}/kanidm"

[[variables]]
name        = "KANIDM_DOMAIN"
description = "The domain Kanidm serves (e.g. auth.example.com)."
required    = true
field_type  = "text"

[[variables]]
name        = "KANIDM_HTTPS_PORT"
description = "HTTPS port Kanidm listens on."
field_type  = "port"
default     = "8443"

[setup]
[[setup.fields]]
key        = "kanidm_domain"
label      = "Kanidm domain"
field_type = "text"

[contract]
upstream_tls = true

[[contract.routes]]
id          = "main"
path        = "/"
strip       = false
description = "Kanidm web UI + REST API"
"#;

    const CONTAINER_TOML: &str = r#"
[package]
id          = "chat/tuwunel"
name        = "Tuwunel"
version     = "0.1.0"
type        = "container"
description = "Matrix homeserver"
category    = "chat"

[source]
type  = "oci"
image = "ghcr.io/matrix-construct/tuwunel:latest"

[container]
image     = "ghcr.io/matrix-construct/tuwunel"
image_tag = "latest"
volumes   = ["{data_root}/tuwunel:/var/lib/tuwunel:Z"]

[container.healthcheck]
cmd          = "curl -fsS http://localhost:8008/_matrix/client/versions"
interval     = "30s"
timeout      = "5s"
retries      = 3
start_period = "30s"

[[variables]]
name        = "TUWUNEL_SERVER_NAME"
description = "Matrix server name (e.g. example.com)."
required    = true
"#;

    const BUNDLE_TOML: &str = r#"
[package]
id   = "proxy/zentinel-bundle"
name = "Zentinel Bundle"
type = "bundle"
version = "0.1.0"

[bundle]
packages = ["proxy/zentinel", "proxy/zentinel-plane"]
optional  = []
"#;

    const FULL_TOML: &str = r#"
[package]
id          = "proxy/zentinel"
name        = "Zentinel"
version     = "0.1.0"
description = "Reverse proxy module"
category    = "proxy"
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
    fn parse_app_manifest() {
        let m = ApiManifest::from_toml(APP_TOML).unwrap();
        assert_eq!(m.package.id, "iam/kanidm");
        assert_eq!(m.package.package_type, PackageType::App);
        assert!(m.is_app());

        let app = m.app.as_ref().unwrap();
        assert_eq!(app.binary, "kanidmd");
        assert_eq!(app.service.as_deref(), Some("kanidm.service"));

        let src = m.source.as_ref().unwrap();
        assert_eq!(src.github_repo(), Some("FreeSynergy/fs-kanidm"));
        assert_eq!(src.github_artifact(), Some("kanidmd-x86_64-linux.tar.gz"));

        assert_eq!(m.variables.len(), 2);
        assert_eq!(m.variables[0].name, "KANIDM_DOMAIN");
        assert!(m.variables[0].required);
        assert_eq!(m.variables[1].field_type, ManifestFieldType::Port);
        assert_eq!(m.variables[1].default.as_deref(), Some("8443"));

        let contract = m.contract.as_ref().unwrap();
        assert!(contract.upstream_tls);
        assert_eq!(contract.routes[0].id, "main");
    }

    #[test]
    fn parse_container_manifest() {
        let m = ApiManifest::from_toml(CONTAINER_TOML).unwrap();
        assert_eq!(m.package.package_type, PackageType::Container);
        assert!(m.is_container());

        let ctr = m.container.as_ref().unwrap();
        assert_eq!(ctr.image, "ghcr.io/matrix-construct/tuwunel");
        assert!(ctr.healthcheck.is_some());
    }

    #[test]
    fn parse_bundle_manifest() {
        let m = ApiManifest::from_toml(BUNDLE_TOML).unwrap();
        assert_eq!(m.package.package_type, PackageType::Bundle);
        let b = m.bundle.as_ref().unwrap();
        assert!(b.packages.contains(&"proxy/zentinel".to_string()));
        assert!(b.packages.contains(&"proxy/zentinel-plane".to_string()));
    }

    #[test]
    fn parse_full() {
        let m = ApiManifest::from_toml(FULL_TOML).unwrap();
        assert_eq!(m.package.id, "proxy/zentinel");
        assert_eq!(m.package.tags, vec!["proxy", "tls"]);

        let src = m.source.unwrap();
        let oci = src.oci_ref().unwrap().unwrap();
        assert_eq!(oci.tag(), Some("0.1.0"));

        assert!(m.files.config.iter().any(|f| f.source == "zentinel.kdl.j2"));
        assert!(m.files.units.iter().any(|f| f.source == "zentinel.container"));

        assert_eq!(m.hooks.pre_install, vec!["mkdir -p /srv/data/zentinel"]);
        assert_eq!(m.hooks.post_install, vec!["systemctl daemon-reload"]);
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

    #[test]
    fn manifest_variable_display_label() {
        let v = ManifestVariable {
            name: "MY_VAR".into(),
            label: "My Variable".into(),
            ..Default::default()
        };
        assert_eq!(v.display_label(), "My Variable");

        let v2 = ManifestVariable {
            name: "MY_VAR".into(),
            ..Default::default()
        };
        assert_eq!(v2.display_label(), "MY_VAR");
    }

    #[test]
    fn github_release_source() {
        let src = PackageSource::GithubRelease {
            repo:     "FreeSynergy/fs-kanidm".into(),
            artifact: "kanidmd-x86_64-linux.tar.gz".into(),
            checksum: Some("abc123".into()),
        };
        assert_eq!(src.github_repo(),     Some("FreeSynergy/fs-kanidm"));
        assert_eq!(src.github_artifact(), Some("kanidmd-x86_64-linux.tar.gz"));
        assert!(src.oci_ref().is_none());
    }
}
