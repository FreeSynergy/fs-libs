//! CPU temperature readings (on demand).
//!
//! - Linux: reads `/sys/class/thermal/thermal_zone*/temp` and `/sys/class/hwmon/`
//! - macOS: reads SMC sensors via the `sysinfo` Components API
//! - Other platforms: returns an empty sensor list

use serde::{Deserialize, Serialize};

/// A single CPU temperature reading.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuTemp {
    /// Sensor label, e.g. `"Core 0"` or `"cpu_thermal"`.
    pub label: String,
    /// Temperature in degrees Celsius.
    pub temp_celsius: f32,
}

/// A snapshot of all detected CPU temperature sensors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThermalInfo {
    /// All available temperature sensors.
    pub sensors: Vec<CpuTemp>,
}

impl ThermalInfo {
    /// Read current CPU temperatures from the OS.
    pub fn detect() -> Self {
        #[cfg(target_os = "linux")]
        { Self::detect_linux() }
        #[cfg(target_os = "macos")]
        { Self::detect_smc() }
        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        { ThermalInfo { sensors: vec![] } }
    }

    /// Maximum CPU temperature across all sensors (None if no sensors detected).
    pub fn max_temp(&self) -> Option<f32> {
        self.sensors.iter().map(|s| s.temp_celsius).reduce(f32::max)
    }

    // ── Linux ────────────────────────────────────────────────────────────────

    #[cfg(target_os = "linux")]
    fn detect_linux() -> Self {
        let mut sensors = Vec::new();
        sensors.extend(Self::read_thermal_zones());
        sensors.extend(Self::read_hwmon());
        ThermalInfo { sensors }
    }

    #[cfg(target_os = "linux")]
    fn read_thermal_zones() -> Vec<CpuTemp> {
        use std::{fs, path::Path};
        let mut out = Vec::new();
        let base = Path::new("/sys/class/thermal");
        let Ok(entries) = fs::read_dir(base) else { return out };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            if !name.starts_with("thermal_zone") { continue; }
            let temp_path = path.join("temp");
            let type_path = path.join("type");
            if let Ok(raw) = fs::read_to_string(&temp_path) {
                if let Ok(millideg) = raw.trim().parse::<i64>() {
                    let label = fs::read_to_string(&type_path)
                        .map(|s| s.trim().to_owned())
                        .unwrap_or_else(|_| name.into_owned());
                    out.push(CpuTemp { label, temp_celsius: millideg as f32 / 1000.0 });
                }
            }
        }
        out
    }

    #[cfg(target_os = "linux")]
    fn read_hwmon() -> Vec<CpuTemp> {
        use std::{fs, path::Path};
        let mut out = Vec::new();
        let base = Path::new("/sys/class/hwmon");
        let Ok(entries) = fs::read_dir(base) else { return out };
        for entry in entries.flatten() {
            let path = entry.path();
            let mut i = 1u32;
            loop {
                let temp_input = path.join(format!("temp{i}_input"));
                if !temp_input.exists() { break; }
                let label_path = path.join(format!("temp{i}_label"));
                if let Ok(raw) = fs::read_to_string(&temp_input) {
                    if let Ok(millideg) = raw.trim().parse::<i64>() {
                        let label = fs::read_to_string(&label_path)
                            .map(|s| s.trim().to_owned())
                            .unwrap_or_else(|_| format!("hwmon{i}"));
                        out.push(CpuTemp { label, temp_celsius: millideg as f32 / 1000.0 });
                    }
                }
                i += 1;
            }
        }
        out
    }

    // ── macOS ────────────────────────────────────────────────────────────────

    #[cfg(target_os = "macos")]
    fn detect_smc() -> Self {
        use sysinfo::Components;
        let components = Components::new_with_refreshed_list();
        let sensors = components
            .iter()
            .map(|c| CpuTemp { label: c.label().to_owned(), temp_celsius: c.temperature() })
            .collect();
        ThermalInfo { sensors }
    }
}
