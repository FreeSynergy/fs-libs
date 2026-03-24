//! Runtime feature detection: systemd, Podman, PAM, Git, SSH, etc.

use std::path::Path;

use serde::{Deserialize, Serialize};

/// A detectable system feature or installed tool.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Feature {
    /// systemd is the init system (Linux only).
    Systemd,
    /// PAM (Pluggable Authentication Modules) is available.
    Pam,
    /// launchd is the init system (macOS only).
    Launchd,
    /// Windows Services are available (Windows only).
    WindowsServices,
    /// Podman container runtime is installed.
    Podman,
    /// Docker container runtime is installed.
    Docker,
    /// Git version control system is installed.
    Git,
    /// OpenSSH client is installed.
    Ssh,
    /// smartmontools (`smartctl`) is installed.
    Smartctl,
}

impl Feature {
    /// Human-readable display label.
    pub fn label(self) -> &'static str {
        match self {
            Feature::Systemd => "systemd",
            Feature::Pam => "PAM",
            Feature::Launchd => "launchd",
            Feature::WindowsServices => "Windows Services",
            Feature::Podman => "Podman",
            Feature::Docker => "Docker",
            Feature::Git => "Git",
            Feature::Ssh => "SSH",
            Feature::Smartctl => "smartmontools",
        }
    }

    /// Check whether this feature is present on the current system.
    pub fn is_available(self) -> bool {
        match self {
            Feature::Systemd => check_systemd(),
            Feature::Pam => check_pam(),
            Feature::Launchd => check_launchd(),
            Feature::WindowsServices => cfg!(target_os = "windows"),
            Feature::Podman => binary_in_path("podman"),
            Feature::Docker => binary_in_path("docker"),
            Feature::Git => binary_in_path("git"),
            Feature::Ssh => binary_in_path("ssh"),
            Feature::Smartctl => binary_in_path("smartctl"),
        }
    }

    /// Parse a feature from a kebab-case or snake_case string.
    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.to_lowercase().replace('-', "_").as_str() {
            "systemd" => Some(Feature::Systemd),
            "pam" => Some(Feature::Pam),
            "launchd" => Some(Feature::Launchd),
            "windows_services" => Some(Feature::WindowsServices),
            "podman" => Some(Feature::Podman),
            "docker" => Some(Feature::Docker),
            "git" => Some(Feature::Git),
            "ssh" => Some(Feature::Ssh),
            "smartctl" | "smart" => Some(Feature::Smartctl),
            _ => None,
        }
    }
}

/// All features detected at once.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedFeatures {
    /// The set of features that are available on this system.
    pub available: Vec<Feature>,
}

impl DetectedFeatures {
    /// Check all known features.
    pub fn detect() -> Self {
        let all = [
            Feature::Systemd,
            Feature::Pam,
            Feature::Launchd,
            Feature::WindowsServices,
            Feature::Podman,
            Feature::Docker,
            Feature::Git,
            Feature::Ssh,
            Feature::Smartctl,
        ];
        let available = all.iter().copied().filter(|f| f.is_available()).collect();
        DetectedFeatures { available }
    }

    /// Returns `true` when `feature` is in the detected set.
    pub fn has(&self, feature: Feature) -> bool {
        self.available.contains(&feature)
    }
}

/// Entry point for one-shot or cached feature detection.
pub struct FeatureDetect;

impl FeatureDetect {
    /// Detect all features synchronously.
    pub fn run() -> DetectedFeatures {
        DetectedFeatures::detect()
    }

    /// Check a single feature without building the full set.
    pub fn check(feature: Feature) -> bool {
        feature.is_available()
    }
}

// ── Internal checks ───────────────────────────────────────────────────────────

fn binary_in_path(name: &str) -> bool {
    // Try `which` first; fall back to known PATH directories.
    std::process::Command::new("which")
        .arg(name)
        .output()
        .map(|o| o.status.success())
        .unwrap_or_else(|_| {
            for dir in ["/usr/bin", "/usr/local/bin", "/bin", "/sbin", "/usr/sbin"] {
                if Path::new(dir).join(name).exists() {
                    return true;
                }
            }
            false
        })
}

fn check_systemd() -> bool {
    #[cfg(target_os = "linux")]
    {
        Path::new("/run/systemd/system").exists() || Path::new("/sys/fs/cgroup/systemd").exists()
    }
    #[cfg(not(target_os = "linux"))]
    {
        false
    }
}

fn check_pam() -> bool {
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        Path::new("/etc/pam.d").exists()
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        false
    }
}

fn check_launchd() -> bool {
    #[cfg(target_os = "macos")]
    {
        binary_in_path("launchctl")
    }
    #[cfg(not(target_os = "macos"))]
    {
        false
    }
}
