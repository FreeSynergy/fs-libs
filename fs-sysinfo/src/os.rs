//! OS type, version, and architecture detection.

use serde::{Deserialize, Serialize};

/// The operating system family.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OsType {
    Linux,
    MacOs,
    Windows,
    Unknown,
}

impl OsType {
    /// Detect the current OS at compile time.
    pub fn detect() -> Self {
        #[cfg(target_os = "linux")]
        {
            OsType::Linux
        }
        #[cfg(target_os = "macos")]
        {
            OsType::MacOs
        }
        #[cfg(target_os = "windows")]
        {
            OsType::Windows
        }
        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            OsType::Unknown
        }
    }

    /// Display label.
    pub fn label(&self) -> &'static str {
        match self {
            OsType::Linux => "Linux",
            OsType::MacOs => "macOS",
            OsType::Windows => "Windows",
            OsType::Unknown => "Unknown",
        }
    }
}

/// Static OS information for this node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsInfo {
    /// Operating system family.
    pub os_type: OsType,
    /// OS version / distribution string, e.g. `"Ubuntu 24.04"`.
    pub version: String,
    /// CPU architecture, e.g. `"x86_64"`, `"aarch64"`.
    pub arch: String,
    /// Kernel version string.
    pub kernel: String,
    /// Hostname.
    pub hostname: String,
}

impl OsInfo {
    /// Detect OS information from the current system.
    pub fn detect() -> Self {
        use sysinfo::System;
        let os_type = OsType::detect();
        let version = System::long_os_version().unwrap_or_else(|| "unknown".into());
        let kernel = System::kernel_version().unwrap_or_else(|| "unknown".into());
        let hostname = System::host_name().unwrap_or_else(|| "unknown".into());
        let arch = std::env::consts::ARCH.to_owned();
        OsInfo {
            os_type,
            version,
            arch,
            kernel,
            hostname,
        }
    }
}
