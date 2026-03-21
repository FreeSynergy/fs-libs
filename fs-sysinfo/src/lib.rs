//! `fs-sysinfo` — System information detection and alerting for FreeSynergy.Node.
//!
//! # Static data (cached 24 h)
//! - [`OsInfo`]          — OS type, version, architecture, kernel, hostname
//! - [`DetectedFeatures`] — which system features are present (systemd, Podman, …)
//! - [`SysInfoCache`]    — read/write `~/.config/fsn/sysinfo.toml`
//!
//! # Dynamic data (on demand)
//! - [`DiskInfo`]    — partition list with used / free bytes
//! - [`MemInfo`]     — RAM and swap usage
//! - [`ThermalInfo`] — CPU temperature sensors
//!
//! # Optional (feature = "smart")
//! - [`SmartInfo`]   — SMART disk health via `smartctl`
//!
//! # Alerting
//! - [`AlertChecker`]    — compares live metrics against [`AlertThresholds`]
//! - [`SysInfoAlert`]    — returned alerts carry the correct bus topic

pub mod alert;
pub mod cache;
pub mod disk;
pub mod features;
pub mod mem;
pub mod os;
pub mod thermal;

#[cfg(feature = "smart")]
pub mod smart;

pub use alert::{AlertChecker, AlertThresholds, SysInfoAlert};
pub use cache::SysInfoCache;
pub use disk::{DiskInfo, Partition};
pub use features::{DetectedFeatures, Feature, FeatureDetect};
pub use mem::MemInfo;
pub use os::{OsInfo, OsType};
pub use thermal::{CpuTemp, ThermalInfo};

#[cfg(feature = "smart")]
pub use smart::{DriveSmartStatus, SmartInfo};
