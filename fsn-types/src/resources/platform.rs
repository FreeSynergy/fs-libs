//! Platform and feature requirements for store resources.
//!
//! Resources can declare which OS and system features they require.
//! The installer checks these against detected system info before proceeding.
//!
//! In `ResourceMeta`, declare platform requirements as:
//! ```toml
//! [platform]
//! os = "linux"
//! requires = ["systemd", "podman"]
//! ```
//!
//! In store catalog `tags`, use shorthand:
//! `platform:linux`, `requires:systemd`, `requires:podman`
//! The `platform_filter_from_tags()` helper parses these.

use serde::{Deserialize, Serialize};

// ── OsFamily ─────────────────────────────────────────────────────────────────

/// Required operating system family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum OsFamily {
    Linux,
    MacOs,
    Windows,
    /// Compatible with any OS (default).
    #[default]
    Any,
}

impl OsFamily {
    /// Badge shown in the store UI, e.g. `"Linux only"`.
    pub fn badge(self) -> &'static str {
        match self {
            OsFamily::Linux   => "Linux only",
            OsFamily::MacOs   => "macOS only",
            OsFamily::Windows => "Windows only",
            OsFamily::Any     => "",
        }
    }

    /// Parse from a tag value like `"linux"`, `"macos"`, `"windows"`.
    pub fn from_tag(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "linux"   => Some(OsFamily::Linux),
            "macos" | "mac_os" | "darwin" => Some(OsFamily::MacOs),
            "windows" => Some(OsFamily::Windows),
            "any"     => Some(OsFamily::Any),
            _         => None,
        }
    }
}

// ── RequiredFeature ───────────────────────────────────────────────────────────

/// A system feature a store resource may require.
///
/// Mirrors `fsn_sysinfo::Feature` but lives in `fsn-types` so the Store UI
/// can use it without depending on `fsn-sysinfo`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequiredFeature {
    Systemd,
    Pam,
    Launchd,
    WindowsServices,
    Podman,
    Docker,
    Git,
    Ssh,
    Smartctl,
}

impl RequiredFeature {
    /// Human-readable name shown in the store badge.
    pub fn label(self) -> &'static str {
        match self {
            RequiredFeature::Systemd         => "systemd",
            RequiredFeature::Pam             => "PAM",
            RequiredFeature::Launchd         => "launchd",
            RequiredFeature::WindowsServices => "Windows Services",
            RequiredFeature::Podman          => "Podman",
            RequiredFeature::Docker          => "Docker",
            RequiredFeature::Git             => "Git",
            RequiredFeature::Ssh             => "SSH",
            RequiredFeature::Smartctl        => "smartmontools",
        }
    }

    /// Parse from a tag value like `"systemd"`, `"podman"`.
    pub fn from_tag(s: &str) -> Option<Self> {
        match s.to_lowercase().replace('-', "_").as_str() {
            "systemd"          => Some(RequiredFeature::Systemd),
            "pam"              => Some(RequiredFeature::Pam),
            "launchd"          => Some(RequiredFeature::Launchd),
            "windows_services" => Some(RequiredFeature::WindowsServices),
            "podman"           => Some(RequiredFeature::Podman),
            "docker"           => Some(RequiredFeature::Docker),
            "git"              => Some(RequiredFeature::Git),
            "ssh"              => Some(RequiredFeature::Ssh),
            "smartctl" | "smart" => Some(RequiredFeature::Smartctl),
            _                  => None,
        }
    }
}

// ── PlatformFilter ────────────────────────────────────────────────────────────

/// Platform constraints declared by a store resource.
///
/// When `platform` is `None` in `ResourceMeta`, the resource is compatible
/// with all platforms.  When present, the store checks OS family and feature
/// requirements before allowing installation.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlatformFilter {
    /// Required OS family (`any` = no restriction).
    #[serde(default)]
    pub os: OsFamily,
    /// System features that must be present (e.g. `["systemd", "podman"]`).
    #[serde(default)]
    pub requires: Vec<RequiredFeature>,
}

impl PlatformFilter {
    /// A filter that matches every platform.
    pub fn any() -> Self {
        PlatformFilter { os: OsFamily::Any, requires: vec![] }
    }

    /// Linux-only, no additional feature requirements.
    pub fn linux_only() -> Self {
        PlatformFilter { os: OsFamily::Linux, requires: vec![] }
    }

    /// Returns `true` when the given platform satisfies all constraints.
    ///
    /// - `current_os`         — the host OS family
    /// - `available_features` — features detected by `fsn-sysinfo`
    pub fn is_satisfied_by(&self, current_os: OsFamily, available_features: &[RequiredFeature]) -> bool {
        let os_ok = self.os == OsFamily::Any || self.os == current_os;
        let features_ok = self.requires.iter().all(|req| available_features.contains(req));
        os_ok && features_ok
    }

    /// Returns human-readable descriptions of every unmet requirement.
    /// Empty when all requirements are satisfied.
    pub fn unmet(&self, current_os: OsFamily, available: &[RequiredFeature]) -> Vec<String> {
        let mut out = Vec::new();
        if self.os != OsFamily::Any && self.os != current_os {
            out.push(self.os.badge().to_owned());
        }
        for req in &self.requires {
            if !available.contains(req) {
                out.push(format!("{} not available", req.label()));
            }
        }
        out
    }
}

// ── Tag parsing ───────────────────────────────────────────────────────────────

/// Build a `PlatformFilter` from a list of store tags.
///
/// Recognized tag formats:
/// - `platform:linux`     → `os = OsFamily::Linux`
/// - `platform:macos`     → `os = OsFamily::MacOs`
/// - `platform:windows`   → `os = OsFamily::Windows`
/// - `requires:systemd`   → adds `RequiredFeature::Systemd`
/// - `requires:podman`    → adds `RequiredFeature::Podman`
/// - … (all `RequiredFeature` variants)
///
/// Returns `None` if no platform or requires tags are present.
pub fn platform_filter_from_tags(tags: &[String]) -> Option<PlatformFilter> {
    let mut os = OsFamily::Any;
    let mut requires = Vec::new();

    for tag in tags {
        if let Some(val) = tag.strip_prefix("platform:") {
            if let Some(family) = OsFamily::from_tag(val) {
                os = family;
            }
        } else if let Some(val) = tag.strip_prefix("requires:") {
            if let Some(feat) = RequiredFeature::from_tag(val) {
                requires.push(feat);
            }
        }
    }

    if os == OsFamily::Any && requires.is_empty() {
        None
    } else {
        Some(PlatformFilter { os, requires })
    }
}
