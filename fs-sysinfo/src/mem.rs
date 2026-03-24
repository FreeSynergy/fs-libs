//! RAM usage (on demand).

use serde::{Deserialize, Serialize};
use sysinfo::System;

/// Current memory statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemInfo {
    /// Total RAM in bytes.
    pub total_bytes: u64,
    /// Available (free + reclaimable) RAM in bytes.
    pub available_bytes: u64,
    /// Actually used RAM in bytes.
    pub used_bytes: u64,
    /// Total swap in bytes.
    pub swap_total_bytes: u64,
    /// Used swap in bytes.
    pub swap_used_bytes: u64,
}

impl MemInfo {
    /// Read current memory usage from the OS.
    pub fn detect() -> Self {
        let mut sys = System::new();
        sys.refresh_memory();
        MemInfo {
            total_bytes: sys.total_memory(),
            available_bytes: sys.available_memory(),
            used_bytes: sys.used_memory(),
            swap_total_bytes: sys.total_swap(),
            swap_used_bytes: sys.used_swap(),
        }
    }

    /// Used memory as a percentage (0–100).
    pub fn used_percent(&self) -> f64 {
        if self.total_bytes == 0 {
            return 0.0;
        }
        (self.used_bytes as f64 / self.total_bytes as f64) * 100.0
    }
}
