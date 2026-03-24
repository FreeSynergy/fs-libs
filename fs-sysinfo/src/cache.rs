//! Persistent cache for static sysinfo data (OsInfo + DetectedFeatures).
//!
//! Cache file: `~/.config/fsn/sysinfo.toml`
//! TTL:        24 hours (based on `cached_at` unix timestamp)

use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{DetectedFeatures, OsInfo};

const CACHE_TTL_SECS: i64 = 24 * 3600;

/// Cached static sysinfo data with a unix timestamp.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SysInfoCacheData {
    /// Unix timestamp (seconds since epoch) when the cache was written.
    pub cached_at_unix: i64,
    /// Detected OS information.
    pub os_info: OsInfo,
    /// Detected system features.
    pub features: DetectedFeatures,
}

/// Manages the persistent sysinfo cache file.
pub struct SysInfoCache {
    path: PathBuf,
}

impl SysInfoCache {
    /// Create a cache manager pointing to `~/.config/fsn/sysinfo.toml`.
    pub fn default_path() -> Self {
        let home = home_dir();
        Self {
            path: home.join(".config").join("fsn").join("sysinfo.toml"),
        }
    }

    /// Create a cache manager with a custom path (useful for testing).
    pub fn with_path(path: PathBuf) -> Self {
        Self { path }
    }

    /// Load cached data if it exists and is not older than 24 hours.
    pub fn load(&self) -> Option<SysInfoCacheData> {
        let content = std::fs::read_to_string(&self.path).ok()?;
        let data: SysInfoCacheData = toml::from_str(&content).ok()?;
        let age = now_unix() - data.cached_at_unix;
        if age > CACHE_TTL_SECS {
            return None;
        }
        Some(data)
    }

    /// Save OsInfo + DetectedFeatures to the cache file.
    /// Creates parent directories if they do not exist.
    pub fn save(&self, os_info: OsInfo, features: DetectedFeatures) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let data = SysInfoCacheData {
            cached_at_unix: now_unix(),
            os_info,
            features,
        };
        let content = toml::to_string_pretty(&data)?;
        std::fs::write(&self.path, content)?;
        Ok(())
    }

    /// Delete the cache file, forcing fresh detection on the next call to `get_or_detect`.
    pub fn clear(&self) -> Result<()> {
        if self.path.exists() {
            std::fs::remove_file(&self.path)?;
        }
        Ok(())
    }

    /// Return cached data if fresh, otherwise detect, save, and return.
    pub fn get_or_detect(&self) -> (OsInfo, DetectedFeatures) {
        if let Some(cached) = self.load() {
            return (cached.os_info, cached.features);
        }
        let os_info = OsInfo::detect();
        let features = DetectedFeatures::detect();
        if let Err(e) = self.save(os_info.clone(), features.clone()) {
            tracing::warn!("Could not write sysinfo cache: {e}");
        }
        (os_info, features)
    }

    /// Path to the cache file.
    pub fn path(&self) -> &std::path::Path {
        &self.path
    }
}

fn now_unix() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn home_dir() -> PathBuf {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/tmp"))
}
