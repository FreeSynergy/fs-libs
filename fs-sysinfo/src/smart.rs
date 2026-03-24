//! SMART disk health via `smartctl` (feature = "smart", requires smartmontools).
//!
//! Runs `smartctl -A -H --json=o <device>` per block device found via `lsblk`.
//! Results are parsed from JSON output.  If smartctl is unavailable or a
//! device does not support SMART, the drive is silently skipped.

use serde::{Deserialize, Serialize};
use std::process::Command;

/// SMART health status for a single drive.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveSmartStatus {
    /// Device path, e.g. `"/dev/sda"`.
    pub device: String,
    /// Whether SMART overall assessment is PASSED.
    pub passed: bool,
    /// Raw assessment string from smartctl (e.g. `"PASSED"`, `"FAILED!"`).
    pub assessment: String,
    /// Drive temperature if reported by SMART (degrees Celsius).
    pub temp_celsius: Option<f32>,
    /// Reallocated sector count — 0 means healthy.
    pub reallocated_sectors: Option<u64>,
}

/// SMART info for all detectable block devices.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartInfo {
    /// One entry per detected block device that responded to smartctl.
    pub drives: Vec<DriveSmartStatus>,
}

impl SmartInfo {
    /// Query SMART health for every block device found via `lsblk`.
    pub fn detect() -> Self {
        let devices = Self::list_block_devices();
        let drives = devices
            .into_iter()
            .filter_map(|dev| Self::check_device(&dev))
            .collect();
        SmartInfo { drives }
    }

    fn list_block_devices() -> Vec<String> {
        let out = Command::new("lsblk")
            .args(["-d", "-n", "-o", "NAME"])
            .output()
            .ok();
        let Some(out) = out else { return vec![] };
        String::from_utf8_lossy(&out.stdout)
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| format!("/dev/{}", l.trim()))
            .collect()
    }

    fn check_device(device: &str) -> Option<DriveSmartStatus> {
        let out = Command::new("smartctl")
            .args(["-A", "-H", "--json=o", device])
            .output()
            .ok()?;

        let json: serde_json::Value = serde_json::from_slice(&out.stdout).ok()?;

        let passed = json["smart_status"]["passed"].as_bool().unwrap_or(false);
        let assessment = json["smart_status"]["string"]
            .as_str()
            .unwrap_or("unknown")
            .to_owned();

        let temp_celsius = json["temperature"]["current"].as_f64().map(|v| v as f32);

        let reallocated_sectors = json["ata_smart_attributes"]["table"]
            .as_array()
            .and_then(|attrs| attrs.iter().find(|a| a["id"].as_u64() == Some(5)))
            .and_then(|a| a["raw"]["value"].as_u64());

        Some(DriveSmartStatus {
            device: device.to_owned(),
            passed,
            assessment,
            temp_celsius,
            reallocated_sectors,
        })
    }
}
