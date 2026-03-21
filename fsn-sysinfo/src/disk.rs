//! Disk and partition information (on demand).

use serde::{Deserialize, Serialize};
use sysinfo::Disks;

/// Information about a single disk partition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Partition {
    /// Mount point, e.g. `"/"` or `"/home"`.
    pub mount_point: String,
    /// File system type, e.g. `"ext4"`, `"btrfs"`.
    pub fs_type: String,
    /// Total space in bytes.
    pub total_bytes: u64,
    /// Available (free) space in bytes.
    pub available_bytes: u64,
}

impl Partition {
    /// Used space in bytes.
    pub fn used_bytes(&self) -> u64 {
        self.total_bytes.saturating_sub(self.available_bytes)
    }

    /// Used space as a percentage (0–100).
    pub fn used_percent(&self) -> f64 {
        if self.total_bytes == 0 { return 0.0; }
        (self.used_bytes() as f64 / self.total_bytes as f64) * 100.0
    }
}

/// Snapshot of all mounted disk partitions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskInfo {
    /// All detected partitions.
    pub partitions: Vec<Partition>,
}

impl DiskInfo {
    /// Read current disk usage from the OS.
    pub fn detect() -> Self {
        let disks = Disks::new_with_refreshed_list();
        let partitions = disks
            .iter()
            .map(|d| Partition {
                mount_point:     d.mount_point().to_string_lossy().into_owned(),
                fs_type:         d.file_system().to_string_lossy().into_owned(),
                total_bytes:     d.total_space(),
                available_bytes: d.available_space(),
            })
            .collect();
        DiskInfo { partitions }
    }

    /// The partition with the highest usage percentage.
    pub fn most_used(&self) -> Option<&Partition> {
        self.partitions
            .iter()
            .max_by(|a, b| a.used_percent().partial_cmp(&b.used_percent()).unwrap_or(std::cmp::Ordering::Equal))
    }
}
